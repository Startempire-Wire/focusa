<?php
/**
 * Admin Grant UI — Focusa • UIAI Licenses
 * Spec 173 B1-B2 — one-click grant + full license table with product/price/dates/remaining.
 * Menu: Focusa • UIAI Licenses (position 31, dashicons-admin-network)
 */
defined('ABSPATH') || exit;

class WPUIAI_AIC_Admin_License_Grant {
    private static $instance = null;
    public static function instance(): self {
        if (self::$instance === null) {
            self::$instance = new self();
            add_action('admin_menu', [self::$instance, 'register_menu']);
            add_action('admin_enqueue_scripts', [self::$instance, 'enqueue']);
            add_action('wp_ajax_wpuiai_grant_license', [self::$instance, 'ajax_grant']);
        }
        return self::$instance;
    }
    public function register_menu(): void {
        add_menu_page(
            'Focusa • UIAI Licenses',
            'Focusa • UIAI Licenses',
            'manage_options',
            'focusa-uiai-licenses',
            [$this, 'render_page'],
            'dashicons-admin-network',
            31
        );
    }
    public function enqueue(string $hook): void {
        if (strpos($hook, 'focusa-uiai-licenses') === false) return;
        $css = WPUIAI_AIC_PLUGIN_DIR . 'assets/css/licenses.css';
        $ver = file_exists($css) ? filemtime($css) : WPUIAI_AIC_VERSION;
        wp_enqueue_style('wpuiai-grant', WPUIAI_AIC_PLUGIN_URL . 'assets/css/licenses.css', [], $ver);
        wp_localize_script('jquery', 'wpuiaiGrant', [
            'ajaxurl' => admin_url('admin-ajax.php'),
            'nonce' => wp_create_nonce('wpuiai_grant_license'),
            'licensesNonce' => wp_create_nonce('wpuiai_aic_licenses'),
        ]);
    }

