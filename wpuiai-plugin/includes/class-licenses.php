<?php
/**
 * License Management
 *
 * @package WPUIAI_SSC
 */

defined('ABSPATH') || exit;

class WPUIAI_AIC_Licenses {

    private static $instance = null;

    public static function instance(): self {
        if (self::$instance === null) {
            self::$instance = new self();

            add_action('admin_enqueue_scripts', [self::$instance, 'enqueue_scripts']);
            add_action('wp_ajax_wpuiai_aic_license_details', [self::$instance, 'ajax_license_details']);
            add_action('wp_ajax_wpuiai_aic_activate_license', [self::$instance, 'ajax_activate_license']);
            add_action('wp_ajax_wpuiai_aic_deactivate_license', [self::$instance, 'ajax_deactivate_license']);
            add_action('wp_ajax_wpuiai_aic_assign_license', [self::$instance, 'ajax_assign_license']);
            add_action('wp_ajax_wpuiai_aic_bulk_licenses', [self::$instance, 'ajax_bulk_licenses']);
            add_action('wp_ajax_wpuiai_aic_search_users', [self::$instance, 'ajax_search_users']);
            add_action('wp_ajax_wpuiai_aic_delete_license', [self::$instance, 'ajax_delete_license']);
            add_action('wp_ajax_wpuiai_aic_set_distribution_limit', [self::$instance, 'ajax_set_distribution_limit']);
        }
        return self::$instance;
    }


    public function enqueue_scripts(string $hook): void {
        if (strpos($hook, 'wpuiai-ai-cloud-licenses') === false) {
            return;
        }

        $css_ver = file_exists(WPUIAI_AIC_PLUGIN_DIR . 'assets/css/licenses.css') ? filemtime(WPUIAI_AIC_PLUGIN_DIR . 'assets/css/licenses.css') : WPUIAI_AIC_VERSION;
        $js_ver = file_exists(WPUIAI_AIC_PLUGIN_DIR . 'assets/js/licenses.js') ? filemtime(WPUIAI_AIC_PLUGIN_DIR . 'assets/js/licenses.js') : WPUIAI_AIC_VERSION;

        wp_enqueue_style('wpuiai-aic-licenses', WPUIAI_AIC_PLUGIN_URL . 'assets/css/licenses.css', [], $css_ver);
        wp_enqueue_script('wpuiai-aic-licenses', WPUIAI_AIC_PLUGIN_URL . 'assets/js/licenses.js', ['jquery'], $js_ver, true);

        wp_localize_script('wpuiai-aic-licenses', 'wpuiaiAICLicenses', [
            'nonce' => wp_create_nonce('wpuiai_aic_licenses'),
            'ajaxurl' => admin_url('admin-ajax.php'),
            'strings' => [
                'confirmActivate' => 'Are you sure you want to activate this license?',
                'confirmDeactivate' => 'Are you sure you want to deactivate this license?',
                'confirmDelete' => 'Are you sure you want to delete this license? This removes all activations, payment plans, and seats — it cannot be undone.',
                'activated' => 'License activated successfully.',
                'deactivated' => 'License deactivated successfully.',
                'assigned' => 'License assigned successfully.',
                'deleted' => 'License deleted successfully.',
                'limitUpdated' => 'Distribution limit updated.',
                'error' => 'An error occurred. Please try again.',
            ],
        ]);
    }