    // ── Grant + Table page ──────────────────────────
    public function render_page(): void {
        if (!current_user_can('manage_options')) { wp_die('Access denied'); }

        // POST fallback grant
        $notice = '';
        if ($_SERVER['REQUEST_METHOD'] === 'POST' && isset($_POST['wpuiai_grant_nonce']) && wp_verify_nonce($_POST['wpuiai_grant_nonce'], 'wpuiai_grant_license')) {
            $email = sanitize_email($_POST['grant_email'] ?? '');
            $product = sanitize_text_field($_POST['grant_product'] ?? '');
            $out = $this->do_grant($email, $product);
            if (!empty($out['ok'])) {
                $notice = '<div class="notice notice-success"><p>Granted <strong>' . esc_html($out['tier']) . '</strong> to ' . esc_html($email) . ' — <code>' . esc_html($out['license_key']) . '</code> (id ' . (int)$out['license_id'] . ')</p><p><button type="button" class="button" onclick="navigator.clipboard.writeText(\'' . esc_js($out['license_key']) . '\');this.textContent=\'Copied!\'">Copy key</button> Status: active.</p></div>';
            } else {
                $notice = '<div class="notice notice-error"><p>Grant failed: ' . esc_html($out['error'] ?? 'unknown') . '</p></div>';
            }
        }

        $search = isset($_GET['s']) ? sanitize_text_field($_GET['s']) : '';
        $status_filter = isset($_GET['status']) ? sanitize_text_field($_GET['status']) : '';
        $product_filter = isset($_GET['product']) ? sanitize_text_field($_GET['product']) : '';
        $page = isset($_GET['paged']) ? max(1, intval($_GET['paged'])) : 1;
        $per_page = 20;
        $offset = ($page - 1) * $per_page;
        $total = $this->get_total_licenses($search, $status_filter, $product_filter);
        $licenses = $this->get_licenses($offset, $per_page, $search, $status_filter, $product_filter);
        $all_products = $this->get_all_downloads();

        ?>
        <div class="wrap wpuiai-aic-wrap">
            <h1><span class="dashicons dashicons-admin-network"></span> Focusa &bull; UIAI Licenses</h1>
            <style>
            /* 11 cols: ID 62 + Product 160 + Key 165 + Customer 140 + Price 85 + Created 135 + Expires 120 + Seats 120 + Payment 155 + Status 85 + Actions 260 ≈ 1487 total — never squish, scroll instead */
            .wpuiai-table-wrap{overflow-x:auto;-webkit-overflow-scrolling:touch;border:1px solid #ccd0d4;border-radius:4px;background:#fff}
            .wpuiai-table-wrap table{margin:0 !important;border:none !important;min-width:1487px !important;table-layout:fixed !important;width:1487px !important}
            .wpuiai-table-wrap thead th{white-space:nowrap !important;font-size:12px !important;letter-spacing:.02em;text-transform:uppercase;color:#50575e;vertical-align:bottom;padding:10px 8px !important;line-height:1.3}
            .wpuiai-table-wrap tbody td{vertical-align:top !important;padding:10px 8px !important;line-height:1.45;word-wrap:break-word;overflow-wrap:break-word;hyphens:auto}
            .wpuiai-table-wrap tbody td:nth-child(1){width:62px;max-width:62px;white-space:nowrap}
            .wpuiai-table-wrap tbody td:nth-child(2){width:160px;max-width:160px}
            .wpuiai-table-wrap tbody td:nth-child(2) strong{display:block;white-space:normal;line-height:1.3}
            .wpuiai-table-wrap tbody td:nth-child(2) small{white-space:nowrap}
            .wpuiai-table-wrap tbody td:nth-child(3){width:165px;max-width:165px}
            .wpuiai-table-wrap tbody td:nth-child(3) code{display:block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;font-size:11px;padding:3px 5px;background:#f6f7f7;border:1px solid #dcdcde;border-radius:3px}
            .wpuiai-table-wrap tbody td:nth-child(4){width:140px;max-width:140px}
            .wpuiai-table-wrap tbody td:nth-child(4) small{word-break:break-all;white-space:normal;display:block;line-height:1.3}
            .wpuiai-table-wrap tbody td:nth-child(5){width:85px;max-width:85px;white-space:nowrap;text-align:center}
            .wpuiai-table-wrap tbody td:nth-child(6){width:135px;max-width:135px;white-space:nowrap;font-size:12px}
            .wpuiai-table-wrap tbody td:nth-child(6) small{display:block;white-space:nowrap}
            .wpuiai-table-wrap tbody td:nth-child(7){width:120px;max-width:120px;white-space:nowrap;font-size:12px}
            .wpuiai-table-wrap tbody td:nth-child(8){width:120px;max-width:120px}
            .wpuiai-table-wrap tbody td:nth-child(9){width:155px;max-width:155px}
            .wpuiai-table-wrap tbody td:nth-child(10){width:85px;max-width:85px;text-align:center;white-space:nowrap}
            .wpuiai-table-wrap tbody td:nth-child(11){width:260px;max-width:260px}
            .wpuiai-table-wrap tbody td:nth-child(11) div{flex-wrap:nowrap !important}
            .wpuiai-table-wrap tbody td:nth-child(11) .button{white-space:nowrap;font-size:11px;padding:0 8px;height:26px;line-height:24px}
            .wpuiai-aic-stats-cards{flex-wrap:wrap}
            .wpuiai-filter-bar{flex-wrap:wrap}
            /* Keep readable via scroll — never shrink table on smaller viewports */
            @media (max-width: 1440px){
                .wpuiai-table-wrap table{min-width:1487px !important;width:1487px !important}
            }
            @media (max-width: 1024px){
                .wpuiai-table-wrap table{min-width:1487px !important;width:1487px !important}
                .wpuiai-aic-stats-cards div{min-width:100px !important}
            }
            @media (max-width: 768px){
                .wpuiai-table-wrap{margin-left:-10px;margin-right:-10px;border-radius:0;border-left:none;border-right:none}
                .wpuiai-table-wrap table{min-width:1487px !important;width:1487px !important;font-size:13px}
                .wpuiai-filter-bar input[type=text]{min-width:160px !important;flex:1}
                .wpuiai-aic-stats-cards{gap:8px !important}
                .wpuiai-aic-stats-cards div{flex:1 1 90px;min-width:90px !important;padding:10px 8px !important}
            }
            @media (max-width: 375px){
                #wpuiai-grant-panel .form-table th{display:block;width:100% !important;padding-bottom:0 !important}
                #wpuiai-grant-panel .form-table td{display:block;width:100% !important;padding-top:4px !important}
                #wpuiai-grant-panel .form-table input[type=email],
                #wpuiai-grant-panel .form-table select{max-width:100% !important}
                #wpuiai-grant-panel .wpuiai-grant-plan label{display:block;margin-bottom:6px}
                .wpuiai-table-wrap{margin-left:-20px;margin-right:-20px}
                .wpuiai-table-wrap table{min-width:1487px !important;width:1487px !important;font-size:12px}
                .wpuiai-aic-stats-cards{flex-direction:row}
                .wpuiai-aic-stats-cards div{flex:1 1 46%;min-width:46% !important}
                .wpuiai-filter-bar{gap:6px !important}
                .wpuiai-filter-bar input[type=text],.wpuiai-filter-bar select{flex:1 1 100%;min-width:0 !important}
                .wpuiai-filter-bar .button{flex:1}
            }
            @media (max-width:782px){
                #wpuiai-grant-panel .form-table{margin:0}
            }
            </style>
            <p class="description">Grant any license instantly — no checkout. Table shows product, price, dates, and remaining activations. Immediately queryable via frontend lookup.</p>
            <?php echo $notice; ?>

            <h2 class="nav-tab-wrapper">
                <a href="#grant" class="nav-tab nav-tab-active" onclick="document.getElementById('wpuiai-grant-panel').style.display='block';return false;">Grant</a>
                <a href="<?php echo esc_url(admin_url('admin.php?page=wpuiai-ai-cloud-licenses')); ?>" class="nav-tab">Manage (Advanced)</a>
                <a href="<?php echo esc_url(admin_url('edit.php?post_type=download&page=edd-licenses')); ?>" class="nav-tab">EDD Licenses (Native)</a>
            </h2>

            <!-- GRANT PANEL -->
            <div id="wpuiai-grant-panel" style="margin-top:20px;max-width:760px;background:#fff;border:1px solid #ccd0d4;padding:20px;border-radius:8px;">
                <h3 style="margin-top:0;">Grant License — Focusa / UIAI Engine / Bundle</h3>
                <form method="post" id="wpuiai-grant-form">
                    <?php wp_nonce_field('wpuiai_grant_license', 'wpuiai_grant_nonce'); ?>
                    <div class="wpuiai-grant-plan" style="margin-top:12px;padding:10px;background:#f6f7f7;border:1px solid #dcdcde;">
                        <label><strong>Payment plan</strong> &nbsp;<input type="checkbox" id="wpuiai-plan-toggle" name="plan_enabled" value="1"> Enable installments</label>
                        <div id="wpuiai-plan-fields" style="display:none;margin-top:8px;">
                            <label>Total price (USD): <input type="number" step="0.01" name="plan_total" id="wpuiai-plan-total" style="width:120px"></label> &nbsp;
                            <label>Installments: <input type="number" name="plan_installments" id="wpuiai-plan-installments" value="3" min="2" max="36" style="width:70px"></label> &nbsp;
                            <label>Type: <select name="plan_type" id="wpuiai-plan-type"><option value="manual">Manual / invoice</option><option value="stripe">Stripe (auto-create)</option></select></label>
                            <p class="description">Stripe: creates a plan linked to Stripe — payments via Stripe PaymentIntents will auto-reconcile via webhook <code>/wp-json/wpuiai-ai-cloud/v1/stripe/payment-plan-webhook</code>. Add <code>metadata[license_id]</code> to the Intent. EDD Stripe is already live (<code>pk_live_...</code>). Reminders will email the customer daily/overdue.</p>
                        </div>
                    </div>
                    <script>document.getElementById('wpuiai-plan-toggle')?.addEventListener('change', e=>{document.getElementById('wpuiai-plan-fields').style.display=e.target.checked?'block':'none'; if(e.target.checked){var pr=document.querySelector('select[name="product"]'); var title=pr?.selectedOptions[0]?.text||""; var m=title.match(/\$(\d+[.,]?\d*)/); if(m) document.getElementById('wpuiai-plan-total').value=m[1].replace(',','');}});</script>
                    <table class="form-table">
                        <tr>
                            <th><label for="grant_email">Customer Email *</label></th>
                            <td>
                                <input type="email" id="grant_email" name="grant_email" required style="width:100%;max-width:420px;" placeholder="user@example.com" list="wpuiai-customer-emails">
                                <datalist id="wpuiai-customer-emails">
                                    <?php
                                    global $wpdb;
                                    $recent = $wpdb->get_col("SELECT email FROM {$wpdb->prefix}edd_customers ORDER BY id DESC LIMIT 30");
                                    foreach ((array)$recent as $e) { echo '<option value="' . esc_attr($e) . '"></option>'; }
                                    ?>
                                </datalist>
                                <p class="description">EDD customer or WP user email. If no WP account exists, one will be created automatically.</p>
                            </td>
                        </tr>
                        <tr>
                            <th><label for="grant_product">Product *</label></th>
                            <td>
                                <select id="grant_product" name="grant_product" required style="width:100%;max-width:420px;">
                                    <option value="1736" selected>Focusa Operator (Lifetime) — 1736 — $697 — Lifetime — limit 5 nodes</option>
                                    <option value="1736_uiai">UIAI Engine Operator (Lifetime) — 1736 + tier uiai_operator — limit 5</option>
                                    <option value="bundle_1736">Bundle Focusa+UIAI (Lifetime) — 1736 x2 grants</option>
                                    <option value="1735">Focusa Evaluation (30-day) — 1735 — $0 — Expires +30d</option>
                                    <option value="453">WPUIAI Pro Lifetime — 453</option>
                                    <option value="452">WPUIAI Starter Lifetime — 452</option>
                                </select>
                                <p class="description">Price and limit shown from EDD download record. Keys are <code>focusa_live_&lt;download&gt;_&lt;rand&gt;</code>.</p>
                            </td>
                        </tr>
                    </table>
                    <p>
                        <button type="submit" class="button button-primary button-large" id="grant-btn"><span class="dashicons dashicons-plus-alt" style="margin-top:4px;"></span> Grant License</button>
                        <span id="grant-spinner" style="display:none;margin-left:10px;">Granting...</span>
                    </p>
                    <div id="grant-result" style="margin-top:16px;"></div>
                </form>
            </div>

            <!-- DISTRIBUTION LIMITS (Operator 50 etc) -->
            <?php $dist_stats_grant = $this->get_distribution_stats(); $dist_map_grant = [1736=>'Focusa Operator (1736) — limit 50 default',1735=>'Focusa Evaluation (1735)',452=>'Starter LTD (452)',453=>'Pro LTD (453)']; ?>
            <div style="margin-top:18px;background:#fff;border:1px solid #ccd0d4;border-radius:8px;padding:14px;">
                <h3 style="margin:0 0 10px;"><span class="dashicons dashicons-chart-bar"></span> Distribution Limits <small style="font-weight:normal;color:#666;">— set/reset caps; distributed counts live from DB; deletes reclaim slots</small></h3>
                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:10px;">
                <?php foreach($dist_map_grant as $did=>$lbl): $st=$dist_stats_grant[$did] ?? ['distributed'=>0,'limit'=>null,'remaining'=>null]; $lim_disp=$st['limit']===null?'Unlimited':number_format($st['limit']); $rem_disp=$st['remaining']===null?'—':number_format($st['remaining']).' left'; $pct=($st['limit']&&$st['limit']>0)?min(100,max(0,($st['distributed']/$st['limit'])*100)):0; $bar=($st['remaining']!==null&&$st['remaining']<=0)?'#d63638':'#2271b1'; ?>
                    <div style="border:1px solid #dcdcde;border-radius:6px;padding:10px;background:#f9f9f9;">
                        <div style="font-weight:600;"><?php echo esc_html($lbl); ?> <small style="color:#666;">#<?php echo (int)$did; ?></small></div>
                        <div style="margin:6px 0;display:flex;gap:10px;flex-wrap:wrap;"><span><strong><?php echo number_format($st['distributed']); ?></strong> distributed</span><span>Limit: <strong><?php echo esc_html($lim_disp); ?></strong></span><span style="color:#555;"><?php echo esc_html($rem_disp); ?></span></div>
                        <div style="background:#eee;height:6px;border-radius:3px;"><div style="height:6px;background:<?php echo esc_attr($bar); ?>;width:<?php echo (int)$pct; ?>%;border-radius:3px;"></div></div>
                        <div style="display:flex;gap:6px;margin-top:8px;align-items:center;flex-wrap:wrap;">
                            <input type="number" min="0" placeholder="Limit (blank=unlimited)" class="dist-limit-input-grant" data-download="<?php echo (int)$did; ?>" style="width:150px;" value="<?php echo $st['limit']===null?'':(int)$st['limit']; ?>">
                            <button type="button" class="button button-small dist-set-limit-grant" data-download="<?php echo (int)$did; ?>">Set limit</button>
                            <button type="button" class="button button-small dist-reset-limit-grant" data-download="<?php echo (int)$did; ?>">Reset</button>
                        </div>
                    </div>
                <?php endforeach; ?>
                </div>
                <p class="description" style="margin:8px 0 0;">Enforced on grant (E_DISTRIBUTION_LIMIT). Remove test licenses via Delete in Quick Actions to reclaim.</p>
            </div>
            <!-- LICENSE TABLE -->
            <div style="margin-top:28px;">
                <h3>Licenses — Product / Price / Dates / Remaining <span style="font-weight:normal;color:#666;">(<?php echo number_format($total); ?> total)</span></h3>

                <div class="wpuiai-aic-stats-cards" style="display:flex;gap:12px;margin:12px 0;">
                    <?php foreach (['active'=>'Active','inactive'=>'Inactive','expired'=>'Expired','disabled'=>'Disabled'] as $k=>$lbl): ?>
                        <div style="background:#fff;border:1px solid #ccd0d4;padding:12px 16px;border-radius:6px;min-width:110px;text-align:center;">
                            <div style="font-size:20px;font-weight:700;"><?php echo number_format($this->get_count_by_status($k)); ?></div>
                            <div style="color:#666;font-size:12px;"><?php echo esc_html($lbl); ?></div>
                        </div>
                    <?php endforeach; ?>
                    <div style="background:#f6f7f7;border:1px solid #ccd0d4;padding:12px 16px;border-radius:6px;min-width:140px;text-align:center;">
                        <div style="font-size:20px;font-weight:700;"><?php echo number_format($total); ?></div>
                        <div style="color:#666;font-size:12px;">Total</div>
                    </div>
                </div>

                <form method="get" class="wpuiai-filter-bar" style="margin:12px 0;display:flex;gap:8px;align-items:center;">
                    <input type="hidden" name="page" value="focusa-uiai-licenses">
                    <input type="text" name="s" value="<?php echo esc_attr($search); ?>" placeholder="Search email, key, ID…" style="min-width:220px;">
                    <select name="status">
                        <option value="">All Status</option>
                        <?php foreach (['active'=>'Active','inactive'=>'Inactive','expired'=>'Expired','disabled'=>'Disabled'] as $k=>$l): ?>
                            <option value="<?php echo esc_attr($k); ?>" <?php selected($status_filter,$k); ?>><?php echo esc_html($l); ?></option>
                        <?php endforeach; ?>
                    </select>
                    <select name="product">
                        <option value="">All Products</option>
                        <?php foreach ($all_products as $pid=>$title): ?>
                            <option value="<?php echo esc_attr($pid); ?>" <?php selected($product_filter,(string)$pid); ?>><?php echo esc_html($title); ?> (<?php echo (int)$pid; ?>)</option>
                        <?php endforeach; ?>
                    </select>
                    <button class="button">Filter</button>
                    <a class="button" href="<?php echo esc_url(admin_url('admin.php?page=focusa-uiai-licenses')); ?>">Reset</a>
                </form>

                <div class="wpuiai-table-wrap"><table class="wp-list-table widefat fixed striped" style="margin-top:0;">
                    <thead>
                        <tr>
                            <th style="width:62px;">ID</th>
                            <th>Product</th>
                            <th>License Key</th>
                            <th>Customer</th>
                            <th>Price</th>
                            <th>Date Created</th>
                            <th>Expires</th>
                            <th style="width:120px;">Seats</th>
                            <th style="width:150px;">Payment</th>
                            <th>Status</th>
                            <th style="width:260px;">Quick Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                    <?php if (empty($licenses)): ?>
                        <tr><td colspan="11" style="text-align:center;padding:24px;">No licenses found.</td></tr>
                    <?php else: foreach ($licenses as $lic): ?>
                        <tr>
                            <td><strong>#<?php echo (int)$lic['id']; ?></strong></td>
                            <td>
                                <strong><?php echo esc_html($lic['product_title']); ?></strong>
                                <br><small style="color:#666;">download_id <?php echo (int)$lic['download_id']; ?><?php if($lic['price_id']) echo ' price_id '.(int)$lic['price_id']; ?></small>
                            </td>
                            <td>
                                <code class="license-key-<?php echo (int)$lic['id']; ?>" title="<?php echo esc_attr($lic['license_key']); ?>"><?php echo esc_html(substr($lic['license_key'],0,22)); ?>…</code>
                                <br><button type="button" class="button button-small" onclick="navigator.clipboard.writeText('<?php echo esc_js($lic['license_key']); ?>');this.textContent='Copied!';setTimeout(()=>this.textContent='Copy',1500)">Copy</button>
                                <button type="button" class="button button-small" onclick="(function(el){el.textContent=el.dataset.full; el.style.wordBreak='break-all';})(this)" data-full="<?php echo esc_attr($lic['license_key']); ?>">Reveal</button>
                            </td>
                            <td>
                                <?php if ($lic['user_id']): ?><a href="<?php echo esc_url(admin_url('user-edit.php?user_id='.$lic['user_id'])); ?>"><?php echo esc_html($lic['user_name']); ?></a><br><?php endif; ?>
                                <small><?php echo esc_html($lic['customer_email'] ?: $lic['user_email']); ?></small>
                                <?php if ($lic['customer_id']): ?><br><small style="color:#666;">customer #<?php echo (int)$lic['customer_id']; ?></small><?php endif; ?>
                            </td>
                            <td>
                                <strong><?php echo esc_html($lic['price_display']); ?></strong>
                                <?php if ($lic['price_id']): ?><br><small style="color:#666;">variant <?php echo (int)$lic['price_id']; ?></small><?php endif; ?>
                            </td>
                            <td><?php echo esc_html($lic['date_created']); ?><br><small style="color:#666;"><?php echo esc_html($lic['date_created_human']); ?></small></td>
                            <td>
                                <?php if ($lic['expires_display']==='Lifetime'): ?><span style="color:#0a7;"> Lifetime</span><?php else: ?>
                                    <?php echo esc_html($lic['expires_display']); ?>
                                    <br><small class="<?php echo $lic['days_left']!==null && $lic['days_left']<=7 ? 'expires-soon' : ''; ?>" style="color:<?php echo $lic['days_left']!==null && $lic['days_left']<=7 ? '#d63638' : '#666'; ?>;"><?php echo $lic['days_left']!==null ? esc_html($lic['days_left'].' days') : ''; ?></small>
                                <?php endif; ?>
                            </td>
                            <td>
                                <?php if ($lic['license_limit']===0 || $lic['license_limit']===null): ?>
                                    <span style="color:#666;">Unlimited</span><br><small><?php echo (int)$lic['activation_count']; ?> used</small>
                                <?php else: ?>
                                    <strong><?php echo (int)$lic['remaining']; ?> left</strong><br><small><?php echo (int)$lic['activation_count']; ?>/<?php echo (int)$lic['license_limit']; ?> seats</small>
                                    <div style="background:#eee;border-radius:3px;height:6px;margin-top:4px;max-width:100px;"><div style="height:6px;background:<?php echo $lic['remaining']>0 ? '#2271b1' : '#d63638'; ?>;width:<?php echo min(100, max(0, (1 - $lic['remaining'] / max(1,$lic['license_limit']))*100)); ?>%;border-radius:3px;"></div></div>
                                <?php endif; ?>
                            </td>
                            <td>
                                <?php if (!empty($lic['payment_plan'])): $pp=$lic['payment_plan']; $rem=(float)$pp['remaining_amount']; $paid=(float)$pp['paid_amount']; $tot=(float)$pp['total_price']; $pct=$tot>0 ? ($paid/$tot)*100 : 0; $badge = $pp['status']==='completed' ? '#d5e9d9' : ($pp['status']==='overdue' ? '#f5d6d6' : '#e7f3ff'); ?>
                                    <div style="background:<?php echo esc_attr($badge); ?>;padding:4px 6px;border-radius:4px;">
                                        <strong>$<?php echo number_format($rem,2); ?> due</strong><br>
                                        <small>$<?php echo number_format($paid,2); ?> / $<?php echo number_format($tot,2); ?> • <?php echo (int)$pp['installments_paid']; ?>/<?php echo (int)$pp['installments_total']; ?> • <span style="text-transform:capitalize;"><?php echo esc_html($pp['status']); ?></span><?php if($pp['plan_type']==='stripe') echo ' • Stripe'; ?></small>
                                        <div style="background:rgba(0,0,0,.08);border-radius:3px;height:6px;margin-top:4px;"><div style="height:6px;background:#2271b1;width:<?php echo min(100, max(0, $pct)); ?>%;border-radius:3px;"></div></div>
                                    </div>
                                <?php else: ?>
                                    <span style="color:#666;">—</span><br><small style="color:#888;">Full price • <a href="#" onclick="return false;" class="wpuiai-create-plan" data-id="<?php echo (int)$lic['id']; ?>" data-download="<?php echo (int)$lic['download_id']; ?>">Add plan</a></small>
                                <?php endif; ?>
                            </td>
                            <td><span class="wpuiai-aic-status status-<?php echo esc_attr($lic['status']); ?>" style="display:inline-block;padding:2px 8px;border-radius:10px;font-size:12px;background:<?php echo $lic['status']==='active' ? '#d5e9d9' : ($lic['status']==='expired' ? '#f5d6d6' : '#eee'); ?>;"><?php echo esc_html(ucfirst($lic['status'])); ?></span></td>
                            <td>
                                <div style="display:flex;gap:4px;flex-wrap:wrap;">
                                    <a href="<?php echo esc_url(admin_url('edit.php?post_type=download&page=edd-licenses&view=overview&id='.$lic['id'])); ?>" class="button button-small" target="_blank">View</a>
                                    <a href="<?php echo esc_url(admin_url('admin.php?page=wpuiai-ai-cloud-licenses')); ?>" class="button button-small">Manage</a>
                                    <?php if ($lic['status']==='active'): ?>
                                        <button type="button" class="button button-small wpuiai-quick-deactivate" data-id="<?php echo (int)$lic['id']; ?>">Deactivate</button>
                                    <?php elseif ($lic['status']==='inactive'): ?>
                                        <button type="button" class="button button-small button-primary wpuiai-quick-activate" data-id="<?php echo (int)$lic['id']; ?>">Activate</button>
                                    <?php endif; ?>
                                    <button type="button" class="button button-small wpuiai-quick-copy" data-key="<?php echo esc_attr($lic['license_key']); ?>">Copy Key</button>
                                    <button type="button" class="button button-small button-link-delete wpuiai-quick-delete" data-id="<?php echo (int)$lic['id']; ?>" style="color:#b32d2e;">Delete</button>
                                </div>
                            </td>
                        </tr>
                    <?php endforeach; endif; ?>
                    </tbody>
                </table></div>

                <?php if ($total > $per_page): ?>
                    <div class="tablenav bottom" style="margin-top:12px;">
                        <?php echo paginate_links([
                            'base' => admin_url('admin.php?page=focusa-uiai-licenses&s='.urlencode($search).'&status='.urlencode($status_filter).'&product='.urlencode($product_filter).'%_%'),
                            'format' => '&paged=%#%',
                            'current' => $page,
                            'total' => ceil($total/$per_page),
                            'prev_text' => '&laquo; Previous',
                            'next_text' => 'Next &raquo;',
                        ]); ?>
                    </div>
                <?php endif; ?>
            </div>
        </div>
        <script>
        jQuery(function($){
            $("#wpuiai-grant-form").on("submit", function(e){
                e.preventDefault();
                var email = $("#grant_email").val().trim();
                var product = $("#grant_product").val();
                if(!email) return alert("Email required");
                $("#grant-btn").prop("disabled", true);
                $("#grant-spinner").show();
                $("#grant-result").html("");
                var planEnabled = $("#wpuiai-plan-toggle").is(":checked") ? 1 : 0;
                var planTotal = $("#wpuiai-plan-total").val();
                var planInstallments = $("#wpuiai-plan-installments").val();
                var planType = $("#wpuiai-plan-type").val();
                $.post(wpuiaiGrant.ajaxurl, {
                    action: "wpuiai_grant_license",
                    _ajax_nonce: wpuiaiGrant.nonce,
                    email: email,
                    product: product,
                    plan_enabled: planEnabled,
                    plan_total: planTotal,
                    plan_installments: planInstallments,
                    plan_type: planType
                }, function(resp){
                    $("#grant-btn").prop("disabled", false);
                    $("#grant-spinner").hide();
                    if(resp && resp.success){
                        var d = resp.data;
                        $("#grant-result").html("<div class=\"notice notice-success\" style=\"padding:10px;\"><p><strong>Granted "+ (d.tier||"") +" to "+ email +"</strong></p><p>License <code>"+ (d.license_key||"") +"</code> (id "+ (d.license_id||"") +") — status active</p><p><button type=\"button\" class=\"button\" onclick=\"navigator.clipboard.writeText('"+ (d.license_key||"") +"');this.textContent='Copied!'\">Copy key</button> <a class=\"button\" href=\"<?php echo esc_url(admin_url('admin.php?page=focusa-uiai-licenses')); ?>\" onclick=\"location.reload();return false;\">Refresh table</a></p></div>");
                        setTimeout(function(){ location.reload(); }, 1200);
                    } else {
                        var msg = (resp && resp.data && resp.data.message) ? resp.data.message : (resp && resp.data ? JSON.stringify(resp.data) : "unknown error");
                        $("#grant-result").html("<div class=\"notice notice-error\" style=\"padding:10px;\"><p>Grant failed: "+ msg +"</p></div>");
                    }
                }).fail(function(xhr){
                    $("#grant-btn").prop("disabled", false);
                    $("#grant-spinner").hide();
                    $("#grant-result").html("<div class=\"notice notice-error\" style=\"padding:10px;\"><p>Request failed: "+ xhr.status +"</p></div>");
                });
            });
            $(".wpuiai-quick-activate, .wpuiai-quick-deactivate").on("click", function(){
                var id = $(this).data("id");
                var isActivate = $(this).hasClass("wpuiai-quick-activate");
                var action = isActivate ? "wpuiai_aic_activate_license" : "wpuiai_aic_deactivate_license";
                if(!confirm(isActivate ? "Activate license #"+id+"?" : "Deactivate license #"+id+"?")) return;
                var $btn=$(this); $btn.prop("disabled",true).text("...");
                $.post(wpuiaiGrant.ajaxurl, {action: action, license_id: id, nonce: wpuiaiGrant.licensesNonce}, function(resp){ // payment-plan-aware
                    if(resp && resp.success) location.reload();
                    else { alert((resp&&resp.data&&resp.data.message)||"Failed"); $btn.prop("disabled",false).text(isActivate?"Activate":"Deactivate"); }
                }).fail(function(){ alert("Request failed"); $btn.prop("disabled",false).text(isActivate?"Activate":"Deactivate"); });
            });
            $(".wpuiai-quick-copy").on("click", function(){
                var k=$(this).data("key"); navigator.clipboard.writeText(k); var $b=$(this); var t=$b.text(); $b.text("Copied!"); setTimeout(function(){ $b.text(t); },1500);
            });
            // Delete — cascade removes activations, payment plans, machines
            $(".wpuiai-quick-delete").on("click", function(){
                var id=$(this).data("id"); if(!confirm("Delete license #"+id+"? This removes all activations, payment plans and seats — it cannot be undone.")) return;
                var $btn=$(this); $btn.prop("disabled",true).text("Deleting...");
                $.post(wpuiaiGrant.ajaxurl, {action:"wpuiai_aic_delete_license", license_id:id, nonce:wpuiaiGrant.licensesNonce}, function(resp){
                    if(resp && resp.success) location.reload(); else { alert((resp&&resp.data&&resp.data.message)||"Failed to delete"); $btn.prop("disabled",false).text("Delete"); }
                }).fail(function(){ alert("Request failed"); $btn.prop("disabled",false).text("Delete"); });
            });
            // Distribution limits on this Grant page (reuse same endpoint as Licenses page)
            $(".dist-set-limit-grant").on("click", function(){
                var dl=$(this).data("download"); var input=$('.dist-limit-input-grant[data-download="'+dl+'"]'); var limit=input.val();
                if(limit!=='' && (isNaN(parseInt(limit,10))||parseInt(limit,10)<0)){ alert('Limit must be >=0 or blank'); return; }
                var $b=$(this); $b.prop('disabled',true).text('Saving...');
                $.post(wpuiaiGrant.ajaxurl, {action:"wpuiai_aic_set_distribution_limit", nonce:wpuiaiGrant.licensesNonce, download_id:dl, limit:limit}, function(resp){ if(resp&&resp.success) location.reload(); else { alert((resp&&resp.data&&resp.data.message)||"Failed"); $b.prop('disabled',false).text('Set limit'); }}).fail(function(){ alert("Request failed"); $b.prop('disabled',false).text('Set limit'); });
            });
            $(".dist-reset-limit-grant").on("click", function(){
                var dl=$(this).data("download"); if(!confirm('Reset limit for #'+dl+' to default?')) return; var $b=$(this); $b.prop('disabled',true).text('...');
                $.post(wpuiaiGrant.ajaxurl, {action:"wpuiai_aic_set_distribution_limit", nonce:wpuiaiGrant.licensesNonce, download_id:dl, limit:''}, function(resp){ if(resp&&resp.success) location.reload(); else { alert('Failed'); $b.prop('disabled',false).text('Reset'); }}).fail(function(){ alert("Request failed"); $b.prop('disabled',false).text('Reset'); });
            });
        });
        </script>
        <?php
    }

    // ── Data helpers ──────────────────────────────
    private function get_all_downloads(): array {
        $posts = get_posts(['post_type'=>'download','posts_per_page'=>50,'post_status'=>'any','orderby'=>'title','order'=>'ASC']);
        $out=[];
        foreach($posts as $p){ $out[$p->ID] = $p->post_title; }
        return $out;
    }
    private function get_price_display(int $download_id, ?int $price_id): string {
        $price = get_post_meta($download_id, 'edd_price', true);
        if ($price === '' || $price === null) {
            $var = get_post_meta($download_id, 'edd_variable_prices', true);
            if (is_array($var) && isset($var[0]['amount'])) $price = $var[0]['amount'];
        }
        // variable price id lookup
        if ($price_id) {
            $var = get_post_meta($download_id, 'edd_variable_prices', true);
            if (is_array($var)) {
                foreach($var as $v){ if((int)($v['index']??-1)===$price_id || (int)($v['price_id']??-1)===$price_id) { $price = $v['amount'] ?? $price; break; }}
            }
        }
        if ($price === '' || $price === null) return '—';
        if ((float)$price == 0) return 'Free';
        return '$' . number_format((float)$price, 2);
    }
    private function get_license_limit(int $download_id): ?int {
        $lim = get_post_meta($download_id, '_edd_sl_limit', true);
        if ($lim === '' || $lim === null) $lim = get_post_meta($download_id, '_edd_activation_limit', true);
        if ($lim === '' || $lim === null) return null;
        $i = (int)$lim; return $i; // 0 = unlimited
    }
    // ── Distribution caps (total licenses issued per product, e.g. 50 Operator) ──
    private function get_distribution_limits_raw(): array {
        $raw = get_option('wpuiai_license_distribution_limits', []);
        if (!is_array($raw)) $raw = [];
        $defaults = [1736 => 50];
        foreach ($defaults as $did=>$def) { if (!array_key_exists($did, $raw)) $raw[$did] = $def; }
        return $raw;
    }
    public function get_distribution_limits(): array { return $this->get_distribution_limits_raw(); }
    public function get_distribution_count(int $download_id): int {
        global $wpdb; return (int)$wpdb->get_var($wpdb->prepare("SELECT COUNT(*) FROM {$wpdb->prefix}edd_licenses WHERE download_id=%d", $download_id));
    }
    public function get_distribution_stats(): array {
        $raw_opt = get_option('wpuiai_license_distribution_limits', []);
        if (!is_array($raw_opt)) $raw_opt=[];
        $targets=[1736,1735,452,453]; $out=[];
        foreach($targets as $did){
            $cnt=$this->get_distribution_count($did);
            $lim = array_key_exists($did,$raw_opt) ? $raw_opt[$did] : (($did===1736)?50:null);
            if ($lim !== null) $lim = (int)$lim;
            $rem = ($lim===null)?null:max(0,$lim-$cnt);
            $out[$did]=['distributed'=>$cnt,'limit'=>$lim,'remaining'=>$rem];
        }
        return $out;
    }
    public function is_distribution_limit_reached(int $download_id): bool {
        $stats=$this->get_distribution_stats();
        if (!isset($stats[$download_id])) return false;
        $s=$stats[$download_id];
        if ($s['limit']===null) return false;
        return $s['distributed'] >= $s['limit'];
    }
    public function get_distribution_limit(int $download_id): ?int {
        $stats=$this->get_distribution_stats();
        return $stats[$download_id]['limit'] ?? null;
    }
    private function get_licenses(int $offset, int $per_page, string $search='', string $status='', string $product=''): array {
        global $wpdb;
        $activations_table = $wpdb->prefix . 'edd_license_activations';
        $activations_exist = $wpdb->get_var($wpdb->prepare('SHOW TABLES LIKE %s', $activations_table)) === $activations_table;
        $where=[]; $ph=[];
        if ($search !== '') {
            $where[] = "(l.license_key LIKE %s OR l.id = %s OR u.user_email LIKE %s OR u.display_name LIKE %s OR c.email LIKE %s)";
            $like = "%$search%"; $id_search = ctype_digit($search) ? (string)intval($search) : '-1';
            $ph[]=$like; $ph[]=$id_search; $ph[]=$like; $ph[]=$like; $ph[]=$like;
        }
        if ($status !== '') { $where[]="l.status = %s"; $ph[]=$status; }
        if ($product !== '' && ctype_digit($product)) { $where[]="l.download_id = %d"; $ph[]=(int)$product; }
        $where_sql = $where ? 'WHERE '.implode(' AND ', $where) : '';
        $sql = $wpdb->prepare(
            "SELECT l.id, l.license_key, l.status, l.download_id, l.price_id, l.date_created, l.expiration, l.customer_id, l.user_id, u.display_name as user_name, u.user_email, c.email as customer_email
             FROM {$wpdb->prefix}edd_licenses l
             LEFT JOIN {$wpdb->users} u ON l.user_id = u.ID
             LEFT JOIN {$wpdb->prefix}edd_customers c ON l.customer_id = c.id
             $where_sql
             ORDER BY l.date_created DESC
             LIMIT %d OFFSET %d",
            array_merge($ph, [$per_page, $offset])
        );
        $rows = $wpdb->get_results($sql);
        // Pre-fetch payment plans for these license ids
        $plan_map = [];
        if (class_exists('WPUIAI_AIC_License_Payment_Plan') && !empty($rows)) {
            $ids = array_map(fn($x)=>(int)$x->id, (array)$rows);
            $in = implode(',', array_map('intval', $ids)); // ints validated, safe for IN
            if ($in !== '') {
                $prows = $wpdb->get_results("SELECT * FROM {$wpdb->prefix}wpuiai_license_payment_plans WHERE license_id IN ($in)", ARRAY_A); // $in is intval-sanitized
                foreach ((array)$prows as $pr) { $plan_map[(int)$pr['license_id']] = $pr; }
            }
        }
        $out=[];
        foreach((array)$rows as $r){
            $activation_count = 0;
            if($activations_exist){
                $activation_count = (int)$wpdb->get_var($wpdb->prepare("SELECT COUNT(*) FROM {$activations_table} WHERE license_id = %d AND activated = 1", $r->id));
            }
            $limit = $this->get_license_limit((int)$r->download_id);
            $remaining = ($limit===0 || $limit===null) ? null : max(0, $limit - $activation_count);
            $price_display = $this->get_price_display((int)$r->download_id, $r->price_id ? (int)$r->price_id : null);
            $product_title = get_the_title((int)$r->download_id) ?: 'Unknown product';
            $expires_display='Lifetime'; $days_left=null;
            if($r->expiration !== null && $r->expiration !== '' && (int)$r->expiration !== 0){
                $ts=(int)$r->expiration; $expires_display=date('Y-m-d H:i', $ts); $days_left = max(0, (int)floor(($ts - time())/86400));
            }
            $date_created = $r->date_created ?: '';
            $human = $date_created ? date('Y-m-d H:i', strtotime($date_created)) : '—';
            // also try to get expiration from edd_licensemeta if expiration column is null? keep as is
            $plan = $plan_map[(int)$r->id] ?? null;
            $out[]=[
                'id'=>(int)$r->id,
                'license_key'=>$r->license_key,
                'status'=>$r->status,
                'download_id'=>(int)$r->download_id,
                'product_title'=>$product_title,
                'price_id'=>$r->price_id ? (int)$r->price_id : 0,
                'price_display'=>$price_display,
                'customer_id'=>$r->customer_id ? (int)$r->customer_id : 0,
                'customer_email'=>$r->customer_email ?: '',
                'user_id'=>$r->user_id ? (int)$r->user_id : 0,
                'user_name'=>$r->user_name ?: '—',
                'user_email'=>$r->user_email ?: '',
                'date_created'=>$human,
                'date_created_human'=>$date_created ? (human_time_diff(strtotime($date_created), time()).' ago') : '—',
                'expires_display'=>$expires_display,
                'days_left'=>$days_left,
                'activation_count'=>$activation_count,
                'license_limit'=>$limit,
                'remaining'=>$remaining,
                'payment_plan'=>$plan,
            ];
        }
        return $out;
    }
    private function get_total_licenses(string $search='', string $status='', string $product=''): int {
        global $wpdb;
        $where=[]; $ph=[];
        if ($search !== '') {
            $where[] = "(l.license_key LIKE %s OR l.id = %s OR u.user_email LIKE %s OR u.display_name LIKE %s OR c.email LIKE %s)";
            $like="%$search%"; $id_search=ctype_digit($search) ? (string)intval($search) : '-1';
            $ph[]=$like; $ph[]=$id_search; $ph[]=$like; $ph[]=$like; $ph[]=$like;
        }
        if ($status !== '') { $where[]="l.status = %s"; $ph[]=$status; }
        if ($product !== '' && ctype_digit($product)) { $where[]="l.download_id = %d"; $ph[]=(int)$product; }
        $where_sql=$where ? 'WHERE '.implode(' AND ',$where) : '';
        if($ph){
            $sql=$wpdb->prepare("SELECT COUNT(*) FROM {$wpdb->prefix}edd_licenses l LEFT JOIN {$wpdb->users} u ON l.user_id=u.ID LEFT JOIN {$wpdb->prefix}edd_customers c ON l.customer_id=c.id $where_sql", $ph);
        } else {
            $sql="SELECT COUNT(*) FROM {$wpdb->prefix}edd_licenses l LEFT JOIN {$wpdb->users} u ON l.user_id=u.ID LEFT JOIN {$wpdb->prefix}edd_customers c ON l.customer_id=c.id $where_sql";
        }
        return (int)$wpdb->get_var($sql);
    }
    private function get_count_by_status(string $status): int {
        global $wpdb;
        return (int)$wpdb->get_var($wpdb->prepare("SELECT COUNT(*) FROM {$wpdb->prefix}edd_licenses WHERE status = %s", $status));
    }
    // ── Grant action ──────────────────────────────
    private function do_grant(string $email, string $product): array {
        if (!is_email($email)) return ['ok'=>false,'error'=>'invalid_email'];
        // Spec 173 strict price_version gate — if caller supplies it, must match registry exactly else E_PRICE_MISMATCH
        $price_version = isset($_POST['price_version']) ? sanitize_text_field($_POST['price_version']) : (isset($_REQUEST['price_version']) ? sanitize_text_field($_REQUEST['price_version']) : '');
        if ($price_version !== '') {
            $expected = '';
            if ($product==='1736' || $product==='1736_uiai' || $product==='bundle_1736') $expected='focusa_operator_lifetime_v1.697.00.v1';
            else if ($product==='1735') $expected='focusa_evaluation_v1.0.00.v1';
            if ($expected!=='' && $price_version!==$expected) return ['ok'=>false,'error'=>'E_PRICE_MISMATCH: expected '.$expected.' got '.$price_version];
        }
        $download_id = 1736; $tier = 'operator'; $is_bundle = false;
        if ($product === '1735') { $download_id = 1735; $tier = 'evaluation'; }
        else if ($product === '453') { $download_id = 453; $tier = 'operator'; }
        else if ($product === '452') { $download_id = 452; $tier = 'operator'; }
        else if ($product === '1736_uiai') { $download_id = 1736; $tier = 'uiai_operator'; }
        else if ($product === 'bundle_1736') { $is_bundle = true; $download_id = 1736; $tier = 'operator'; }
        else if ($product === '1736') { $download_id = 1736; $tier = 'operator'; }
        else { $download_id = (int)$product; }
        $user = get_user_by('email', $email);
        if (!$user) {
            $login = sanitize_user(explode('@', $email)[0], true);
            if ($login === '') $login = 'user_' . wp_generate_password(6, false, false);
            $base = $login; $i=1;
            while (username_exists($login)) { $login = $base . $i; $i++; }
            $pass = wp_generate_password(20, true, true);
            $uid = wp_create_user($login, $pass, $email);
            if (is_wp_error($uid)) return ['ok'=>false,'error'=>'user_create_failed: ' . $uid->get_error_message()];
            $user = get_user_by('id', $uid);
            wp_update_user(['ID'=>$uid, 'display_name'=>explode('@',$email)[0], 'role'=>'subscriber']);
        }
        // Distribution cap check — e.g. 50 Operator limit. Bare grant must respect the global distribution cap.
        if ($this->is_distribution_limit_reached($download_id)) {
            $stats = $this->get_distribution_stats()[$download_id];
            return ['ok'=>false,'error'=>'E_DISTRIBUTION_LIMIT: limit '.($stats['limit']).' reached for download '.$download_id.' ('.$stats['distributed'].' distributed, 0 remaining). Delete test licenses or raise the limit via Licenses → Distribution Limits.'];
        }
        if ($is_bundle && $this->is_distribution_limit_reached($download_id)) {
            // bundle would issue 2, need at least 2 slots; simple check already covers 1, but double-check remaining
            $stats = $this->get_distribution_stats()[$download_id];
            if (($stats['remaining'] ?? 999) < 2) return ['ok'=>false,'error'=>'E_DISTRIBUTION_LIMIT: bundle needs 2 slots but only '.($stats['remaining'] ?? 0).' remaining for '.$download_id];
        }
        if (!class_exists('WPUIAI_AIC_Focusa_License_Production')) return ['ok'=>false,'error'=>'license_production_missing'];
        $res = WPUIAI_AIC_Focusa_License_Production::issue_license($email, $download_id, 0, $tier);
        if (!empty($res['error'])) return ['ok'=>false,'error'=>$res['error']];
        if ($is_bundle) {
            $res2 = WPUIAI_AIC_Focusa_License_Production::issue_license($email, $download_id, 0, 'uiai_operator');
            $res['bundle_second'] = $res2;
        }
        global $wpdb;
        $wpdb->update($wpdb->prefix . 'edd_licenses', ['status'=>'active'], ['id'=>(int)$res['license_id']], ['%s'], ['%d']);
        // If grant requested a payment plan (UI checkbox), create it now
        $plan_enabled = isset($_POST['plan_enabled']) ? (int)$_POST['plan_enabled'] : (isset($_REQUEST['plan_enabled']) ? (int)$_REQUEST['plan_enabled'] : 0);
        if ($plan_enabled && class_exists('WPUIAI_AIC_License_Payment_Plan')) {
            $plan_total = isset($_POST['plan_total']) ? (float)$_POST['plan_total'] : (isset($_POST['total_price']) ? (float)$_POST['total_price'] : 0);
            $plan_installments = isset($_POST['plan_installments']) ? max(2, (int)$_POST['plan_installments']) : (isset($_POST['installments']) ? max(2,(int)$_POST['installments']) : 3);
            $plan_type = isset($_POST['plan_type']) ? sanitize_text_field($_POST['plan_type']) : 'manual';
            if ($plan_total <= 0) {
                $price = get_post_meta((int)$download_id, 'edd_price', true);
                $plan_total = $price !== '' ? (float)$price : 0;
            }
            if ($plan_total > 0) {
                $pr = WPUIAI_AIC_License_Payment_Plan::create_plan((int)$res['license_id'], (int)$download_id, $plan_total, $plan_installments, $plan_type);
                $res['payment_plan'] = $pr;
            }
        }
        $res['ok'] = true;
        return $res;
    }
    public function ajax_grant(): void {
        check_ajax_referer('wpuiai_grant_license');
        if (!current_user_can('manage_options')) wp_send_json_error(['message'=>'Permission denied']);
        $email = sanitize_email($_POST['email'] ?? '');
        $product = sanitize_text_field($_POST['product'] ?? '');
        $out = $this->do_grant($email, $product);
        if (!empty($out['ok'])) wp_send_json_success($out);
        else wp_send_json_error(['message'=>$out['error'] ?? 'grant_failed', 'raw'=>$out]);
    }
}
WPUIAI_AIC_Admin_License_Grant::instance();