    public function render_page(): void {
        if (!current_user_can('manage_options')) {
            wp_die('Access denied');
        }

        $search = isset($_GET['search']) ? sanitize_text_field($_GET['search']) : '';
        $status_filter = isset($_GET['status']) ? sanitize_text_field($_GET['status']) : '';
        $tier_filter = isset($_GET['tier']) ? sanitize_text_field($_GET['tier']) : '';

        $page = isset($_GET['paged']) ? max(1, intval($_GET['paged'])) : 1;
        $per_page = 20;
        $offset = ($page - 1) * $per_page;

        $total = $this->get_total_licenses($search, $status_filter, $tier_filter);
        $licenses = $this->get_licenses($offset, $per_page, $search, $status_filter, $tier_filter);

        ?>
        <div class="wrap wpuiai-aic-wrap">
            <h1><span class="dashicons dashicons-admin-network"></span> Licenses</h1>

            <div class="wpuiai-aic-page-header">
                <a href="<?php echo esc_url(admin_url('edit.php?post_type=download&page=edd-licenses&view=add')); ?>" class="button button-primary">
                    <span class="dashicons dashicons-plus"></span> Create New License
                </a>
            </div>

            <?php
            // Distribution limits — per-product cap (e.g. 50 Operator). Default 50 for 1736, unlimited otherwise.
            $dist_stats = $this->get_distribution_stats();
            $dist_products = [1736 => 'Focusa Operator (1736)', 1735 => 'Focusa Evaluation (1735)', 452 => 'WPUIAI Starter LTD (452)', 453 => 'WPUIAI Pro LTD (453)'];
            ?>
            <div class="wpuiai-aic-distribution-limits" style="background:#fff;border:1px solid #ccd0d4;border-radius:8px;padding:16px;margin:14px 0;">
                <h3 style="margin:0 0 12px;"><span class="dashicons dashicons-chart-bar" style="margin-top:2px;"></span> Distribution Limits <small style="font-weight:normal;color:#666;">— set/reset caps for Operator etc (default 50 Operator)</small></h3>
                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:12px;">
                <?php foreach ($dist_products as $did => $label):
                    $st = $dist_stats[$did] ?? ['distributed'=>0,'limit'=>null,'remaining'=>null];
                    $limit_display = $st['limit'] === null ? 'Unlimited' : number_format($st['limit']);
                    $rem_display = $st['remaining'] === null ? '—' : number_format($st['remaining']) . ' left';
                    $pct = ($st['limit'] && $st['limit']>0) ? min(100, max(0, ($st['distributed']/$st['limit'])*100)) : 0;
                    $bar_color = ($st['remaining'] !== null && $st['remaining']<=0) ? '#d63638' : '#2271b1';
                ?>
                    <div class="wpuiai-aic-dist-card" data-download="<?php echo (int)$did; ?>" style="border:1px solid #dcdcde;border-radius:6px;padding:12px;background:#f9f9f9;">
                        <div style="font-weight:600;font-size:13px;"><?php echo esc_html($label); ?> <small style="color:#666;"> #<?php echo (int)$did; ?></small></div>
                        <div style="margin:8px 0;display:flex;gap:12px;align-items:baseline;flex-wrap:wrap;">
                            <span><strong><?php echo number_format($st['distributed']); ?></strong> distributed</span>
                            <span>Limit: <strong class="dist-limit-value"><?php echo esc_html($limit_display); ?></strong></span>
                            <span style="color:#555;"><?php echo esc_html($rem_display); ?></span>
                        </div>
                        <div style="background:#eee;border-radius:3px;height:6px;margin:6px 0;"><div style="height:6px;background:<?php echo esc_attr($bar_color); ?>;width:<?php echo esc_attr($pct); ?>%;border-radius:3px;"></div></div>
                        <div style="display:flex;gap:6px;align-items:center;flex-wrap:wrap;margin-top:8px;">
                            <input type="number" min="0" placeholder="Limit (blank=unlimited)" class="dist-limit-input" data-download="<?php echo (int)$did; ?>" style="width:150px;" value="<?php echo $st['limit'] === null ? '' : (int)$st['limit']; ?>">
                            <button type="button" class="button button-small dist-set-limit" data-download="<?php echo (int)$did; ?>">Set limit</button>
                            <button type="button" class="button button-small dist-reset-limit" data-download="<?php echo (int)$did; ?>" title="Reset to default (unlimited, except 1736=50)">Reset</button>
                        </div>
                    </div>
                <?php endforeach; ?>
                </div>
                <p class="description" style="margin:8px 0 0;">Limits are enforced on grant — when distributed ≥ limit, new grants are blocked (E_DISTRIBUTION_LIMIT). Delete test licenses to reclaim slots; distributed counts are live from <code>wp_edd_licenses</code>.</p>
            </div>
            <div class="wpuiai-aic-filters">
                <input type="text" id="license-search" name="search" value="<?php echo esc_attr($search); ?>" placeholder="Search by email, license key, or name...">
                <select id="license-status-filter" name="status">
                    <option value="">All Status</option>
                    <option value="active" <?php selected($status_filter, 'active'); ?>>Active</option>
                    <option value="inactive" <?php selected($status_filter, 'inactive'); ?>>Inactive</option>
                    <option value="expired" <?php selected($status_filter, 'expired'); ?>>Expired</option>
                    <option value="disabled" <?php selected($status_filter, 'disabled'); ?>>Disabled</option>
                </select>
                <select id="license-tier-filter" name="tier">
                    <option value="">All Tiers</option>
                    <?php foreach ($this->get_tier_labels() as $value => $label): ?>
                        <option value="<?php echo esc_attr($value); ?>" <?php selected($tier_filter, $value); ?>><?php echo esc_html($label); ?></option>
                    <?php endforeach; ?>
                </select>
                <button type="button" class="button" id="filter-licenses">Filter</button>
                <button type="button" class="button" id="reset-filters">Reset</button>
            </div>

            <div class="wpuiai-aic-stats-cards">
                <div class="wpuiai-aic-card">
                    <div class="stat-value"><?php echo number_format($total); ?></div>
                    <div class="stat-label">Total Licenses</div>
                </div>
                <div class="wpuiai-aic-card">
                    <div class="stat-value"><?php echo number_format($this->get_count_by_status('active')); ?></div>
                    <div class="stat-label">Active</div>
                </div>
                <div class="wpuiai-aic-card">
                    <div class="stat-value"><?php echo number_format($this->get_count_by_status('expired')); ?></div>
                    <div class="stat-label">Expired</div>
                </div>
                <div class="wpuiai-aic-card">
                    <div class="stat-value"><?php echo number_format($this->get_count_by_status('disabled')); ?></div>
                    <div class="stat-label">Disabled</div>
                </div>
            </div>

            <table class="wp-list-table widefat fixed striped wpuiai-aic-licenses-table">
                <thead>
                    <tr>
                        <th><input type="checkbox" id="select-all-licenses"></th>
                        <th>License Key</th>
                        <th>User</th>
                        <th>Email</th>
                        <th>Tier</th>
                        <th>Status</th>
                        <th>Activations</th>
                        <th>Expiry</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    <?php if (empty($licenses)): ?>
                        <tr>
                            <td colspan="9" style="text-align:center;">No licenses found.</td>
                        </tr>
                    <?php else: ?>
                        <?php foreach ($licenses as $license): ?>
                        <tr class="license-<?php echo esc_attr($license['status']); ?>">
                            <td data-label="Select"><input type="checkbox" class="license-checkbox" data-id="<?php echo esc_attr($license['id']); ?>"></td>
                            <td data-label="License Key">
                                <code class="license-key"><?php echo esc_html(substr($license['license_key'], 0, 16)) . '...'; ?></code>
                            </td>
                            <td data-label="User">
                                <a href="<?php echo esc_url(admin_url('user-edit.php?user_id=' . $license['user_id'])); ?>">
                                    <?php echo esc_html($license['user_name']); ?>
                                </a>
                            </td>
                            <td data-label="Email"><?php echo esc_html($license['user_email']); ?></td>
                            <td data-label="Tier"><span class="tier-badge tier-<?php echo esc_attr($license['tier']); ?>"><?php echo esc_html($license['tier_label']); ?></span></td>
                            <td data-label="Status">
                                <span class="wpuiai-aic-status status-<?php echo esc_attr($license['status']); ?>">
                                    <?php echo esc_html(ucfirst($license['status'])); ?>
                                </span>
                            </td>
                            <td data-label="Activations"><?php echo number_format($license['activation_count']); ?></td>
                            <td data-label="Expiry">
                                <?php if ($license['expires_at']): ?>
                                    <?php $days_until_expiry = $this->days_until($license['expires_at']); ?>
                                    <span class="<?php echo $days_until_expiry <= 30 ? 'expires-soon' : ''; ?>">
                                        <?php echo esc_html($this->format_date($license['expires_at'])); ?>
                                    </span>
                                    <br>
                                    <small>(<?php echo $days_until_expiry ?> days)</small>
                                <?php else: ?>
                                    Lifetime
                                <?php endif; ?>
                            </td>
                            <td data-label="Actions">
                                <div class="wpuiai-aic-action-buttons">
                                    <a href="<?php echo esc_url(admin_url('edit.php?post_type=download&page=edd-licenses&view=overview&id=' . $license['id'])); ?>" class="button button-small" target="_blank">
                                        Edit in EDD
                                    </a>
                                    <button type="button" class="button button-small view-license-details" data-id="<?php echo esc_attr($license['id']); ?>">
                                        View Details
                                    </button>
                                    <?php if ($license['status'] === 'active'): ?>
                                        <button type="button" class="button button-small deactivate-license" data-id="<?php echo esc_attr($license['id']); ?>">
                                            Deactivate
                                        </button>
                                    <?php elseif ($license['status'] === 'inactive'): ?>
                                        <button type="button" class="button button-small button-secondary activate-license" data-id="<?php echo esc_attr($license['id']); ?>">
                                            Activate
                                        </button>
                                    <?php endif; ?>
                                    <button type="button" class="button button-small button-link-delete delete-license" data-id="<?php echo esc_attr($license['id']); ?>">
                                        Delete
                                    </button>
                                </div>
                            </td>
                        </tr>
                        <?php endforeach; ?>
                    <?php endif; ?>
                </tbody>
            </table>

            <?php if ($total > $per_page): ?>
                <div class="tablenav bottom">
                    <?php echo paginate_links([
                        'base' => admin_url('admin.php?page=wpuiai-ai-cloud-licenses'),
                        'format' => '?paged=%#%#',
                        'total' => $total,
                        'current' => $page,
                        'prev_text' => '&laquo; Previous',
                        'next_text' => 'Next &raquo;',
                    ]); ?>
                </div>
            <?php endif; ?>

            <div class="wpuiai-aic-bulk-actions">
                <strong>Bulk Actions:</strong>
                <select id="bulk-action">
                    <option value="">Select Action</option>
                    <option value="activate">Activate</option>
                    <option value="deactivate">Deactivate</option>
                    <option value="disable">Disable</option>
                    <option value="delete">Delete</option>
                </select>
                <button type="button" class="button" id="apply-bulk-action" disabled>Apply</button>
            </div>
        </div>

        <div id="license-details-modal" class="wpuiai-aic-modal" style="display:none;">
            <div class="wpuiai-aic-modal-content">
                <div class="wpuiai-aic-modal-header">
                    <h2>License Details</h2>
                    <button type="button" class="close-modal">&times;</button>
                </div>
                <div class="wpuiai-aic-modal-body">
                    <div class="loading">Loading...</div>
                </div>
            </div>
        </div>
        <?php
    }

    public function ajax_license_details(): void {
        check_ajax_referer('wpuiai_aic_licenses', 'nonce');

        if (!current_user_can('manage_options')) {
            wp_send_json_error(['message' => 'Permission denied']);
            return;
        }

        $license_id = intval($_POST['license_id'] ?? 0);
        if ($license_id <= 0) {
            wp_send_json_error(['message' => 'Invalid license ID']);
            return;
        }

        $license = $this->get_license_details($license_id);
        if (!$license) {
            wp_send_json_error(['message' => 'License not found']);
            return;
        }

        $stats = $this->get_license_stats($license_id);
        $usage = $this->get_license_usage($license_id);

        ob_start();
        ?>
        <table class="form-table">
            <tr><th>License Key:</th><td><code><?php echo esc_html($license['license_key']); ?></code> <button class="button-small copy-key" data-key="<?php echo esc_attr($license['license_key']); ?>">Copy</button></td></tr>
            <tr><th>Tier:</th><td><span class="tier-badge tier-<?php echo esc_attr($license['tier']); ?>"><?php echo esc_html($license['tier_label']); ?></span></td></tr>
            <tr><th>Status:</th><td><span class="wpuiai-aic-status status-<?php echo esc_attr($license['status']); ?>"><?php echo esc_html(ucfirst($license['status'])); ?></span></td></tr>
            <tr><th>User:</th><td><a href="<?php echo esc_url(admin_url('user-edit.php?user_id=' . $license['user_id'])); ?>"><?php echo esc_html($license['user_name']); ?></a></td></tr>
            <tr><th>Email:</th><td><?php echo esc_html($license['user_email']); ?></td></tr>
            <tr><th>Created:</th><td><?php echo esc_html($this->format_date($license['created_at'])); ?></td></tr>
            <tr><th>Expiry:</th><td><?php echo $license['expires_at'] ? esc_html($this->format_date($license['expires_at'])) : 'Lifetime'; ?></td></tr>
            <tr><th>Activations:</th><td><?php echo number_format($license['activation_count']); ?></td></tr>
        </table>

        <h3>Assign to User</h3>
        <div class="assign-user-form">
            <input type="text" id="assign-user-search" placeholder="Search by username or email...">
            <button type="button" class="button" id="search-users-btn">Search</button>
            <div id="user-search-results"></div>
            <input type="hidden" id="assign-license-id" value="<?php echo esc_attr($license_id); ?>">
            <input type="hidden" id="selected-user-id">
            <button type="button" class="button button-primary" id="confirm-assign-user" disabled>Assign to Selected User</button>
        </div>

        <h3>Usage Stats</h3>
        <div class="wpuiai-aic-stats-cards">
            <div class="wpuiai-aic-card">
                <div class="stat-value"><?php echo number_format($stats['screenshots']); ?></div>
                <div class="stat-label">Screenshots</div>
            </div>
            <div class="wpuiai-aic-card">
                <div class="stat-value"><?php echo number_format($stats['critiques']); ?></div>
                <div class="stat-label">Critiques</div>
            </div>
            <div class="wpuiai-aic-card">
                <div class="stat-value"><?php echo number_format($stats['ui_reverse']); ?></div>
                <div class="stat-label">UI Reverse</div>
            </div>
        </div>

        <h3>Feature Limits</h3>
        <table class="form-table">
            <?php foreach ($this->get_tier_limits($license['tier']) as $feature => $limit): ?>
                <tr>
                    <th><?php echo esc_html($feature); ?>:</th>
                    <td><?php echo esc_html($limit); ?></td>
                </tr>
            <?php endforeach; ?>
        </table>

        <h3>Recent Usage</h3>
        <table class="wp-list-table widefat fixed striped wpuiai-aic-license-usage">
            <thead>
                <tr>
                    <th>Type</th>
                    <th>Date</th>
                    <th>Status</th>
                </tr>
            </thead>
            <tbody>
                <?php foreach ($usage as $item): ?>
                    <tr>
                        <td data-label="Type"><?php echo esc_html($item['type']); ?></td>
                        <td data-label="Date"><?php echo esc_html($this->format_date($item['created_at'])); ?></td>
                        <td data-label="Status"><span class="wpuiai-aic-status status-<?php echo esc_attr($item['status']); ?>"><?php echo esc_html($item['status']); ?></span></td>
                    </tr>
                <?php endforeach; ?>
            </tbody>
        </table>
        <?php
        $html = ob_get_clean();

        wp_send_json_success(['html' => $html]);
    }

    public function ajax_activate_license(): void {
        check_ajax_referer('wpuiai_aic_licenses', 'nonce');

        if (!current_user_can('manage_options')) {
            wp_send_json_error(['message' => 'Permission denied']);
            return;
        }

        $license_id = intval($_POST['license_id'] ?? 0);
        if ($license_id <= 0) {
            wp_send_json_error(['message' => 'Invalid license ID']);
            return;
        }

        global $wpdb;
        $result = $wpdb->update(
            $wpdb->prefix . 'edd_licenses',
            ['status' => 'active'],
            ['id' => $license_id],
            ['%s'],
            ['%d']
        );

        if ($result !== false) {
            wp_send_json_success(['message' => 'License activated successfully']);
        } else {
            wp_send_json_error(['message' => 'Failed to activate license']);
        }
    }

    public function ajax_deactivate_license(): void {
        check_ajax_referer('wpuiai_aic_licenses', 'nonce');

        if (!current_user_can('manage_options')) {
            wp_send_json_error(['message' => 'Permission denied']);
            return;
        }

        $license_id = intval($_POST['license_id'] ?? 0);
        if ($license_id <= 0) {
            wp_send_json_error(['message' => 'Invalid license ID']);
            return;
        }

        global $wpdb;
        $result = $wpdb->update(
            $wpdb->prefix . 'edd_licenses',
            ['status' => 'inactive'],
            ['id' => $license_id],
            ['%s'],
            ['%d']
        );

        if ($result !== false) {
            wp_send_json_success(['message' => 'License deactivated successfully']);
        } else {
            wp_send_json_error(['message' => 'Failed to deactivate license']);
        }
    }

    public function ajax_assign_license(): void {
        check_ajax_referer('wpuiai_aic_licenses', 'nonce');

        if (!current_user_can('manage_options')) {
            wp_send_json_error(['message' => 'Permission denied']);
            return;
        }

        $user_id = intval($_POST['user_id'] ?? 0);
        if ($user_id <= 0) {
            wp_send_json_error(['message' => 'Invalid user ID']);
            return;
        }

        $license_id = intval($_POST['license_id'] ?? 0);
        if ($license_id <= 0) {
            wp_send_json_error(['message' => 'Invalid license ID']);
            return;
        }

        global $wpdb;
        $result = $wpdb->update(
            $wpdb->prefix . 'edd_licenses',
            ['user_id' => $user_id],
            ['id' => $license_id],
            ['%d'],
            ['%d']
        );

        if ($result !== false) {
            wp_send_json_success(['message' => 'License assigned successfully']);
        } else {
            wp_send_json_error(['message' => 'Failed to assign license']);
        }
    }

    public function ajax_search_users(): void {
        check_ajax_referer('wpuiai_aic_licenses', 'nonce');

        if (!current_user_can('manage_options')) {
            wp_send_json_error(['message' => 'Permission denied']);
            return;
        }

        $search = sanitize_text_field($_POST['search'] ?? '');
        if (strlen($search) < 2) {
            wp_send_json_error(['message' => 'Search term too short']);
            return;
        }

        global $wpdb;
        $users = $wpdb->get_results($wpdb->prepare(
            "SELECT ID, user_login, user_email, display_name 
             FROM {$wpdb->users} 
             WHERE user_login LIKE %s OR user_email LIKE %s OR display_name LIKE %s
             LIMIT 10",
            ["%$search%", "%$search%", "%$search%"]
        ));

        $results = [];
        foreach ($users as $user) {
            $results[] = [
                'id' => $user->ID,
                'name' => "$user->display_name ($user->user_email)",
                'email' => $user->user_email,
            ];
        }

        wp_send_json_success(['users' => $results]);
    }

    public function ajax_bulk_licenses(): void {
        check_ajax_referer('wpuiai_aic_licenses', 'nonce');

        if (!current_user_can('manage_options')) {
            wp_send_json_error(['message' => 'Permission denied']);
            return;
        }

        $license_ids = isset($_POST['license_ids']) ? array_map('intval', $_POST['license_ids']) : [];
        // WP ajax uses $_POST['action'] for routing, so bulk action is in bulk_action / bulkAction
        $action = sanitize_text_field($_POST['bulk_action'] ?? $_POST['bulkAction'] ?? '');
        if ($action === '') $action = sanitize_text_field($_POST['action'] === 'wpuiai_aic_bulk_licenses' ? '' : ($_POST['action'] ?? ''));

        if (empty($license_ids) || empty($action)) {
            wp_send_json_error(['message' => 'Invalid parameters']);
            return;
        }

        global $wpdb;
        $results = ['success' => 0, 'failed' => 0];

        foreach ($license_ids as $license_id) {
            $result = false;
            switch ($action) {
                case 'activate':
                    $result = $wpdb->update(
                        $wpdb->prefix . 'edd_licenses',
                        ['status' => 'active'],
                        ['id' => $license_id],
                        ['%s'],
                        ['%d']
                    );
                    break;
                case 'deactivate':
                    $result = $wpdb->update(
                        $wpdb->prefix . 'edd_licenses',
                        ['status' => 'inactive'],
                        ['id' => $license_id],
                        ['%s'],
                        ['%d']
                    );
                    break;
                case 'disable':
                    $result = $wpdb->update(
                        $wpdb->prefix . 'edd_licenses',
                        ['status' => 'disabled'],
                        ['id' => $license_id],
                        ['%s'],
                        ['%d']
                    );
                    break;
                case 'delete':
                    $result = $this->delete_license_cascade($license_id);
                    break;
            }

            if ($result !== false) {
                $results['success']++;
            } else {
                $results['failed']++;
            }
        }

        wp_send_json_success($results);
    }

    public function ajax_delete_license(): void {
        check_ajax_referer('wpuiai_aic_licenses', 'nonce');
        if (!current_user_can('manage_options')) { wp_send_json_error(['message'=>'Permission denied']); return; }
        $license_id = intval($_POST['license_id'] ?? 0);
        if ($license_id <= 0) { wp_send_json_error(['message'=>'Invalid license ID']); return; }
        $ok = $this->delete_license_cascade($license_id);
        if ($ok !== false) wp_send_json_success(['message'=>'License deleted','license_id'=>$license_id]);
        else wp_send_json_error(['message'=>'Failed to delete license']);
    }

    public function ajax_set_distribution_limit(): void {
        check_ajax_referer('wpuiai_aic_licenses', 'nonce');
        if (!current_user_can('manage_options')) { wp_send_json_error(['message'=>'Permission denied']); return; }
        $download_id = intval($_POST['download_id'] ?? 0);
        $limit_raw = isset($_POST['limit']) ? trim(sanitize_text_field($_POST['limit'])) : '';
        if ($download_id <= 0) { wp_send_json_error(['message'=>'Invalid download ID']); return; }
        $limits = $this->get_distribution_limits_raw();
        if ($limit_raw === '' || $limit_raw === 'null' || $limit_raw === 'unlimited') {
            unset($limits[$download_id]);
        } else {
            $limit = intval($limit_raw);
            if ($limit < 0) $limit = 0;
            $limits[$download_id] = $limit;
        }
        update_option('wpuiai_license_distribution_limits', $limits, false);
        $stats = $this->get_distribution_stats();
        wp_send_json_success(['limits'=>$limits,'stats'=>$stats]);
    }

    // ── Distribution limits helpers ───────────────────────
    private function get_distribution_limits_raw(): array {
        $raw = get_option('wpuiai_license_distribution_limits', []);
        if (!is_array($raw)) $raw = [];
        // Defaults: 1736 Operator capped at 50 unless explicitly overridden
        $defaults = [1736 => 50];
        foreach ($defaults as $did=>$def) { if (!array_key_exists($did, $raw)) $raw[$did] = $def; }
        return $raw;
    }
    public function get_distribution_limits(): array { return $this->get_distribution_limits_raw(); }
    public function get_distribution_count(int $download_id): int {
        global $wpdb; return (int)$wpdb->get_var($wpdb->prepare("SELECT COUNT(*) FROM {$wpdb->prefix}edd_licenses WHERE download_id=%d", $download_id));
    }
    public function get_distribution_stats(): array {
        $limits = $this->get_distribution_limits_raw();
        $targets = [1736,1735,452,453];
        $out=[];
        foreach ($targets as $did) {
            $cnt = $this->get_distribution_count($did);
            $lim = array_key_exists($did, $limits) ? $limits[$did] : null;
            // For non-default products, null=unlimited
            if ($did !== 1736 && $lim === 50 && !array_key_exists($did, get_option('wpuiai_license_distribution_limits', []))) { /* don't auto-apply 1736 default to others */ }
            // Correct unlimited handling
            $raw_opt = get_option('wpuiai_license_distribution_limits', []);
            if (!is_array($raw_opt)) $raw_opt=[];
            if (!array_key_exists($did, $raw_opt)) {
                $lim = ($did === 1736) ? 50 : null;
            } else {
                $lim = $raw_opt[$did];
                if ($lim === null || $lim === '' ) $lim = null;
            }
            $rem = ($lim===null) ? null : max(0, $lim - $cnt);
            $out[$did]=['distributed'=>$cnt,'limit'=>$lim,'remaining'=>$rem];
        }
        return $out;
    }
    public function is_distribution_limit_reached(int $download_id): bool {
        $stats = $this->get_distribution_stats();
        if (!isset($stats[$download_id])) return false;
        $s=$stats[$download_id];
        if ($s['limit']===null) return false;
        return $s['distributed'] >= $s['limit'];
    }
    /**
     * Cascading delete — removes license + metas + activations + payment plans/payments + machines + usage refs.
     * Returns true on success, false on failure. Never leaves orphans for test licenses.
     */
    private function delete_license_cascade(int $license_id) {
        global $wpdb;
        $lid = intval($license_id);
        if ($lid <= 0) return false;
        // Verify exists
        $exists = $wpdb->get_var($wpdb->prepare("SELECT id FROM {$wpdb->prefix}edd_licenses WHERE id=%d", $lid));
        if (!$exists) return false;
        $wpdb->query('START TRANSACTION');
        try {
            $pfx = $wpdb->prefix;
            // Activations (EDD 3.x)
            $wpdb->query($wpdb->prepare("DELETE FROM {$pfx}edd_license_activations WHERE license_id=%d", $lid));
            // Licensemeta
            $wpdb->query($wpdb->prepare("DELETE FROM {$pfx}edd_licensemeta WHERE edd_license_id=%d", $lid));
            // Payment plans & payments
            $wpdb->query($wpdb->prepare("DELETE FROM {$pfx}wpuiai_license_payments WHERE license_id=%d", $lid));
            $wpdb->query($wpdb->prepare("DELETE FROM {$pfx}wpuiai_license_payment_plans WHERE license_id=%d", $lid));
            // Focusa machines/meta if tables exist
            $mach = $pfx . 'wpuiai_license_machines';
            if ($wpdb->get_var($wpdb->prepare('SHOW TABLES LIKE %s', $mach)) === $mach) {
                $wpdb->query($wpdb->prepare("DELETE FROM {$mach} WHERE license_id=%d", $lid));
            }
            $meta = $pfx . 'wpuiai_license_meta';
            if ($wpdb->get_var($wpdb->prepare('SHOW TABLES LIKE %s', $meta)) === $meta) {
                $wpdb->query($wpdb->prepare("DELETE FROM {$meta} WHERE license_id=%d", $lid));
            }
            // Also wpuiai_license_machines meta table if exists
            $mach_meta = $pfx . 'wpuiai_license_machines_meta';
            if ($wpdb->get_var($wpdb->prepare('SHOW TABLES LIKE %s', $mach_meta)) === $mach_meta) {
                $wpdb->query($wpdb->prepare("DELETE FROM {$mach_meta} WHERE license_id=%d", $lid));
            }
            // Client keys linked
            $ck = $pfx . 'uiai_client_keys';
            if ($wpdb->get_var($wpdb->prepare('SHOW TABLES LIKE %s', $ck)) === $ck) {
                $wpdb->query($wpdb->prepare("DELETE FROM {$ck} WHERE license_id=%d", $lid));
            }
            // Finally license row
            $del = $wpdb->delete($pfx.'edd_licenses', ['id'=>$lid], ['%d']);
            if ($del === false) throw new Exception('delete_failed');
            $wpdb->query('COMMIT');
            return true;
        } catch (Exception $e) {
            $wpdb->query('ROLLBACK');
            return false;
        }
    }

    private function get_licenses(int $offset, int $per_page, string $search = '', string $status = '', string $tier = ''): array {
        global $wpdb;

        // EDD 3.x: edd_licenses has download_id directly (no separate edd_software_licenses table)
        // EDD 3.x: edd_license_activations replaces old edd_sl_activations
        $activations_table = $wpdb->prefix . 'edd_license_activations';
        $activations_exist = $wpdb->get_var( $wpdb->prepare( 'SHOW TABLES LIKE %s', $activations_table ) ) === $activations_table;

        // download_id is directly on edd_licenses — no JOIN needed
        $activation_count_sql = $activations_exist
            ? "(SELECT COUNT(*) FROM {$activations_table} WHERE license_id = l.id) as activation_count"
            : "0 as activation_count";

        // Tier comes from l.download_id directly (EDD 3.x)
        $tier_sql = "l.download_id as tier";
        $tier_join = "";

        $where = [];
        $placeholders = [];

        if (!empty($search)) {
            $where[] = "(u.user_email LIKE %s OR l.license_key LIKE %s OR u.display_name LIKE %s)";
            $placeholders[] = "%$search%";
            $placeholders[] = "%$search%";
            $placeholders[] = "%$search%";
        }

        if (!empty($status)) {
            $where[] = "l.status = %s";
            $placeholders[] = $status;
        }

        if (!empty($tier) && $edd_sl_exists) {
            $where[] = "sl.download_id = %d";
            $placeholders[] = $this->get_tier_product_id($tier);
        }

        $where_clause = !empty($where) ? 'WHERE ' . implode(' AND ', $where) : '';

        $sql = $wpdb->prepare(
            "SELECT l.id, l.license_key, l.user_id, l.status, l.expiration as expires_at, l.date_created as created_at,
                    u.display_name as user_name, u.user_email, $tier_sql, $activation_count_sql
             FROM {$wpdb->prefix}edd_licenses l
             INNER JOIN {$wpdb->users} u ON l.user_id = u.ID
             $tier_join
             $where_clause
             ORDER BY l.date_created DESC
             LIMIT %d OFFSET %d",
            array_merge($placeholders, [$per_page, $offset])
        );

        $results = $wpdb->get_results($sql);

        $licenses = [];
        foreach ($results as $row) {
            $licenses[] = [
                'id' => $row->id,
                'license_key' => $row->license_key,
                'user_id' => $row->user_id,
                'user_name' => $row->user_name,
                'user_email' => $row->user_email,
                'status' => $row->status,
                'tier' => $this->get_tier_label($row->tier),
                'tier_label' => $this->get_tier_label($row->tier),
                'expires_at' => $row->expires_at,
                'created_at' => $row->created_at,
                'activation_count' => $row->activation_count,
            ];
        }

        return $licenses;
    }

    private function get_total_licenses(string $search = '', string $status = '', string $tier = ''): int {
        global $wpdb;

        // EDD 3.x: download_id is on edd_licenses directly — no JOIN needed
        $tier_join = "";

        $where = [];
        $placeholders = [];

        if (!empty($search)) {
            $where[] = "(u.user_email LIKE %s OR l.license_key LIKE %s OR u.display_name LIKE %s)";
            $placeholders[] = "%$search%";
            $placeholders[] = "%$search%";
            $placeholders[] = "%$search%";
        }

        if (!empty($status)) {
            $where[] = "l.status = %s";
            $placeholders[] = $status;
        }

        if (!empty($tier) && $edd_sl_exists) {
            $where[] = "sl.download_id = %d";
            $placeholders[] = $this->get_tier_product_id($tier);
        }

        $where_clause = !empty($where) ? 'WHERE ' . implode(' AND ', $where) : '';

        if (!empty($placeholders)) {
            $sql = $wpdb->prepare(
                "SELECT COUNT(*)
                 FROM {$wpdb->prefix}edd_licenses l
                 INNER JOIN {$wpdb->users} u ON l.user_id = u.ID
                 $tier_join
                 $where_clause",
                $placeholders
            );
        } else {
            $sql = "SELECT COUNT(*) FROM {$wpdb->prefix}edd_licenses l INNER JOIN {$wpdb->users} u ON l.user_id = u.ID $tier_join $where_clause";
        }

        return (int) $wpdb->get_var($sql);
    }

    private function get_count_by_status(string $status): int {
        global $wpdb;
        return (int) $wpdb->get_var($wpdb->prepare(
            "SELECT COUNT(*) FROM {$wpdb->prefix}edd_licenses WHERE status = %s",
            $status
        ));
    }

    private function get_license_details(int $license_id): ?array {
        global $wpdb;

        $result = $wpdb->get_row($wpdb->prepare(
            "SELECT l.id, l.license_key, l.user_id, l.status, l.expiration as expires_at, l.date_created as created_at,
                    u.display_name as user_name, u.user_email, l.download_id as tier
             FROM {$wpdb->prefix}edd_licenses l
             INNER JOIN {$wpdb->users} u ON l.user_id = u.ID
             WHERE l.id = %d",
            $license_id
        ));

        if (!$result) {
            return null;
        }

        return [
            'id' => $result->id,
            'license_key' => $result->license_key,
            'user_id' => $result->user_id,
            'user_name' => $result->user_name,
            'user_email' => $result->user_email,
            'status' => $result->status,
            'tier' => $result->tier,
            'tier_label' => $this->get_tier_label($result->tier),
            'expires_at' => $result->expires_at,
            'created_at' => $result->created_at,
            'activation_count' => $this->get_license_activation_count($license_id),
        ];
    }

    private function get_license_activation_count(int $license_id): int {
        global $wpdb;
        $table = $wpdb->prefix . 'edd_license_activations';
        // Verify table exists (EDD 3.x)
        if ( $wpdb->get_var( $wpdb->prepare( 'SHOW TABLES LIKE %s', $table ) ) !== $table ) {
            return 0;
        }
        return (int) $wpdb->get_var($wpdb->prepare(
            "SELECT COUNT(*) FROM {$table} WHERE license_id = %d",
            $license_id
        ));
    }

    private function get_license_stats(int $license_id): array {
        global $wpdb;

        $screenshot_table = $wpdb->prefix . 'uiai_screenshot_usage';
        $critique_table = $wpdb->prefix . 'uiai_critique_usage';
        $ui_reverse_table = $wpdb->prefix . 'uiai_ui_reverse_usage';

        $screenshots = $wpdb->get_var($wpdb->prepare(
            "SELECT COUNT(*) FROM $screenshot_table WHERE license_id = %d",
            $license_id
        )) ?? 0;

        $critiques = $wpdb->get_var($wpdb->prepare(
            "SELECT COUNT(*) FROM $critique_table WHERE license_id = %d",
            $license_id
        )) ?? 0;

        $ui_reverse = $wpdb->get_var($wpdb->prepare(
            "SELECT COUNT(*) FROM $ui_reverse_table WHERE license_id = %d",
            $license_id
        )) ?? 0;

        return [
            'screenshots' => (int) $screenshots,
            'critiques' => (int) $critiques,
            'ui_reverse' => (int) $ui_reverse,
        ];
    }

    private function table_exists(string $table_name): bool {
        global $wpdb;
        $result = $wpdb->get_var($wpdb->prepare("SHOW TABLES LIKE %s", $table_name));
        return $result === $table_name;
    }

    private function get_license_usage(int $license_id, int $limit = 10): array {
        global $wpdb;
        $usage = [];

        $screenshot_table = $wpdb->prefix . 'uiai_screenshot_usage';
        $critique_table = $wpdb->prefix . 'uiai_critique_usage';

        if ($this->table_exists($screenshot_table)) {
            $screenshots = $wpdb->get_results($wpdb->prepare(
                "SELECT 'Screenshot' as type, created_at, status FROM $screenshot_table WHERE license_id = %d ORDER BY created_at DESC LIMIT %d",
                [$license_id, $limit]
            ));

            foreach ($screenshots as $row) {
                $usage[] = (array) $row;
            }
        }

        if ($this->table_exists($critique_table)) {
            $critiques = $wpdb->get_results($wpdb->prepare(
                "SELECT 'Critique' as type, created_at, status FROM $critique_table WHERE license_id = %d ORDER BY created_at DESC LIMIT %d",
                [$license_id, $limit]
            ));

            foreach ($critiques as $row) {
                $usage[] = (array) $row;
            }
        }

        if (!empty($usage)) {
            usort($usage, function($a, $b) {
                return strtotime($b['created_at']) - strtotime($a['created_at']);
            });
            return array_slice($usage, 0, $limit);
        }

        return [];
    }

    private function get_tier_labels(): array {
        return [
            'free' => 'Free',
            'starter' => 'Starter',
            'developer' => 'Developer',
            'pro' => 'Pro',
            'agency' => 'Agency',
            'enterprise' => 'Enterprise',
        ];
    }

    private function get_tier_label($product_id): string {
        if (!is_numeric($product_id)) {
            return 'Unknown';
        }
        // Actual EDD download IDs
        $tier_map = [
            21  => 'free',       // WPUIAI Screenshots - Free
            22  => 'developer',  // WPUIAI Screenshots - Developer
            23  => 'pro',        // WPUIAI Screenshots - Pro
            24  => 'agency',     // WPUIAI Screenshots - Agency
            25  => 'enterprise', // WPUIAI Screenshots - Enterprise
            66  => 'starter',    // WPUIAI Starter
            16  => 'pro',        // WPUIAI Pro
            17  => 'agency',     // WPUIAI Agency
            452 => 'starter',    // WPUIAI Starter LTD
            453 => 'pro',        // WPUIAI Pro LTD
            454 => 'agency',     // WPUIAI Agency LTD
            1735 => 'evaluation',// Focusa Evaluation (EVAL)
            1736 => 'operator',  // Focusa Operator (Lifetime) — primary product
        ];
        return $tier_map[(int) $product_id] ?? 'Unknown';
    }

    private function get_tier_product_id(string $tier): int {
        // Map tier name → canonical EDD download ID
        $product_map = [
            'free'       => 21,
            'developer'  => 22,
            'starter'    => 66,
            'pro'        => 23,
            'agency'     => 24,
            'enterprise' => 25,
        ];
        return $product_map[$tier] ?? 0;
    }

    private function get_tier_limits(string $tier): array {
        $limits = [
            'free' => [
                'Screenshots per day' => 10,
                'Critiques per day' => 0,
                'UI Reverse ops per day' => 0,
                'Max resolution' => '1280x800',
                'Share expiry' => '1 hour',
                'Batch concurrency' => 1,
            ],
            'developer' => [
                'Screenshots per day' => 500,
                'Critiques per day' => 10,
                'UI Reverse ops per day' => 10,
                'Max resolution' => '1920x1080',
                'Share expiry' => '24 hours',
                'Batch concurrency' => 3,
            ],
            'pro' => [
                'Screenshots per day' => 2000,
                'Critiques per day' => 50,
                'UI Reverse ops per day' => 25,
                'Max resolution' => '1920x1080',
                'Share expiry' => '7 days',
                'Batch concurrency' => 5,
            ],
            'agency' => [
                'Screenshots per day' => 10000,
                'Critiques per day' => 200,
                'UI Reverse ops per day' => 100,
                'Max resolution' => '3840x2160',
                'Share expiry' => '7 days',
                'Batch concurrency' => 10,
            ],
            'enterprise' => [
                'Screenshots per day' => 'Unlimited',
                'Critiques per day' => 1000,
                'UI Reverse ops per day' => 500,
                'Max resolution' => 'Unlimited',
                'Share expiry' => '30 days',
                'Batch concurrency' => 20,
            ],
        ];
        return $limits[$tier] ?? [];
    }

    private function format_date(string $date): string {
        $timestamp = strtotime($date);
        return date('Y-m-d H:i:s', $timestamp);
    }

    private function days_until(string $date): int {
        $diff = strtotime($date) - time();
        return max(0, floor($diff / 86400));
    }
}
