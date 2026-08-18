<?php
/**
 * Settings Management
 *
 * @package WPUIAI_SSC
 */

defined('ABSPATH') || exit;

if (!function_exists('register_setting')) {
    function register_setting($option_group, $option_name, $args = []) {}
}
if (!function_exists('esc_url_raw')) {
    function esc_url_raw($url, $protocols = null) { return $url; }
}
if (!function_exists('wp_remote_get')) {
    function wp_remote_get($url, $args = []) {
        return ['body' => '{}', 'response' => ['code' => 200]];
    }
}
if (!function_exists('checked')) {
    function checked($value, $current = true, $echo = false) { return ''; }
}

class WPUIAI_AIC_Settings {

    private static $instance = null;

    public static function instance(): self {
        if (self::$instance === null) {
            self::$instance = new self();
        }
        return self::$instance;
    }

    public function __construct() {
        add_action('admin_init', [$this, 'register_settings']);
        add_action('admin_enqueue_scripts', [$this, 'enqueue_scripts']);
        add_action('wp_ajax_wpuiai_aic_save_settings', [$this, 'ajax_save_settings']);
        add_action('wp_ajax_wpuiai_aic_test_connection', [$this, 'ajax_test_connection']);
        add_action('rest_api_init', [$this, 'register_rest_routes']);

        // Dev mode AJAX handlers (A6-A11)
        add_action('wp_ajax_wpuiai_aic_toggle_dev_mode', [$this, 'ajax_toggle_dev_mode']);
        add_action('wp_ajax_wpuiai_aic_test_critique', [$this, 'ajax_test_critique']);
        add_action('wp_ajax_wpuiai_aic_dev_health', [$this, 'ajax_dev_mode_health']);
        add_action('wp_ajax_wpuiai_aic_rotate_secret', [$this, 'ajax_rotate_webhook_secret']);
        add_action('wp_ajax_wpuiai_aic_clear_credit_cache', [$this, 'ajax_clear_credit_cache']);
        add_action('wp_ajax_wpuiai_aic_force_sync', [$this, 'ajax_force_settings_sync']);
    }

    public function register_rest_routes(): void {
        foreach ($this->get_rest_namespaces() as $namespace) {
            register_rest_route($namespace, '/ai-settings', [
                'methods' => 'GET',
                'callback' => [$this, 'rest_get_ai_settings'],
                'permission_callback' => [$this, 'rest_check_engine_auth'],
            ]);

            register_rest_route($namespace, '/usage', [
                'methods' => 'POST',
                'callback' => [$this, 'rest_record_usage'],
                'permission_callback' => [$this, 'rest_check_engine_auth'],
            ]);

            register_rest_route($namespace, '/license/validate', [
                'methods' => 'POST',
                'callback' => [$this, 'rest_validate_license'],
                'permission_callback' => '__return_true',
            ]);

            register_rest_route($namespace, '/license/limits', [
                'methods' => 'GET',
                'callback' => [$this, 'rest_get_license_limits'],
                'permission_callback' => '__return_true',
            ]);

            register_rest_route($namespace, '/rate-limits', [
                'methods' => 'GET',
                'callback' => [$this, 'rest_get_rate_limits'],
                'permission_callback' => [$this, 'rest_check_engine_auth'],
            ]);

            register_rest_route($namespace, '/user-context', [
                'methods' => 'GET',
                'callback' => [$this, 'rest_get_user_context'],
                'permission_callback' => [$this, 'rest_check_engine_auth'],
            ]);

            register_rest_route($namespace, '/keys/validate', [
                'methods' => 'POST',
                'callback' => [$this, 'rest_validate_api_key'],
                'permission_callback' => [$this, 'rest_check_engine_auth'],
            ]);

            // Credit system endpoints
            register_rest_route($namespace, '/credits/balance', [
                'methods' => 'POST',
                'callback' => [$this, 'rest_credit_balance'],
                'permission_callback' => '__return_true',
            ]);

            register_rest_route($namespace, '/credits/deduct', [
                'methods' => 'POST',
                'callback' => [$this, 'rest_credit_deduct'],
                'permission_callback' => [$this, 'rest_check_engine_auth'],
            ]);

            register_rest_route($namespace, '/credits/history', [
                'methods' => 'POST',
                'callback' => [$this, 'rest_credit_history'],
                'permission_callback' => '__return_true',
            ]);
        }
    }

    public function rest_check_engine_auth(): bool {
        $secret = $_SERVER['HTTP_X_WEBHOOK_SECRET'] ?? '';
        $expected = get_option('wpuiai_aic_webhook_secret', '');
        return !empty($expected) && hash_equals($expected, $secret);
    }

    private function get_rest_namespaces(): array {
        $namespaces = ['wpuiai-ai-cloud/v1'];
        return apply_filters('wpuiai_aic_rest_namespaces', $namespaces);
    }

    public function rest_get_ai_settings(): WP_REST_Response {
        // Default provider/model: read from parent plugin settings first,
        // fall back to cloud admin settings, then static defaults.
        // No hardcoded fallbacks. Values come from WP admin settings only.
        $default_provider = get_option( 'wpuiai_aic_default_provider',
            get_option( 'uiai_ai_provider', '' )
        );
        $default_model = get_option( 'wpuiai_aic_default_model',
            get_option( 'uiai_ai_model', '' )
        );

        return new WP_REST_Response([
            'default_provider' => $default_provider,
            'default_model'    => $default_model,
            'anthropic' => [
                'key' => $this->get_ai_key('wpuiai_aic_anthropic_key', 'uiai_anthropic_api_key'),
                'model' => get_option('wpuiai_aic_anthropic_model', get_option('uiai_anthropic_model', '')),
            ],
            'openai' => [
                'key' => $this->get_ai_key('wpuiai_aic_openai_key', 'uiai_openai_api_key'),
                'model' => get_option('wpuiai_aic_openai_model', get_option('uiai_openai_model', '')),
            ],
            'openrouter' => [
                'key' => $this->get_ai_key('wpuiai_aic_openrouter_key', 'uiai_openrouter_key'),
                'model' => get_option('wpuiai_aic_openrouter_model', get_option('uiai_openrouter_model', '')),
            ],
            'fireworks' => [
                'key' => get_option('wpuiai_aic_fireworks_key', ''),
                'model' => get_option('wpuiai_aic_fireworks_model', ''),
            ],
            'kimi' => [
                'key' => $this->get_ai_key('wpuiai_aic_kimi_key', 'uiai_moonshot_key'),
                'model' => get_option('wpuiai_aic_kimi_model', get_option('uiai_moonshot_model', 'kimi-k2.5')),
            ],
            'minimax' => [
                'key' => $this->get_ai_key('wpuiai_aic_minimax_key', 'uiai_minimax_key'),
                'model' => get_option('wpuiai_aic_minimax_model', get_option('uiai_minimax_model', 'MiniMax-M2.5')),
            ],
            'qwen' => [
                'key' => get_option('wpuiai_aic_qwen_key', ''),
                'model' => get_option('wpuiai_aic_qwen_model', ''),
            ],
        ]);
    }

    private function get_ai_key(string $admin_option, string $parent_option): string {
        $key = get_option($admin_option, '');
        if (!empty($key)) {
            return $key;
        }
        $canonical_option = $parent_option;
        if (substr($parent_option, -8) === '_api_key') {
            $canonical_option = substr($parent_option, 0, -8) . '_key';
        }
        if ($canonical_option !== $parent_option) {
            $canonical_value = get_option($canonical_option, '');
            if (!empty($canonical_value)) {
                return $canonical_value;
            }
        }
        return get_option($parent_option, '');
    }

    public function rest_record_usage(WP_REST_Request $request): WP_REST_Response {
        global $wpdb;

        $type = $request->get_param('type');
        $api_key = $request->get_param('api_key');
        $provider = $request->get_param('provider');
        $model = $request->get_param('model');
        $tokens = intval($request->get_param('tokens') ?? 0);
        $cost = floatval($request->get_param('cost') ?? 0);
        $image_url = $request->get_param('image_url');
        $built_image_url = $request->get_param('built_image_url');
        $reference_image_url = $request->get_param('reference_image_url');
        $operation = $request->get_param('operation');
        $license_id = intval($request->get_param('license_id') ?? 0);
        $client_id = sanitize_text_field($request->get_param('client_id') ?? '');

        if ($type === 'critique') {
            $insert_data = [
                'api_key' => substr($api_key, 0, 64),
                'provider' => $provider,
                'model' => $model,
                'tokens_used' => $tokens,
                'cost' => $cost,
                'image_url' => substr($image_url ?? '', 0, 512),
                'created_at' => current_time('mysql'),
            ];
            if ($license_id > 0) {
                $insert_data['license_id'] = $license_id;
            }
            if (!empty($client_id)) {
                $insert_data['client_id'] = $client_id;
            }
            $wpdb->insert($wpdb->prefix . 'uiai_critique_usage', $insert_data);
        } elseif ($type === 'ui-reverse') {
            $insert_data = [
                'api_key' => substr($api_key, 0, 64),
                'provider' => $provider,
                'model' => $model,
                'operation' => $operation ? sanitize_text_field($operation) : '',
                'tokens_used' => $tokens,
                'cost' => $cost,
                'image_url' => substr($image_url ?? '', 0, 512),
                'created_at' => current_time('mysql'),
            ];
            if ($license_id > 0) {
                $insert_data['license_id'] = $license_id;
            }
            if (!empty($client_id)) {
                $insert_data['client_id'] = $client_id;
            }
            $extracted = $request->get_param('extracted_elements');
            $insert_data['extracted_elements'] = is_string($extracted) ? $extracted : json_encode($extracted);
            $wpdb->insert($wpdb->prefix . 'uiai_ui_reverse_usage', $insert_data);
        } elseif ($type === 'section_detect') {
            $insert_data = [
                'api_key' => substr($api_key, 0, 64),
                'provider' => $provider,
                'model' => $model,
                'tokens_used' => $tokens,
                'cost' => $cost,
                'image_url' => substr($image_url ?? '', 0, 512),
                'created_at' => current_time('mysql'),
            ];
            if ($license_id > 0) {
                $insert_data['license_id'] = $license_id;
            }
            if (!empty($client_id)) {
                $insert_data['client_id'] = $client_id;
            }
            $wpdb->insert($wpdb->prefix . 'uiai_section_detect_usage', $insert_data);
        } elseif ($type === 'layout_compare') {
            $insert_data = [
                'api_key' => substr($api_key, 0, 64),
                'provider' => $provider,
                'model' => $model,
                'tokens_used' => $tokens,
                'cost' => $cost,
                'built_image_url' => substr(($built_image_url ?: $image_url) ?? '', 0, 512),
                'reference_image_url' => substr($reference_image_url ?? '', 0, 512),
                'created_at' => current_time('mysql'),
            ];
            if ($license_id > 0) {
                $insert_data['license_id'] = $license_id;
            }
            if (!empty($client_id)) {
                $insert_data['client_id'] = $client_id;
            }
            $wpdb->insert($wpdb->prefix . 'uiai_layout_compare_usage', $insert_data);
        } elseif ($type === 'style_enhance') {
            $insert_data = [
                'api_key' => substr($api_key, 0, 64),
                'provider' => $provider,
                'model' => $model,
                'tokens_used' => $tokens,
                'cost' => $cost,
                'built_image_url' => substr(($built_image_url ?: $image_url) ?? '', 0, 512),
                'reference_image_url' => substr($reference_image_url ?? '', 0, 512),
                'created_at' => current_time('mysql'),
            ];
            if ($license_id > 0) {
                $insert_data['license_id'] = $license_id;
            }
            if (!empty($client_id)) {
                $insert_data['client_id'] = $client_id;
            }
            $wpdb->insert($wpdb->prefix . 'uiai_style_enhance_usage', $insert_data);
        }

        return new WP_REST_Response(['success' => true]);
    }

    /**
     * Check if dev mode is active on the parent plugin.
     *
     * Mirrors WPUIAI_License::is_dev_mode(). Checks constant first,
     * then falls back to the uiai_dev_mode WP option.
     */
    private function is_dev_mode(): bool {
        if ( defined( 'WPUIAI_DEV_MODE' ) && WPUIAI_DEV_MODE ) {
            return true;
        }
        return (bool) get_option( 'uiai_dev_mode', false );
    }

    /**
     * Simple IP-based rate limiter using transients.
     * Returns WP_REST_Response on limit exceeded, null otherwise.
     */
    private function check_rate_limit( string $action, int $max_per_minute = 30 ): ?WP_REST_Response {
        $ip   = $_SERVER['REMOTE_ADDR'] ?? 'unknown';
        $key  = 'wpuiai_rl_' . md5( $action . $ip );
        $hits = (int) get_transient( $key );

        if ( $hits >= $max_per_minute ) {
            return new WP_REST_Response( [
                'error'       => 'rate_limited',
                'message'     => 'Too many requests. Try again in 60 seconds.',
                'retry_after' => 60,
            ], 429 );
        }

        set_transient( $key, $hits + 1, 60 );
        return null;
    }

    public function rest_validate_license(WP_REST_Request $request): WP_REST_Response {
        // Rate limit: 30 validation attempts per minute per IP
        $limited = $this->check_rate_limit( 'license_validate', 30 );
        if ( $limited ) {
            return $limited;
        }

        // Dev mode: return synthetic enterprise tier.
        // This is called by the Go engine during validateLicense() — when the parent
        // plugin sends X-Webhook-Secret the Go engine authenticates directly as "internal"
        // and never calls this endpoint. But external dev clients may still hit it.
        if ( $this->is_dev_mode() ) {
            return new WP_REST_Response([
                'valid'      => true,
                'license_id' => 0,
                'tier'       => 'enterprise',
                'status'     => 'dev_mode',
                'limits'     => $this->get_tier_limits('enterprise'),
                'credits'    => [
                    'balance'  => 999999,
                    'granted'  => 999999,
                    'used'     => 0,
                    'expired'  => 0,
                ],
            ], 200);
        }

        $license_key = $request->get_param('license_key') ?: $request->get_header('X-License-Key');

        if (empty($license_key)) {
            return new WP_REST_Response(['valid' => false, 'error' => 'License key required'], 400);
        }

        global $wpdb;
        $license_table = $wpdb->prefix . 'edd_licenses';
        $license = $wpdb->get_row($wpdb->prepare(
            "SELECT * FROM {$license_table} WHERE license_key = %s",
            $license_key
        ));

        if (!$license) {
            return new WP_REST_Response(['valid' => false, 'error' => 'License not found'], 404);
        }

        $status = $license->status;
        if ($status !== 'active') {
            return new WP_REST_Response([
                'valid' => false,
                'error' => "License is {$status}",
                'status' => $status,
            ], 401);
        }

        // V3 — Per-machine seat enforcement (Focusa license production).
        // Reads X-Machine-Id from the request, applies the seat cap. If
        // exhausted we return status=revoked so the focusa daemon's
        // refresh/watch loop reacts.
        $machine_id = trim((string) $request->get_header('X-Machine-Id'));
        if ($machine_id !== '' && class_exists('WPUIAI_AIC_Focusa_License_Production')) {
            $gate = WPUIAI_AIC_Focusa_License_Production::machine_seat_check(
                (int) $license->id,
                $machine_id,
                'validate'
            );
            if (empty($gate['allowed'])) {
                $reason = $gate['reason'] ?? 'unknown';
                $cap = isset($gate['cap']) ? (int) $gate['cap'] : 0;
                $active = isset($gate['active']) ? (int) $gate['active'] : 0;
                return new WP_REST_Response([
                    'valid' => false,
                    'error' => "seat_cap_reached: machine not enrolled (cap={$cap}, active={$active})",
                    'status' => 'revoked',
                    'reason' => $reason,
                    'cap' => $cap,
                    'active' => $active,
                ], 403);
            }
        }

        // EDD SL stores expiration as Unix timestamp (bigint), not datetime string
        $expiration_ts = is_numeric($license->expiration) ? (int) $license->expiration : strtotime($license->expiration);
        if (!empty($license->expiration) && $expiration_ts > 0 && $expiration_ts < time()) {
            return new WP_REST_Response([
                'valid' => false,
                'error' => 'License has expired',
                'expired' => true,
            ], 401);
        }

        $tier = $this->resolve_tier_for_license( $license );
        $limits = $this->get_tier_limits($tier);
        $usage = $this->get_daily_usage_counts((int) $license->id, (int) $license->user_id);

        // Include credit balance
        require_once __DIR__ . '/class-credit-service.php';
        $credits = WPUIAI_Credit_Service::instance();
        $credit_info = $credits->get_balance_info( (int) $license->id );

        return new WP_REST_Response([
            'valid' => true,
            'license_id' => $license->id,
            'tier' => $tier,
            'status' => $status,
            'limits' => $limits,
            'credits' => [
                'balance'  => $credit_info['balance'],
                'granted'  => $credit_info['credits_granted'],
                'used'     => $credit_info['credits_used'],
                'costs'    => WPUIAI_Credit_Service::COSTS,
            ],
            'usage' => [
                'screenshot_remaining' => $this->get_remaining_limit($limits['screenshots'] ?? 0, $usage['screenshot_used']),
                'screenshot_used' => $usage['screenshot_used'],
                'critique_remaining' => $this->get_remaining_limit($limits['critiques'] ?? 0, $usage['critique_used']),
                'critique_used' => $usage['critique_used'],
                'ui_reverse_remaining' => $this->get_remaining_limit($limits['ui_reverse'] ?? 0, $usage['ui_reverse_used']),
                'ui_reverse_used' => $usage['ui_reverse_used'],
                'copilot_remaining' => $this->get_remaining_limit($limits['copilot'] ?? 0, $usage['copilot_used']),
                'copilot_used' => $usage['copilot_used'],
            ],
        ]);
    }

    public function rest_validate_api_key(WP_REST_Request $request): WP_REST_Response {
        // Rate limit: 20 API key validations per minute per IP
        $limited = $this->check_rate_limit( 'api_key_validate', 20 );
        if ( $limited ) {
            return $limited;
        }

        $api_key = $request->get_param('api_key') ?: $request->get_header('X-API-Key');

        if (empty($api_key)) {
            return new WP_REST_Response(['valid' => false, 'error' => 'API key required'], 400);
        }

        global $wpdb;
        $table = $wpdb->prefix . 'uiai_client_keys';
        $key = $wpdb->get_row($wpdb->prepare(
            "SELECT * FROM {$table} WHERE (client_secret = %s OR client_id = %s) AND status = 'active' LIMIT 1",
            $api_key,
            $api_key
        ), ARRAY_A);

        if (!$key) {
            $rows = $wpdb->get_results(
                "SELECT * FROM {$table} WHERE status = 'active'",
                ARRAY_A
            );
            foreach ($rows as $row) {
                if (!empty($row['client_secret']) && password_verify($api_key, $row['client_secret'])) {
                    $key = $row;
                    break;
                }
            }
        }

        if (!$key) {
            return new WP_REST_Response(['valid' => false, 'error' => 'Invalid API key'], 401);
        }

        $license_id = intval($key['license_id'] ?? 0);
        $tier = null;
        $limits = null;
        $usage = null;

        if ($license_id > 0) {
            $license_table = $wpdb->prefix . 'edd_licenses';
            $license = $wpdb->get_row($wpdb->prepare(
                "SELECT * FROM {$license_table} WHERE id = %d",
                $license_id
            ));

            if ($license) {
                $tier = $this->resolve_tier_for_license( $license );
                $limits = $this->get_tier_limits($tier);
                $usage = $this->get_daily_usage_counts($license_id, (int) $license->user_id);
            }
        }

        return new WP_REST_Response([
            'valid' => true,
            'client_id' => $key['client_id'],
            'license_id' => $license_id,
            'client_type' => $key['client_type'],
            'rate_limit_hourly' => $key['rate_limit_hourly'],
            'rate_limit_daily' => $key['rate_limit_daily'],
            'status' => $key['status'],
            'tier' => $tier,
            'limits' => $limits,
            'usage' => $usage ? [
                'critique_remaining' => $this->get_remaining_limit($limits['critiques'] ?? 0, $usage['critique_used']),
                'critique_used' => $usage['critique_used'],
                'ui_reverse_remaining' => $this->get_remaining_limit($limits['ui_reverse'] ?? 0, $usage['ui_reverse_used']),
                'ui_reverse_used' => $usage['ui_reverse_used'],
                'copilot_remaining' => $this->get_remaining_limit($limits['copilot'] ?? 0, $usage['copilot_used']),
                'copilot_used' => $usage['copilot_used'],
            ] : null,
        ]);
    }

    public function rest_get_license_limits(WP_REST_Request $request): WP_REST_Response {
        $license_key = $request->get_param('license_key') ?: $request->get_header('X-License-Key');

        if (empty($license_key)) {
            return new WP_REST_Response(['error' => 'License key required'], 400);
        }

        global $wpdb;
        $license_table = $wpdb->prefix . 'edd_licenses';
        $license = $wpdb->get_row($wpdb->prepare(
            "SELECT * FROM {$license_table} WHERE license_key = %s",
            $license_key
        ));

        if (!$license) {
            return new WP_REST_Response(['error' => 'License not found'], 404);
        }

        $tier = $this->resolve_tier_for_license( $license );
        $limits = $this->get_tier_limits($tier);

        return new WP_REST_Response([
            'tier' => $tier,
            'limits' => $limits,
        ]);
    }

    public function rest_get_rate_limits(): WP_REST_Response {
        return new WP_REST_Response([
            'global' => [
                'per_hour' => intval(get_option('wpuiai_aic_rate_limit_per_hour', 100)),
                'per_day' => intval(get_option('wpuiai_aic_rate_limit_per_day', 1000)),
            ],
        ]);
    }

    public function rest_get_user_context(WP_REST_Request $request): WP_REST_Response {
        $license_key = $request->get_param('license_key') ?: $request->get_header('X-License-Key');
        $user_id = intval($request->get_param('user_id') ?? 0);

        if (empty($license_key) && $user_id <= 0) {
            return new WP_REST_Response(['error' => 'license_key or user_id required'], 400);
        }

        global $wpdb;

        $context = [
            'user_id' => $user_id,
            'site_url' => get_site_url(),
            'license' => null,
            'preferences' => [],
            'ai_settings' => [],
        ];

        if (!empty($license_key)) {
            $license_table = $wpdb->prefix . 'uiai_licenses';
            $license = $wpdb->get_row($wpdb->prepare(
                "SELECT * FROM {$license_table} WHERE license_key = %s",
                $license_key
            ));

            if ($license) {
                $context['license'] = [
                    'id' => $license->id,
                    'tier' => $license->tier,
                    'status' => $license->status,
                ];

                $limits = $this->get_tier_limits($license->tier);
                $context['license']['limits'] = $limits;

                $context['user_id'] = $license->user_id ?: $user_id;
            }
        }

        if ($user_id > 0) {
            $user = get_userdata($user_id);
            if ($user) {
                $context['user'] = [
                    'id' => $user->ID,
                    'login' => $user->user_login,
                    'email' => $user->user_email,
                    'display_name' => $user->display_name,
                ];

                $context['preferences'] = [
                    'default_provider' => get_user_meta($user_id, 'wpuiai_default_provider', true) ?: get_option('uiai_ai_provider', ''),
                    'default_model' => get_user_meta($user_id, 'wpuiai_default_model', true) ?: get_option('uiai_ai_model', ''),
                    'theme' => get_user_meta($user_id, 'wpuiai_theme', true) ?: 'light',
                ];
            }
        }

        $context['ai_settings'] = [
            'anthropic' => [
                'key' => $this->get_ai_key('wpuiai_aic_anthropic_key', 'uiai_anthropic_api_key'),
                'model' => get_option('wpuiai_aic_anthropic_model', get_option('uiai_anthropic_model', '')),
            ],
            'openrouter' => [
                'key' => $this->get_ai_key('wpuiai_aic_openrouter_key', 'uiai_openrouter_key'),
                'model' => get_option('wpuiai_aic_openrouter_model', get_option('uiai_openrouter_model', '')),
            ],
        ];

        return new WP_REST_Response($context);
    }

    // ── Credit System Endpoints ──────────────────────────────────────

    /**
     * GET credit balance for a license.
     * Public endpoint — requires license_key.
     */
    public function rest_credit_balance( WP_REST_Request $request ): WP_REST_Response {
        // Dev mode: unlimited credits
        if ( $this->is_dev_mode() ) {
            require_once __DIR__ . '/class-credit-service.php';
            return new WP_REST_Response( [
                'balance'  => 999999,
                'granted'  => 999999,
                'used'     => 0,
                'expired'  => 0,
                'costs'    => WPUIAI_Credit_Service::COSTS,
            ] );
        }

        $license_key = $request->get_param( 'license_key' ) ?: $request->get_header( 'X-License-Key' );
        if ( empty( $license_key ) ) {
            return new WP_REST_Response( [ 'error' => 'License key required' ], 400 );
        }

        global $wpdb;
        $license = $wpdb->get_row( $wpdb->prepare(
            "SELECT id FROM {$wpdb->prefix}edd_licenses WHERE license_key = %s AND status = 'active'",
            $license_key
        ) );

        if ( ! $license ) {
            return new WP_REST_Response( [ 'error' => 'License not found or inactive' ], 404 );
        }

        require_once __DIR__ . '/class-credit-service.php';
        $credits = WPUIAI_Credit_Service::instance();
        $info    = $credits->get_balance_info( (int) $license->id );
        $info['costs'] = WPUIAI_Credit_Service::COSTS;

        return new WP_REST_Response( $info );
    }

    /**
     * Deduct credits for an operation.
     * Bun server only — requires webhook secret.
     */
    public function rest_credit_deduct( WP_REST_Request $request ): WP_REST_Response {
        $license_id = (int) $request->get_param( 'license_id' );
        $operation  = sanitize_text_field( $request->get_param( 'operation' ) ?? '' );
        $reference  = sanitize_text_field( $request->get_param( 'reference' ) ?? '' );

        if ( ! $license_id || ! $operation ) {
            return new WP_REST_Response( [ 'error' => 'license_id and operation required' ], 400 );
        }

        require_once __DIR__ . '/class-credit-service.php';
        $credits = WPUIAI_Credit_Service::instance();
        $result  = $credits->deduct( $license_id, $operation, $reference );

        $status = $result['success'] ? 200 : 402; // 402 Payment Required
        return new WP_REST_Response( $result, $status );
    }

    /**
     * Get credit history for a license.
     * Public endpoint — requires license_key.
     */
    public function rest_credit_history( WP_REST_Request $request ): WP_REST_Response {
        $license_key = $request->get_param( 'license_key' ) ?: $request->get_header( 'X-License-Key' );
        if ( empty( $license_key ) ) {
            return new WP_REST_Response( [ 'error' => 'License key required' ], 400 );
        }

        global $wpdb;
        $license = $wpdb->get_row( $wpdb->prepare(
            "SELECT id FROM {$wpdb->prefix}edd_licenses WHERE license_key = %s AND status = 'active'",
            $license_key
        ) );

        if ( ! $license ) {
            return new WP_REST_Response( [ 'error' => 'License not found or inactive' ], 404 );
        }

        $limit = min( (int) ( $request->get_param( 'limit' ) ?? 50 ), 100 );

        require_once __DIR__ . '/class-credit-service.php';
        $credits = WPUIAI_Credit_Service::instance();
        $history = $credits->get_history( (int) $license->id, $limit );

        return new WP_REST_Response( [
            'license_id' => (int) $license->id,
            'entries'     => $history,
        ] );
    }

    private function get_tier_from_product(int $product_id): string {
        // Map EDD download IDs to tier names.
        // Screenshot-tier products (21-25) are the canonical set.
        // Legacy products (16=Pro, 17=Agency) also resolve.
        $product_tier_map = [
            21  => 'free',        // WPUIAI Screenshots - Free
            22  => 'developer',   // WPUIAI Screenshots - Developer
            23  => 'pro',         // WPUIAI Screenshots - Pro
            24  => 'agency',      // WPUIAI Screenshots - Agency
            25  => 'enterprise',  // WPUIAI Screenshots - Enterprise
            16  => 'pro',         // WPUIAI Pro ($99/yr)
            17  => 'agency',      // WPUIAI Agency ($299/yr)
            66  => 'starter',     // WPUIAI Starter ($29/mo)
            1736 => 'enterprise', // Focusa Operator (Lifetime) — maps to UIAI enterprise limits
            1735 => 'free',       // Focusa Evaluation
        ];

        return $product_tier_map[ $product_id ] ?? 'free';
    }

    /**
     * Resolve tier for an EDD license using a two-level fallback:
     *   1. wp_edd_licenses.download_id → product map  (EDD 3.x: always available)
     *   2. wp_uiai_licenses.tier  (direct assignment, no product mapping)
     *
     * Note: EDD 3.x stores download_id directly on edd_licenses.
     * The old edd_software_licenses table (EDD SL 2.x) no longer exists.
     *
     * @param object $license  Row from wp_edd_licenses.
     * @return string  Tier slug.
     */
    private function resolve_tier_for_license( object $license ): string {
        global $wpdb;

        // EDD 3.x: download_id is directly on edd_licenses row
        $product_id = ! empty( $license->download_id ) ? (int) $license->download_id : 0;

        $tier = $this->get_tier_from_product( $product_id );

        // Level 3: wp_uiai_licenses.tier for direct tier assignment
        // Falls through when product ID didn't map to a known tier
        if ( $tier === 'free' ) {
            $custom_table = $wpdb->prefix . 'uiai_licenses';
            if ( $wpdb->get_var( $wpdb->prepare( 'SHOW TABLES LIKE %s', $custom_table ) ) === $custom_table ) {
                $direct_tier = $wpdb->get_var( $wpdb->prepare(
                    "SELECT tier FROM {$custom_table} WHERE license_key = %s AND status = 'active' LIMIT 1",
                    $license->license_key
                ) );
                if ( $direct_tier && $direct_tier !== 'free' ) {
                    $tier = $direct_tier;
                }
            }
        }

        return $tier;
    }

    private function get_tier_limits(string $tier): array {
        $settings = $this->get_settings();

        $tiers = [
            'free' => [
                'screenshots' => intval($settings['tier_free_screenshots'] ?? 10),
                'critiques' => intval($settings['tier_free_critiques'] ?? 0),
                'ui_reverse' => intval($settings['tier_free_ui_reverse'] ?? 0),
                'copilot' => intval($settings['tier_free_copilot'] ?? 0),
            ],
            'developer' => [
                'screenshots' => intval($settings['tier_dev_screenshots'] ?? 500),
                'critiques' => intval($settings['tier_dev_critiques'] ?? 10),
                'ui_reverse' => intval($settings['tier_dev_ui_reverse'] ?? 10),
                'copilot' => intval($settings['tier_dev_copilot'] ?? 20),
            ],
            'pro' => [
                'screenshots' => intval($settings['tier_pro_screenshots'] ?? 2000),
                'critiques' => intval($settings['tier_pro_critiques'] ?? 50),
                'ui_reverse' => intval($settings['tier_pro_ui_reverse'] ?? 25),
                'copilot' => intval($settings['tier_pro_copilot'] ?? 100),
            ],
            'agency' => [
                'screenshots' => intval($settings['tier_agency_screenshots'] ?? 10000),
                'critiques' => intval($settings['tier_agency_critiques'] ?? 200),
                'ui_reverse' => intval($settings['tier_agency_ui_reverse'] ?? 100),
                'copilot' => intval($settings['tier_agency_copilot'] ?? 500),
            ],
            'enterprise' => [
                'screenshots' => intval($settings['tier_ent_screenshots'] ?? -1),
                'critiques' => intval($settings['tier_ent_critiques'] ?? -1),
                'ui_reverse' => intval($settings['tier_ent_ui_reverse'] ?? -1),
                'copilot' => intval($settings['tier_ent_copilot'] ?? -1),
            ],
            'starter' => [
                'screenshots' => intval($settings['tier_starter_screenshots'] ?? 100),
                'critiques' => intval($settings['tier_starter_critiques'] ?? 20),
                'ui_reverse' => intval($settings['tier_starter_ui_reverse'] ?? 10),
                'copilot' => intval($settings['tier_starter_copilot'] ?? 50),
            ],
        ];

        return $tiers[$tier] ?? $tiers['free'];
    }

    private function get_remaining_limit(int $limit, int $used): int {
        if ($limit < 0) {
            return -1;
        }
        return max(0, $limit - $used);
    }

    private function get_daily_usage_counts(int $license_id, int $user_id): array {
        global $wpdb;

        $today = date('Y-m-d');
        $counts = [
            'screenshot_used' => 0,
            'critique_used' => 0,
            'ui_reverse_used' => 0,
            'copilot_used' => 0,
        ];

        // Screenshot usage
        if ($license_id > 0) {
            $ss_table = $wpdb->prefix . 'uiai_screenshot_usage';
            if ($wpdb->get_var( $wpdb->prepare( 'SHOW TABLES LIKE %s', $ss_table ) ) === $ss_table) {
                $counts['screenshot_used'] = (int) ($wpdb->get_var($wpdb->prepare(
                    "SELECT COUNT(*) FROM {$ss_table} WHERE license_id = %d AND DATE(created_at) = %s",
                    $license_id,
                    $today
                )) ?: 0);
            }
        }

        if ($license_id > 0) {
            $usage_table = $wpdb->prefix . 'uiai_critique_usage';
            if ($wpdb->get_var("SHOW TABLES LIKE '{$usage_table}'") === $usage_table) {
                $counts['critique_used'] = (int) ($wpdb->get_var($wpdb->prepare(
                    "SELECT COUNT(*) FROM {$usage_table} WHERE license_id = %d AND DATE(created_at) = %s",
                    $license_id,
                    $today
                )) ?: 0);
            }

            $ui_reverse_table = $wpdb->prefix . 'uiai_ui_reverse_usage';
            if ($wpdb->get_var("SHOW TABLES LIKE '{$ui_reverse_table}'") === $ui_reverse_table) {
                $counts['ui_reverse_used'] = (int) ($wpdb->get_var($wpdb->prepare(
                    "SELECT COUNT(*) FROM {$ui_reverse_table} WHERE license_id = %d AND DATE(created_at) = %s",
                    $license_id,
                    $today
                )) ?: 0);
            }
        }

        if ($user_id > 0) {
            $copilot_table = $wpdb->prefix . 'uiai_copilot_usage';
            if ($wpdb->get_var("SHOW TABLES LIKE '{$copilot_table}'") === $copilot_table) {
                $counts['copilot_used'] = (int) ($wpdb->get_var($wpdb->prepare(
                    "SELECT COUNT(*) FROM {$copilot_table} WHERE user_id = %d AND DATE(created_at) = %s",
                    $user_id,
                    $today
                )) ?: 0);
            }
        }

        return $counts;
    }


    public function register_settings(): void {
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_posthog_host');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_posthog_key');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_browserless_url');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_browserless_token');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_anthropic_key');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_anthropic_model');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_openrouter_key');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_openrouter_model');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_openai_key');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_openai_model');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_webhook_secret');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_default_screenshot_provider');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_default_viewport_width');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_default_viewport_height');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_default_format');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_share_expiry_hours');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_enable_analytics');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_rate_limit_per_hour');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_rate_limit_per_day');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_max_file_size_mb');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_api_path');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_engine_path');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_engine_service_unit');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_cloudflared_config');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_cloudflared_log');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_cloudflared_service_unit');
        
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_free_screenshots');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_dev_screenshots');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_pro_screenshots');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_agency_screenshots');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_ent_screenshots');
        
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_free_critiques');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_dev_critiques');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_pro_critiques');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_agency_critiques');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_ent_critiques');
        
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_free_ui_reverse');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_dev_ui_reverse');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_pro_ui_reverse');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_agency_ui_reverse');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_ent_ui_reverse');

        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_free_copilot');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_dev_copilot');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_pro_copilot');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_agency_copilot');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_ent_copilot');
        
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_free_batch');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_dev_batch');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_pro_batch');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_agency_batch');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_ent_batch');
        
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_free_share');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_dev_share');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_pro_share');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_agency_share');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_ent_share');
        
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_free_comparison');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_dev_comparison');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_pro_comparison');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_agency_comparison');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_ent_comparison');
        
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_free_batch_enabled');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_dev_batch_enabled');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_pro_batch_enabled');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_agency_batch_enabled');
        register_setting('wpuiai_aic_settings', 'wpuiai_aic_tier_ent_batch_enabled');
    }

    public function enqueue_scripts(string $hook): void {
        if (strpos($hook, 'wpuiai-ai-cloud-settings') === false) {
            return;
        }

        $css_ver = file_exists(WPUIAI_AIC_PLUGIN_DIR . 'assets/css/settings.css') ? filemtime(WPUIAI_AIC_PLUGIN_DIR . 'assets/css/settings.css') : WPUIAI_AIC_VERSION;
        $js_ver = file_exists(WPUIAI_AIC_PLUGIN_DIR . 'assets/js/settings.js') ? filemtime(WPUIAI_AIC_PLUGIN_DIR . 'assets/js/settings.js') : WPUIAI_AIC_VERSION;

        wp_enqueue_style('wpuiai-aic-settings', WPUIAI_AIC_PLUGIN_URL . 'assets/css/settings.css', [], $css_ver);
        wp_enqueue_script('wpuiai-aic-settings', WPUIAI_AIC_PLUGIN_URL . 'assets/js/settings.js', ['jquery'], $js_ver, true);

        wp_localize_script('wpuiai-aic-settings', 'wpuiaiAICSettings', [
            'nonce' => wp_create_nonce('wpuiai_aic_settings'),
            'ajaxurl' => admin_url('admin-ajax.php'),
            'strings' => [
                'saved' => 'Settings saved successfully.',
                'error' => 'An error occurred. Please try again.',
                'testing' => 'Testing connection...',
                'connected' => 'Connection successful!',
                'failed' => 'Connection failed.',
            ],
        ]);

        // A12: Dev Mode JavaScript
        wp_add_inline_script('wpuiai-aic-settings', $this->get_dev_mode_js());
    }

    public function render_page(): void {
        if (!current_user_can('manage_options')) {
            wp_die('Access denied');
        }

        $settings = $this->get_settings();
        ?>
        <div class="wrap wpuiai-aic-wrap">
            <h1><span class="dashicons dashicons-admin-generic"></span> Settings</h1>

            <form id="wpuiai-aic-settings-form">
                <input type="hidden" name="action" value="wpuiai_aic_save_settings">
                <input type="hidden" name="nonce" value="<?php echo esc_attr(wp_create_nonce('wpuiai_aic_settings')); ?>">

                <div class="wpuiai-aic-settings-tabs">
                    <div class="nav-tab-wrapper">
                        <a href="#general" class="nav-tab nav-tab-active">General</a>
                        <a href="#tiers" class="nav-tab">Tiers</a>
                        <a href="#providers" class="nav-tab">Providers</a>
                        <a href="#analytics" class="nav-tab">Analytics</a>
                        <a href="#api" class="nav-tab">API</a>
                        <a href="#devmode" class="nav-tab" style="<?php echo $this->is_dev_mode() ? 'background:#f59e0b20;color:#92400e;' : ''; ?>">
                            ⚡ Dev Mode
                        </a>
                    </div>

                    <div id="general" class="tab-content active">
                        <table class="form-table">
                            <tr>
                                <th><label for="default_screenshot_provider">Default Screenshot Provider</label></th>
                                <td>
                                    <select id="default_screenshot_provider" name="default_screenshot_provider">
                                        <option value="go_engine" <?php selected($settings['default_screenshot_provider'], 'go_engine'); ?>>Go Engine (Recommended)</option>
                                        <option value="cloud" <?php selected($settings['default_screenshot_provider'], 'cloud'); ?>>Cloud Service</option>
                                        <option value="browserless" <?php selected($settings['default_screenshot_provider'], 'browserless'); ?>>Browserless</option>
                                        <option value="wkhtml" <?php selected($settings['default_screenshot_provider'], 'wkhtml'); ?>>wkhtmltoimage</option>
                                    </select>
                                    <p class="description">Choose the default service for capturing screenshots. Go Engine uses the built-in Rod pool — fastest and most reliable.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="default_viewport_width">Default Viewport Width</label></th>
                                <td>
                                    <input type="number" id="default_viewport_width" name="default_viewport_width" 
                                           value="<?php echo esc_attr($settings['default_viewport_width']); ?>" class="regular-text">
                                    <p class="description">Default width for screenshots in pixels.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="default_viewport_height">Default Viewport Height</label></th>
                                <td>
                                    <input type="number" id="default_viewport_height" name="default_viewport_height" 
                                           value="<?php echo esc_attr($settings['default_viewport_height']); ?>" class="regular-text">
                                    <p class="description">Default height for screenshots in pixels.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="default_format">Default Format</label></th>
                                <td>
                                    <select id="default_format" name="default_format">
                                        <option value="png" <?php selected($settings['default_format'], 'png'); ?>>PNG</option>
                                        <option value="jpeg" <?php selected($settings['default_format'], 'jpeg'); ?>>JPEG</option>
                                        <option value="webp" <?php selected($settings['default_format'], 'webp'); ?>>WebP</option>
                                    </select>
                                    <p class="description">Default image format for screenshots.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="share_expiry_hours">Share Expiry Hours</label></th>
                                <td>
                                    <input type="number" id="share_expiry_hours" name="share_expiry_hours" 
                                           value="<?php echo esc_attr($settings['share_expiry_hours']); ?>" class="regular-text">
                                    <p class="description">How many hours before shared screenshots expire.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="max_file_size_mb">Max File Size (MB)</label></th>
                                <td>
                                    <input type="number" id="max_file_size_mb" name="max_file_size_mb" 
                                           value="<?php echo esc_attr($settings['max_file_size_mb']); ?>" class="regular-text">
                                    <p class="description">Maximum allowed file size for screenshots.</p>
                                </td>
                            </tr>
                        </table>

                        <h3>Infrastructure Overrides</h3>
                        <p class="description">Keep defaults unless the services live somewhere else. These values feed the Services screen and REST controls.</p>
                        <table class="form-table">
                            <tr>
                                <th><label for="engine_path">Go Engine Path</label></th>
                                <td>
                                    <input type="text" id="engine_path" name="engine_path" value="<?php echo esc_attr($settings['engine_path']); ?>" class="regular-text code">
                                    <p class="description">Root directory for the UIAI Go Engine (binary, config.yaml, logs/).</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="engine_service_unit">Go Engine Service Unit</label></th>
                                <td>
                                    <input type="text" id="engine_service_unit" name="engine_service_unit" value="<?php echo esc_attr($settings['engine_service_unit']); ?>" class="regular-text code">
                                    <p class="description">Systemd unit name for the Go AI engine.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="cloudflared_service_unit">Cloudflared Service Unit</label></th>
                                <td>
                                    <input type="text" id="cloudflared_service_unit" name="cloudflared_service_unit" value="<?php echo esc_attr($settings['cloudflared_service_unit']); ?>" class="regular-text code" placeholder="cloudflared-wpuiai">
                                    <p class="description">Optional unit name for the Cloudflare tunnel. Leave blank to run the fallback command.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="cloudflared_config">Cloudflared Config Path</label></th>
                                <td>
                                    <input type="text" id="cloudflared_config" name="cloudflared_config" value="<?php echo esc_attr($settings['cloudflared_config']); ?>" class="regular-text code">
                                    <p class="description">YAML config file passed to <code>cloudflared tunnel</code>.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="cloudflared_log">Cloudflared Log Path</label></th>
                                <td>
                                    <input type="text" id="cloudflared_log" name="cloudflared_log" value="<?php echo esc_attr($settings['cloudflared_log']); ?>" class="regular-text code">
                                    <p class="description">Log file shown in the Services screen.</p>
                                </td>
                            </tr>
                        </table>
                    </div>

                    <div id="tiers" class="tab-content">
                        <h3>License Tier Limits</h3>
                        <p class="description">Configure feature limits for each license tier. These limits are stored in EDD license meta.</p>
                        
                        <table class="form-table">
                            <thead>
                                <tr>
                                    <th style="width: 150px;">Feature</th>
                                    <th>Free (ID: 21)</th>
                                    <th>Developer (ID: 22)</th>
                                    <th>Pro (ID: 23)</th>
                                    <th>Agency (ID: 24)</th>
                                    <th>Enterprise (ID: 25)</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr>
                                    <td><strong>Screenshots/Day</strong></td>
                                    <td><input type="number" name="tier_free_screenshots" value="<?php echo esc_attr($settings['tier_free_screenshots'] ?? 10); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_dev_screenshots" value="<?php echo esc_attr($settings['tier_dev_screenshots'] ?? 500); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_pro_screenshots" value="<?php echo esc_attr($settings['tier_pro_screenshots'] ?? 2000); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_agency_screenshots" value="<?php echo esc_attr($settings['tier_agency_screenshots'] ?? 10000); ?>" class="small-text"></td>
                                    <td><input type="text" name="tier_ent_screenshots" value="<?php echo esc_attr($settings['tier_ent_screenshots'] ?? '-1'); ?>" class="small-text" placeholder="Unlimited"> <span class="description">(-1 = unlimited)</span></td>
                                </tr>
                                <tr>
                                    <td><strong>Critiques/Day</strong></td>
                                    <td><input type="number" name="tier_free_critiques" value="<?php echo esc_attr($settings['tier_free_critiques'] ?? 0); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_dev_critiques" value="<?php echo esc_attr($settings['tier_dev_critiques'] ?? 10); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_pro_critiques" value="<?php echo esc_attr($settings['tier_pro_critiques'] ?? 50); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_agency_critiques" value="<?php echo esc_attr($settings['tier_agency_critiques'] ?? 200); ?>" class="small-text"></td>
                                    <td><input type="text" name="tier_ent_critiques" value="<?php echo esc_attr($settings['tier_ent_critiques'] ?? '-1'); ?>" class="small-text" placeholder="Unlimited"></td>
                                </tr>
                                <tr>
                                    <td><strong>UI Reverse/Day</strong></td>
                                    <td><input type="number" name="tier_free_ui_reverse" value="<?php echo esc_attr($settings['tier_free_ui_reverse'] ?? 0); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_dev_ui_reverse" value="<?php echo esc_attr($settings['tier_dev_ui_reverse'] ?? 10); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_pro_ui_reverse" value="<?php echo esc_attr($settings['tier_pro_ui_reverse'] ?? 25); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_agency_ui_reverse" value="<?php echo esc_attr($settings['tier_agency_ui_reverse'] ?? 100); ?>" class="small-text"></td>
                                    <td><input type="text" name="tier_ent_ui_reverse" value="<?php echo esc_attr($settings['tier_ent_ui_reverse'] ?? '-1'); ?>" class="small-text" placeholder="Unlimited"></td>
                                </tr>
                                <tr>
                                    <td><strong>Copilot Chats/Day</strong></td>
                                    <td><input type="number" name="tier_free_copilot" value="<?php echo esc_attr($settings['tier_free_copilot'] ?? 0); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_dev_copilot" value="<?php echo esc_attr($settings['tier_dev_copilot'] ?? 20); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_pro_copilot" value="<?php echo esc_attr($settings['tier_pro_copilot'] ?? 100); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_agency_copilot" value="<?php echo esc_attr($settings['tier_agency_copilot'] ?? 500); ?>" class="small-text"></td>
                                    <td><input type="text" name="tier_ent_copilot" value="<?php echo esc_attr($settings['tier_ent_copilot'] ?? '-1'); ?>" class="small-text" placeholder="Unlimited"></td>
                                </tr>
                                <tr>
                                    <td><strong>Batch Concurrency</strong></td>
                                    <td><input type="number" name="tier_free_batch" value="<?php echo esc_attr($settings['tier_free_batch'] ?? 1); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_dev_batch" value="<?php echo esc_attr($settings['tier_dev_batch'] ?? 3); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_pro_batch" value="<?php echo esc_attr($settings['tier_pro_batch'] ?? 5); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_agency_batch" value="<?php echo esc_attr($settings['tier_agency_batch'] ?? 20); ?>" class="small-text"></td>
                                    <td><input type="number" name="tier_ent_batch" value="<?php echo esc_attr($settings['tier_ent_batch'] ?? 100); ?>" class="small-text"></td>
                                </tr>
                                <tr>
                                    <td><strong>Share Expiry</strong></td>
                                    <td><input type="number" name="tier_free_share" value="<?php echo esc_attr($settings['tier_free_share'] ?? 0); ?>" class="small-text"> <span class="description">hrs</span></td>
                                    <td><input type="number" name="tier_dev_share" value="<?php echo esc_attr($settings['tier_dev_share'] ?? 1440); ?>" class="small-text"> <span class="description">hrs</span></td>
                                    <td><input type="number" name="tier_pro_share" value="<?php echo esc_attr($settings['tier_pro_share'] ?? 1440); ?>" class="small-text"> <span class="description">hrs</span></td>
                                    <td><input type="number" name="tier_agency_share" value="<?php echo esc_attr($settings['tier_agency_share'] ?? 10080); ?>" class="small-text"> <span class="description">hrs</span></td>
                                    <td><input type="number" name="tier_ent_share" value="<?php echo esc_attr($settings['tier_ent_share'] ?? 43200); ?>" class="small-text"> <span class="description">hrs</span></td>
                                </tr>
                                <tr>
                                    <td><strong>Comparison</strong></td>
                                    <td><input type="checkbox" name="tier_free_comparison" value="1" <?php checked($settings['tier_free_comparison'] ?? '', '1'); ?>></td>
                                    <td><input type="checkbox" name="tier_dev_comparison" value="1" <?php checked($settings['tier_dev_comparison'] ?? '', '1'); ?>></td>
                                    <td><input type="checkbox" name="tier_pro_comparison" value="1" <?php checked($settings['tier_pro_comparison'] ?? '', '1'); ?>></td>
                                    <td><input type="checkbox" name="tier_agency_comparison" value="1" <?php checked($settings['tier_agency_comparison'] ?? '', '1'); ?>></td>
                                    <td><input type="checkbox" name="tier_ent_comparison" value="1" <?php checked($settings['tier_ent_comparison'] ?? '', '1'); ?>></td>
                                </tr>
                                <tr>
                                    <td><strong>Batch Processing</strong></td>
                                    <td><input type="checkbox" name="tier_free_batch_enabled" value="1" <?php checked($settings['tier_free_batch_enabled'] ?? '', ''); ?>></td>
                                    <td><input type="checkbox" name="tier_dev_batch_enabled" value="1" <?php checked($settings['tier_dev_batch_enabled'] ?? '', '1'); ?>></td>
                                    <td><input type="checkbox" name="tier_pro_batch_enabled" value="1" <?php checked($settings['tier_pro_batch_enabled'] ?? '', '1'); ?>></td>
                                    <td><input type="checkbox" name="tier_agency_batch_enabled" value="1" <?php checked($settings['tier_agency_batch_enabled'] ?? '', '1'); ?>></td>
                                    <td><input type="checkbox" name="tier_ent_batch_enabled" value="1" <?php checked($settings['tier_ent_batch_enabled'] ?? '', '1'); ?>></td>
                                </tr>
                            </tbody>
                        </table>
                    </div>

                    <div id="providers" class="tab-content">
                        <h3>Browserless</h3>
                        <table class="form-table">
                            <tr>
                                <th><label for="browserless_url">Browserless URL</label></th>
                                <td>
                                    <input type="url" id="browserless_url" name="browserless_url" 
                                           value="<?php echo esc_attr($settings['browserless_url']); ?>" class="regular-text">
                                    <p class="description">URL of your Browserless instance.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="browserless_token">Browserless Token</label></th>
                                <td>
                                    <input type="password" id="browserless_token" name="browserless_token" 
                                           value="<?php echo esc_attr($settings['browserless_token']); ?>" class="regular-text">
                                    <p class="description">Authentication token for Browserless.</p>
                                    <button type="button" class="button button-small test-connection" data-provider="browserless">
                                        Test Connection
                                    </button>
                                </td>
                            </tr>
                        </table>

                        <h3>Cloud Service</h3>
                        <table class="form-table">
                            <tr>
                                <th><label for="cloud_api_key">Cloud API Key</label></th>
                                <td>
                                    <input type="password" id="cloud_api_key" name="cloud_api_key" 
                                           value="<?php echo esc_attr($settings['cloud_api_key'] ?? ''); ?>" class="regular-text">
                                    <p class="description">API key for the cloud screenshot service.</p>
                                </td>
                            </tr>
                        </table>
                    </div>

                    <div id="analytics" class="tab-content">
                        <h3>PostHog Analytics</h3>
                        <table class="form-table">
                            <tr>
                                <th><label for="enable_analytics">Enable Analytics</label></th>
                                <td>
                                    <label>
                                        <input type="checkbox" id="enable_analytics" name="enable_analytics" value="1" 
                                               <?php checked($settings['enable_analytics'], '1'); ?>>
                                        Enable PostHog analytics tracking
                                    </label>
                                    <p class="description">Track usage and analytics via PostHog.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="posthog_host">PostHog Host</label></th>
                                <td>
                                    <input type="url" id="posthog_host" name="posthog_host" 
                                           value="<?php echo esc_attr($settings['posthog_host']); ?>" class="regular-text">
                                    <p class="description">Your PostHog instance URL (e.g., https://app.posthog.com).</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="posthog_key">PostHog Key</label></th>
                                <td>
                                    <input type="password" id="posthog_key" name="posthog_key" 
                                           value="<?php echo esc_attr($settings['posthog_key']); ?>" class="regular-text">
                                    <p class="description">Your PostHog project API key.</p>
                                    <button type="button" class="button button-small test-connection" data-provider="posthog">
                                        Test Connection
                                    </button>
                                </td>
                            </tr>
                        </table>
                    </div>

                    <div id="api" class="tab-content">
                        <h3>AI Providers</h3>
                        <div style="background:#eff6ff; border:1px solid #bfdbfe; border-radius:6px; padding:12px 16px; margin-bottom:20px;">
                            <p style="margin:0; font-size:13px; color:#1e3a5f; line-height:1.5;">
                                <strong>Cloud engine defaults come from the parent plugin settings.</strong>
                                If the parent plugin (Settings → AI Engine) has a provider/model configured,
                                the Go engine uses that automatically. Keys set here are <em>overrides</em> —
                                only needed if the cloud admin site runs a different set of keys than the parent plugin.
                            </p>
                        </div>

                        <h4>Anthropic Claude</h4>
                        <table class="form-table">
                            <tr>
                                <th><label for="anthropic_key">API Key</label></th>
                                <td>
                                    <input type="password" id="anthropic_key" name="anthropic_key" 
                                           value="<?php echo esc_attr($settings['anthropic_key']); ?>" class="regular-text">
                                    <p class="description">Your Anthropic API key for Claude. Falls back to parent plugin's <code>uiai_anthropic_key</code> if empty.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="anthropic_model">Default Model</label></th>
                                <td>
                                    <input type="text" id="anthropic_model" name="anthropic_model" 
                                           value="<?php echo esc_attr($settings['anthropic_model']); ?>" class="regular-text">
                                    <p class="description">Override the default Claude model. Leave blank to use the parent plugin's model setting.</p>
                                </td>
                            </tr>
                        </table>

                        <h4>OpenRouter</h4>
                        <table class="form-table">
                            <tr>
                                <th><label for="openrouter_key">API Key</label></th>
                                <td>
                                    <input type="password" id="openrouter_key" name="openrouter_key" 
                                           value="<?php echo esc_attr($settings['openrouter_key']); ?>" class="regular-text">
                                    <p class="description">Your OpenRouter API key. Falls back to parent plugin's <code>uiai_openrouter_key</code> if empty.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="openrouter_model">Default Model</label></th>
                                <td>
                                    <input type="text" id="openrouter_model" name="openrouter_model" 
                                           value="<?php echo esc_attr($settings['openrouter_model']); ?>" class="regular-text">
                                    <p class="description">Override the default OpenRouter model. Leave blank to use the parent plugin's model setting.</p>
                                </td>
                            </tr>
                        </table>

                        <h4>OpenAI</h4>
                        <table class="form-table">
                            <tr>
                                <th><label for="openai_key">API Key</label></th>
                                <td>
                                    <input type="password" id="openai_key" name="openai_key" 
                                           value="<?php echo esc_attr($settings['openai_key']); ?>" class="regular-text">
                                    <p class="description">Your OpenAI API key.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="openai_model">Default Model</label></th>
                                <td>
                                    <input type="text" id="openai_model" name="openai_model" 
                                            value="<?php echo esc_attr($settings['openai_model']); ?>" class="regular-text">
                                    <p class="description">Override the default OpenAI model. Leave blank to use the parent plugin's model setting.</p>
                                </td>
                            </tr>
                        </table>

                        <h4>Fireworks</h4>
                        <table class="form-table">
                            <tr>
                                <th><label for="fireworks_key">API Key</label></th>
                                <td>
                                    <input type="password" id="fireworks_key" name="fireworks_key" 
                                           value="<?php echo esc_attr($settings['fireworks_key'] ?? ''); ?>" class="regular-text">
                                    <p class="description">Your Fireworks AI API key. <a href="https://fireworks.ai/" target="_blank">Get key</a></p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="fireworks_model">Default Model</label></th>
                                <td>
                                    <input type="text" id="fireworks_model" name="fireworks_model" 
                                           value="<?php echo esc_attr($settings['fireworks_model'] ?? ''); ?>" class="regular-text"
                                           placeholder="Leave blank to use parent plugin setting">
                                    <p class="description">Override the default Fireworks model. Leave blank to use the parent plugin's model setting.</p>
                                </td>
                            </tr>
                        </table>

                        <h4>Kimi (Moonshot)</h4>
                        <table class="form-table">
                            <tr>
                                <th><label for="kimi_key">API Key</label></th>
                                <td>
                                    <input type="password" id="kimi_key" name="kimi_key" 
                                           value="<?php echo esc_attr($settings['kimi_key'] ?? ''); ?>" class="regular-text">
                                    <p class="description">Your Kimi/Moonshot API key. <a href="https://kimi.moonshot.cn/" target="_blank">Get key</a></p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="kimi_model">Default Model</label></th>
                                <td>
                                    <input type="text" id="kimi_model" name="kimi_model" 
                                           value="<?php echo esc_attr($settings['kimi_model'] ?? ''); ?>" class="regular-text"
                                           placeholder="Leave blank to use parent plugin setting">
                                    <p class="description">Override the default Kimi model. Leave blank to use the parent plugin's model setting.</p>
                                </td>
                            </tr>
                        </table>

                        <h4>Qwen (Alibaba)</h4>
                        <table class="form-table">
                            <tr>
                                <th><label for="qwen_key">API Key</label></th>
                                <td>
                                    <input type="password" id="qwen_key" name="qwen_key" 
                                           value="<?php echo esc_attr($settings['qwen_key'] ?? ''); ?>" class="regular-text">
                                    <p class="description">Your Qwen/DashScope API key. <a href="https://dashscope.aliyun.com/" target="_blank">Get key</a></p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="qwen_model">Default Model</label></th>
                                <td>
                                    <input type="text" id="qwen_model" name="qwen_model" 
                                           value="<?php echo esc_attr($settings['qwen_model'] ?? ''); ?>" class="regular-text"
                                           placeholder="Leave blank to use parent plugin setting">
                                    <p class="description">Override the default Qwen model. Leave blank to use the parent plugin's model setting.</p>
                                </td>
                            </tr>
                        </table>

                        <h3>Go Engine Authentication</h3>
                        <table class="form-table">
                            <tr>
                                <th><label for="webhook_secret">Webhook Secret</label></th>
                                <td>
                                    <input type="password" id="webhook_secret" name="webhook_secret" 
                                           value="<?php echo esc_attr($settings['webhook_secret']); ?>" class="regular-text">
                                    <p class="description">
                                        Shared secret between the Go AI engine and WordPress.
                                        The engine sends this as <code>X-Webhook-Secret</code> header when calling WP REST endpoints
                                        (license validation, credit deduction, AI settings sync).
                                        Must match <code>WEBHOOK_SECRET</code> in <code>/etc/wpuiai/ai-api.env</code>.
                                        Generate with: <code>openssl rand -hex 32</code>
                                    </p>
                                </td>
                            </tr>
                        </table>

                        <h3>Rate Limiting</h3>
                        <table class="form-table">
                            <tr>
                                <th><label for="rate_limit_per_hour">Rate Limit Per Hour</label></th>
                                <td>
                                    <input type="number" id="rate_limit_per_hour" name="rate_limit_per_hour" 
                                           value="<?php echo esc_attr($settings['rate_limit_per_hour']); ?>" class="regular-text">
                                    <p class="description">Maximum requests per hour per client.</p>
                                </td>
                            </tr>
                            <tr>
                                <th><label for="rate_limit_per_day">Rate Limit Per Day</label></th>
                                <td>
                                    <input type="number" id="rate_limit_per_day" name="rate_limit_per_day" 
                                           value="<?php echo esc_attr($settings['rate_limit_per_day']); ?>" class="regular-text">
                                    <p class="description">Maximum requests per day per client.</p>
                                </td>
                            </tr>
                        </table>
                    </div>

                    <?php $this->render_dev_mode_tab(); ?>
                </div>

                <p class="submit">
                    <button type="submit" class="button button-primary">Save Settings</button>
                </p>
            </form>
        </div>
        <?php
    }

    /**
     * Render the Dev Mode tab content (A2-A5).
     */
    private function render_dev_mode_tab(): void {
        $is_dev       = $this->is_dev_mode();
        $option_val   = get_option( 'uiai_dev_mode', '0' );
        $const_def    = defined( 'WPUIAI_DEV_MODE' );
        $cloud_on     = get_option( 'uiai_use_ai_cloud', '0' ) === '1';
        $secret       = get_option( 'wpuiai_aic_webhook_secret', '' );
        $secret_hint  = $secret ? substr( $secret, 0, 10 ) . '...' : '(not set)';
        $provider     = get_option( 'uiai_ai_provider', '(not set)' );
        $model        = get_option( 'uiai_ai_model', '(not set)' );
        $nonce        = wp_create_nonce( 'wpuiai_aic_dev_mode_nonce' );
        ?>
        <div id="devmode" class="tab-content">
            <div class="wpuiai-devmode-panel" style="max-width:900px;">

                <!-- A2: Status Section -->
                <div class="wpuiai-devmode-card" style="border-left:4px solid <?php echo $is_dev ? '#f59e0b' : '#d1d5db'; ?>;background:<?php echo $is_dev ? '#fefce820' : '#f9fafb'; ?>;border:1px solid <?php echo $is_dev ? '#fcd34d' : '#e5e7eb'; ?>;border-left:4px solid <?php echo $is_dev ? '#f59e0b' : '#d1d5db'; ?>;border-radius:8px;padding:20px;margin-bottom:20px;">
                    <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px;">
                        <h3 style="margin:0;font-size:16px;color:<?php echo $is_dev ? '#92400e' : '#374151'; ?>;">
                            ⚡ Developer Mode
                        </h3>
                        <button type="button" id="wpuiai-dev-toggle" class="button <?php echo $is_dev ? 'button-secondary' : 'button-primary'; ?>"
                                data-enabled="<?php echo $is_dev ? '1' : '0'; ?>"
                                style="<?php echo $is_dev ? 'background:#fef3c7;color:#92400e;border-color:#f59e0b;' : ''; ?>">
                            <?php echo $is_dev ? '❄️ Disable Dev Mode' : '🔥 Enable Dev Mode'; ?>
                        </button>
                    </div>
                    <p style="margin:0 0 16px;color:#6b7280;font-size:13px;">
                        When active, dev mode bypasses license validation and credit checks across the entire WPUIAI ecosystem.
                        All AI endpoints authenticate via webhook secret as "internal" tier.
                    </p>
                    <?php if ( $const_def ) : ?>
                        <div style="background:#dbeafe;border:1px solid #93c5fd;border-radius:4px;padding:8px 12px;margin-bottom:12px;font-size:12px;color:#1e40af;">
                            ℹ️ <code>WPUIAI_DEV_MODE</code> constant is defined in <code>wp-config.php</code>. It takes precedence over the toggle.
                        </div>
                    <?php endif; ?>
                    <table style="font-size:13px;border-collapse:collapse;width:100%;">
                        <tr style="border-bottom:1px solid #f3f4f6;">
                            <td style="padding:8px 0;color:#6b7280;width:220px;">Option (<code>uiai_dev_mode</code>)</td>
                            <td style="padding:8px 0;"><?php echo $option_val ? '<span style="color:#059669;">● Active</span>' : '<span style="color:#9ca3af;">○ Inactive</span>'; ?></td>
                        </tr>
                        <tr style="border-bottom:1px solid #f3f4f6;">
                            <td style="padding:8px 0;color:#6b7280;">Constant (<code>WPUIAI_DEV_MODE</code>)</td>
                            <td style="padding:8px 0;"><?php echo $const_def ? '<span style="color:#059669;">● Defined</span>' : '<span style="color:#9ca3af;">○ Not defined</span>'; ?></td>
                        </tr>
                        <tr style="border-bottom:1px solid #f3f4f6;">
                            <td style="padding:8px 0;color:#6b7280;">Effective</td>
                            <td style="padding:8px 0;"><?php echo $is_dev ? '<span style="color:#059669;">✅ Dev mode ON</span>' : '<span style="color:#ef4444;">❌ Dev mode OFF</span>'; ?></td>
                        </tr>
                        <tr style="border-bottom:1px solid #f3f4f6;">
                            <td style="padding:8px 0;color:#6b7280;">License tier override</td>
                            <td style="padding:8px 0;"><?php echo $is_dev ? '<code>enterprise</code>' : '<span style="color:#9ca3af;">N/A</span>'; ?></td>
                        </tr>
                        <tr style="border-bottom:1px solid #f3f4f6;">
                            <td style="padding:8px 0;color:#6b7280;">Cloud auth method</td>
                            <td style="padding:8px 0;"><code><?php echo $is_dev ? 'X-Webhook-Secret' : 'X-License-Key'; ?></code></td>
                        </tr>
                        <tr style="border-bottom:1px solid #f3f4f6;">
                            <td style="padding:8px 0;color:#6b7280;">Credit balance</td>
                            <td style="padding:8px 0;"><?php echo $is_dev ? '999,999 <span style="color:#9ca3af;">(synthetic)</span>' : '<span style="color:#9ca3af;">(real)</span>'; ?></td>
                        </tr>
                        <tr>
                            <td style="padding:8px 0;color:#6b7280;">Cloud opt-in</td>
                            <td style="padding:8px 0;"><?php echo $cloud_on ? '<span style="color:#059669;">● Enabled</span>' : '<span style="color:#9ca3af;">○ Disabled</span>'; ?></td>
                        </tr>
                    </table>
                </div>

                <!-- A3: Ecosystem Health -->
                <div class="wpuiai-devmode-card" style="border:1px solid #e5e7eb;border-radius:8px;padding:20px;margin-bottom:20px;">
                    <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px;">
                        <h3 style="margin:0;font-size:15px;color:#374151;">Ecosystem Health</h3>
                        <button type="button" id="wpuiai-dev-health-refresh" class="button button-secondary" style="font-size:12px;">🔄 Refresh</button>
                    </div>
                    <div id="wpuiai-dev-health-rows" style="font-size:13px;">
                        <div data-check="go_engine" style="display:flex;align-items:center;gap:8px;padding:8px 0;border-bottom:1px solid #f3f4f6;">
                            <span style="width:180px;color:#6b7280;">Go Engine (7456)</span>
                            <span class="health-status" style="color:#9ca3af;">⏳ Checking...</span>
                        </div>
                        <div data-check="tunnel" style="display:flex;align-items:center;gap:8px;padding:8px 0;border-bottom:1px solid #f3f4f6;">
                            <span style="width:180px;color:#6b7280;">Cloudflare Tunnel</span>
                            <span class="health-status" style="color:#9ca3af;">⏳ Checking...</span>
                        </div>
                        <div data-check="rest" style="display:flex;align-items:center;gap:8px;padding:8px 0;border-bottom:1px solid #f3f4f6;">
                            <span style="width:180px;color:#6b7280;">WP REST ai-settings</span>
                            <span class="health-status" style="color:#9ca3af;">⏳ Checking...</span>
                        </div>
                        <div data-check="webhook" style="display:flex;align-items:center;gap:8px;padding:8px 0;border-bottom:1px solid #f3f4f6;">
                            <span style="width:180px;color:#6b7280;">Webhook secret sync</span>
                            <span class="health-status" style="color:#9ca3af;">⏳ Checking...</span>
                        </div>
                        <div data-check="rod" style="display:flex;align-items:center;gap:8px;padding:8px 0;">
                            <span style="width:180px;color:#6b7280;">Rod pool</span>
                            <span class="health-status" style="color:#9ca3af;">⏳ Checking...</span>
                        </div>
                    </div>
                    <div style="margin-top:12px;padding-top:12px;border-top:1px solid #e5e7eb;font-size:12px;color:#6b7280;">
                        Default provider: <strong><?php echo esc_html( $provider ); ?></strong> ·
                        Default model: <strong><?php echo esc_html( $model ); ?></strong>
                        <br><span style="font-size:11px;">(from WP Settings → AI Engine)</span>
                    </div>
                </div>

                <!-- A4: Quick Actions -->
                <div class="wpuiai-devmode-card" style="border:1px solid #e5e7eb;border-radius:8px;padding:20px;margin-bottom:20px;">
                    <h3 style="margin:0 0 16px;font-size:15px;color:#374151;">Quick Actions</h3>
                    <div style="display:flex;flex-direction:column;gap:12px;">
                        <div style="display:flex;align-items:center;gap:12px;flex-wrap:wrap;">
                            <button type="button" id="wpuiai-dev-test-critique" class="button button-secondary">🧪 Test Cloud Critique</button>
                            <span id="wpuiai-dev-test-result" style="font-size:12px;color:#6b7280;"></span>
                        </div>
                        <div style="display:flex;align-items:center;gap:12px;flex-wrap:wrap;">
                            <button type="button" id="wpuiai-dev-rotate-secret" class="button button-secondary">🔑 Rotate Webhook Secret</button>
                            <span style="font-size:12px;color:#9ca3af;">Current: <code><?php echo esc_html( $secret_hint ); ?></code></span>
                            <span id="wpuiai-dev-rotate-result" style="font-size:12px;color:#6b7280;"></span>
                        </div>
                        <div style="display:flex;align-items:center;gap:12px;flex-wrap:wrap;">
                            <button type="button" id="wpuiai-dev-clear-cache" class="button button-secondary">🗑️ Clear Credit Cache</button>
                            <span id="wpuiai-dev-cache-result" style="font-size:12px;color:#6b7280;"></span>
                        </div>
                        <div style="display:flex;align-items:center;gap:12px;flex-wrap:wrap;">
                            <button type="button" id="wpuiai-dev-force-sync" class="button button-secondary">🔄 Force Settings Sync</button>
                            <span id="wpuiai-dev-sync-result" style="font-size:12px;color:#6b7280;"></span>
                        </div>
                    </div>
                </div>

                <!-- A5: Capability Routing -->
                <div class="wpuiai-devmode-card" style="border:1px solid #e5e7eb;border-radius:8px;padding:20px;margin-bottom:20px;">
                    <h3 style="margin:0 0 16px;font-size:15px;color:#374151;">Capability Routing (live)</h3>
                    <?php if ( class_exists( 'WPUIAI_Capability_Router' ) ) : ?>
                        <?php
                        $router = \WPUIAI_Capability_Router::instance();
                        $caps = [ 'critique', 'ui_reverse', 'layout_compare', 'section_detect', 'style_enhance', 'screenshot_capture', 'copilot_chat' ];
                        ?>
                        <table style="width:100%;font-size:13px;border-collapse:collapse;">
                            <thead>
                                <tr style="border-bottom:2px solid #e5e7eb;text-align:left;">
                                    <th style="padding:6px 0;color:#6b7280;">Capability</th>
                                    <th style="padding:6px 0;color:#6b7280;">Policy</th>
                                    <th style="padding:6px 0;color:#6b7280;">Resolves to</th>
                                </tr>
                            </thead>
                            <tbody>
                                <?php foreach ( $caps as $cap_id ) :
                                    $cap     = $router->get_capability( $cap_id );
                                    $policy  = $cap['policy'] ?? 'unknown';
                                    $cloud   = method_exists( $router, 'is_cloud_available' ) ? $router->is_cloud_available( $cap_id ) : false;
                                    $icon    = $cloud ? '☁️' : '💻';
                                    $target  = $cloud ? 'Cloud AI' : 'Local';
                                ?>
                                <tr style="border-bottom:1px solid #f3f4f6;">
                                    <td style="padding:6px 0;"><code><?php echo esc_html( $cap_id ); ?></code></td>
                                    <td style="padding:6px 0;color:#6b7280;"><?php echo esc_html( $policy ); ?></td>
                                    <td style="padding:6px 0;"><?php echo $icon . ' ' . esc_html( $target ); ?></td>
                                </tr>
                                <?php endforeach; ?>
                            </tbody>
                        </table>
                        <p style="margin:8px 0 0;font-size:11px;color:#9ca3af;">(queried live from WPUIAI_Capability_Router)</p>
                    <?php else : ?>
                        <p style="color:#ef4444;font-size:13px;">WPUIAI parent plugin not active — capability routing unavailable.</p>
                    <?php endif; ?>
                </div>

                <!-- A5b: Effects Reference -->
                <div class="wpuiai-devmode-card" style="border:1px solid #e5e7eb;border-radius:8px;padding:20px;">
                    <h3 style="margin:0 0 16px;font-size:15px;color:#374151;">Dev Mode Effects Reference</h3>
                    <table style="width:100%;font-size:13px;border-collapse:collapse;">
                        <thead>
                            <tr style="border-bottom:2px solid #e5e7eb;text-align:left;">
                                <th style="padding:6px 0;color:#6b7280;">Component</th>
                                <th style="padding:6px 0;color:#6b7280;">Normal</th>
                                <th style="padding:6px 0;color:#6b7280;">Dev Mode</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr style="border-bottom:1px solid #f3f4f6;"><td style="padding:6px 0;">License check</td><td>EDD license lookup</td><td>Always valid</td></tr>
                            <tr style="border-bottom:1px solid #f3f4f6;"><td style="padding:6px 0;">Tier</td><td>From EDD product</td><td>enterprise</td></tr>
                            <tr style="border-bottom:1px solid #f3f4f6;"><td style="padding:6px 0;">Feature gates</td><td>Tier-dependent</td><td>All unlocked</td></tr>
                            <tr style="border-bottom:1px solid #f3f4f6;"><td style="padding:6px 0;">Cloud auth header</td><td><code>X-License-Key</code></td><td><code>X-Webhook-Secret</code></td></tr>
                            <tr style="border-bottom:1px solid #f3f4f6;"><td style="padding:6px 0;">Go engine identity</td><td>Per-license tier</td><td>"internal"</td></tr>
                            <tr style="border-bottom:1px solid #f3f4f6;"><td style="padding:6px 0;">Credit balance</td><td>Real from DB</td><td>999,999</td></tr>
                            <tr style="border-bottom:1px solid #f3f4f6;"><td style="padding:6px 0;">Credit deduction</td><td>Real deduction</td><td>No-op</td></tr>
                            <tr><td style="padding:6px 0;">Rate limits</td><td>Tier-based</td><td>Unlimited</td></tr>
                        </tbody>
                    </table>
                </div>

            </div>

            <input type="hidden" id="wpuiai-dev-nonce" value="<?php echo esc_attr( $nonce ); ?>">
        </div>
        <?php
    }

    public function ajax_save_settings(): void {
        check_ajax_referer('wpuiai_aic_settings', 'nonce');

        if (!current_user_can('manage_options')) {
            wp_send_json_error(['message' => 'Permission denied']);
            return;
        }

        $settings = [
            'default_screenshot_provider' => sanitize_text_field($_POST['default_screenshot_provider'] ?? 'go_engine'),
            'default_viewport_width' => intval($_POST['default_viewport_width'] ?? 1280),
            'default_viewport_height' => intval($_POST['default_viewport_height'] ?? 800),
            'default_format' => sanitize_text_field($_POST['default_format'] ?? 'png'),
            'share_expiry_hours' => intval($_POST['share_expiry_hours'] ?? 24),
            'max_file_size_mb' => intval($_POST['max_file_size_mb'] ?? 10),
            'engine_path' => sanitize_text_field($_POST['engine_path'] ?? '/home/wpuiai/uiai-engine'),
            'engine_service_unit' => sanitize_text_field($_POST['engine_service_unit'] ?? 'uiai-engine'),
            'cloudflared_service_unit' => sanitize_text_field($_POST['cloudflared_service_unit'] ?? 'cloudflared-wpuiai'),
            'cloudflared_config' => sanitize_text_field($_POST['cloudflared_config'] ?? '/etc/cloudflared/wpuiai.yml'),
            'cloudflared_log' => sanitize_text_field($_POST['cloudflared_log'] ?? '/var/log/cloudflared.log'),
            'browserless_url' => esc_url_raw($_POST['browserless_url'] ?? ''),
            'browserless_token' => sanitize_text_field($_POST['browserless_token'] ?? ''),
            'cloud_api_key' => sanitize_text_field($_POST['cloud_api_key'] ?? ''),
            'enable_analytics' => isset($_POST['enable_analytics']) ? '1' : '0',
            'posthog_host' => esc_url_raw($_POST['posthog_host'] ?? ''),
            'posthog_key' => sanitize_text_field($_POST['posthog_key'] ?? ''),
            'anthropic_key' => sanitize_text_field($_POST['anthropic_key'] ?? ''),
            'anthropic_model' => sanitize_text_field($_POST['anthropic_model'] ?? ''),
            'openrouter_key' => sanitize_text_field($_POST['openrouter_key'] ?? ''),
            'openrouter_model' => sanitize_text_field($_POST['openrouter_model'] ?? ''),
            'openai_key' => sanitize_text_field($_POST['openai_key'] ?? ''),
            'openai_model' => sanitize_text_field($_POST['openai_model'] ?? ''),
            'fireworks_key' => sanitize_text_field($_POST['fireworks_key'] ?? ''),
            'fireworks_model' => sanitize_text_field($_POST['fireworks_model'] ?? 'accounts/fireworks/models/llama-v3p1-70b-instruct'),
            'kimi_key' => sanitize_text_field($_POST['kimi_key'] ?? ''),
            'kimi_model' => sanitize_text_field($_POST['kimi_model'] ?? 'moonshot-v1-128k'),
            'qwen_key' => sanitize_text_field($_POST['qwen_key'] ?? ''),
            'qwen_model' => sanitize_text_field($_POST['qwen_model'] ?? 'qwen-max'),
            'webhook_secret' => sanitize_text_field($_POST['webhook_secret'] ?? ''),
            'rate_limit_per_hour' => intval($_POST['rate_limit_per_hour'] ?? 100),
            'rate_limit_per_day' => intval($_POST['rate_limit_per_day'] ?? 1000),
            
            'tier_free_screenshots' => sanitize_text_field($_POST['tier_free_screenshots'] ?? 10),
            'tier_dev_screenshots' => sanitize_text_field($_POST['tier_dev_screenshots'] ?? 500),
            'tier_pro_screenshots' => sanitize_text_field($_POST['tier_pro_screenshots'] ?? 2000),
            'tier_agency_screenshots' => sanitize_text_field($_POST['tier_agency_screenshots'] ?? 10000),
            'tier_ent_screenshots' => sanitize_text_field($_POST['tier_ent_screenshots'] ?? '-1'),
            
            'tier_free_critiques' => sanitize_text_field($_POST['tier_free_critiques'] ?? 0),
            'tier_dev_critiques' => sanitize_text_field($_POST['tier_dev_critiques'] ?? 10),
            'tier_pro_critiques' => sanitize_text_field($_POST['tier_pro_critiques'] ?? 50),
            'tier_agency_critiques' => sanitize_text_field($_POST['tier_agency_critiques'] ?? 200),
            'tier_ent_critiques' => sanitize_text_field($_POST['tier_ent_critiques'] ?? '-1'),
            
            'tier_free_ui_reverse' => sanitize_text_field($_POST['tier_free_ui_reverse'] ?? 0),
            'tier_dev_ui_reverse' => sanitize_text_field($_POST['tier_dev_ui_reverse'] ?? 10),
            'tier_pro_ui_reverse' => sanitize_text_field($_POST['tier_pro_ui_reverse'] ?? 25),
            'tier_agency_ui_reverse' => sanitize_text_field($_POST['tier_agency_ui_reverse'] ?? 100),
            'tier_ent_ui_reverse' => sanitize_text_field($_POST['tier_ent_ui_reverse'] ?? '-1'),

            'tier_free_copilot' => sanitize_text_field($_POST['tier_free_copilot'] ?? 0),
            'tier_dev_copilot' => sanitize_text_field($_POST['tier_dev_copilot'] ?? 20),
            'tier_pro_copilot' => sanitize_text_field($_POST['tier_pro_copilot'] ?? 100),
            'tier_agency_copilot' => sanitize_text_field($_POST['tier_agency_copilot'] ?? 500),
            'tier_ent_copilot' => sanitize_text_field($_POST['tier_ent_copilot'] ?? '-1'),
            
            'tier_free_batch' => sanitize_text_field($_POST['tier_free_batch'] ?? 1),
            'tier_dev_batch' => sanitize_text_field($_POST['tier_dev_batch'] ?? 3),
            'tier_pro_batch' => sanitize_text_field($_POST['tier_pro_batch'] ?? 5),
            'tier_agency_batch' => sanitize_text_field($_POST['tier_agency_batch'] ?? 20),
            'tier_ent_batch' => sanitize_text_field($_POST['tier_ent_batch'] ?? 100),
            
            'tier_free_share' => sanitize_text_field($_POST['tier_free_share'] ?? 0),
            'tier_dev_share' => sanitize_text_field($_POST['tier_dev_share'] ?? 1440),
            'tier_pro_share' => sanitize_text_field($_POST['tier_pro_share'] ?? 1440),
            'tier_agency_share' => sanitize_text_field($_POST['tier_agency_share'] ?? 10080),
            'tier_ent_share' => sanitize_text_field($_POST['tier_ent_share'] ?? 43200),
            
            'tier_free_comparison' => isset($_POST['tier_free_comparison']) ? '1' : '',
            'tier_dev_comparison' => isset($_POST['tier_dev_comparison']) ? '1' : '',
            'tier_pro_comparison' => isset($_POST['tier_pro_comparison']) ? '1' : '',
            'tier_agency_comparison' => isset($_POST['tier_agency_comparison']) ? '1' : '',
            'tier_ent_comparison' => isset($_POST['tier_ent_comparison']) ? '1' : '',
            
            'tier_free_batch_enabled' => isset($_POST['tier_free_batch_enabled']) ? '1' : '',
            'tier_dev_batch_enabled' => isset($_POST['tier_dev_batch_enabled']) ? '1' : '',
            'tier_pro_batch_enabled' => isset($_POST['tier_pro_batch_enabled']) ? '1' : '',
            'tier_agency_batch_enabled' => isset($_POST['tier_agency_batch_enabled']) ? '1' : '',
            'tier_ent_batch_enabled' => isset($_POST['tier_ent_batch_enabled']) ? '1' : '',
        ];

        foreach ($settings as $key => $value) {
            update_option('wpuiai_aic_' . $key, $value);
        }

        wp_send_json_success(['message' => 'Settings saved successfully']);
    }

    public function ajax_test_connection(): void {
        check_ajax_referer('wpuiai_aic_settings', 'nonce');

        if (!current_user_can('manage_options')) {
            wp_send_json_error(['message' => 'Permission denied']);
            return;
        }

        $provider = sanitize_text_field($_POST['provider'] ?? '');
        $success = false;
        $message = '';

        switch ($provider) {
            case 'browserless':
                $url = get_option('wpuiai_aic_browserless_url', 'http://127.0.0.1:3005');
                $token = get_option('wpuiai_aic_browserless_token', '');
                
                $headers = [];
                if (!empty($token)) {
                    $headers['Authorization'] = 'Bearer ' . $token;
                }
                
                $response = wp_remote_get($url . '/health', [
                    'timeout' => 10,
                    'headers' => $headers,
                ]);
                
                if (!is_wp_error($response) && wp_remote_retrieve_response_code($response) === 200) {
                    $success = true;
                    $message = 'Browserless connection successful';
                } else {
                    $message = 'Failed to connect to Browserless';
                }
                break;

            case 'posthog':
                $host = get_option('wpuiai_aic_posthog_host', 'https://app.posthog.com');
                $key = get_option('wpuiai_aic_posthog_key', '');
                
                if (empty($host) || empty($key)) {
                    $message = 'PostHog host and key required';
                } else {
                    $response = wp_remote_get($host, [
                        'timeout' => 10,
                    ]);
                    
                    if (!is_wp_error($response)) {
                        $success = true;
                        $message = 'PostHog connection successful';
                    } else {
                        $message = 'Failed to connect to PostHog';
                    }
                }
                break;

            default:
                $message = 'Unknown provider';
        }

        if ($success) {
            wp_send_json_success(['message' => $message]);
        } else {
            wp_send_json_error(['message' => $message]);
        }
    }

    private function get_settings(): array {
        return [
            'default_screenshot_provider' => get_option('wpuiai_aic_default_screenshot_provider', 'go_engine'),
            'default_viewport_width' => get_option('wpuiai_aic_default_viewport_width', 1280),
            'default_viewport_height' => get_option('wpuiai_aic_default_viewport_height', 800),
            'default_format' => get_option('wpuiai_aic_default_format', 'png'),
            'share_expiry_hours' => get_option('wpuiai_aic_share_expiry_hours', 24),
            'max_file_size_mb' => get_option('wpuiai_aic_max_file_size_mb', 10),
            'engine_path' => get_option('wpuiai_aic_engine_path', '/home/wpuiai/uiai-engine'),
            'engine_service_unit' => get_option('wpuiai_aic_engine_service_unit', 'uiai-engine'),
            'cloudflared_service_unit' => get_option('wpuiai_aic_cloudflared_service_unit', 'cloudflared-wpuiai'),
            'cloudflared_config' => get_option('wpuiai_aic_cloudflared_config', '/etc/cloudflared/wpuiai.yml'),
            'cloudflared_log' => get_option('wpuiai_aic_cloudflared_log', '/var/log/cloudflared.log'),
            'browserless_url' => get_option('wpuiai_aic_browserless_url', 'http://127.0.0.1:3005'),
            'browserless_token' => get_option('wpuiai_aic_browserless_token', ''),
            'cloud_api_key' => get_option('wpuiai_aic_cloud_api_key', ''),
            'enable_analytics' => get_option('wpuiai_aic_enable_analytics', '0'),
            'posthog_host' => get_option('wpuiai_aic_posthog_host', 'https://app.posthog.com'),
            'posthog_key' => get_option('wpuiai_aic_posthog_key', ''),
            'anthropic_key' => get_option('wpuiai_aic_anthropic_key', ''),
            'anthropic_model' => get_option('wpuiai_aic_anthropic_model', ''),
            'openrouter_key' => get_option('wpuiai_aic_openrouter_key', ''),
            'openrouter_model' => get_option('wpuiai_aic_openrouter_model', ''),
            'openai_key' => get_option('wpuiai_aic_openai_key', ''),
            'openai_model' => get_option('wpuiai_aic_openai_model', ''),
            'fireworks_key' => get_option('wpuiai_aic_fireworks_key', ''),
            'fireworks_model' => get_option('wpuiai_aic_fireworks_model', ''),
            'kimi_key' => get_option('wpuiai_aic_kimi_key', ''),
            'kimi_model' => get_option('wpuiai_aic_kimi_model', ''),
            'qwen_key' => get_option('wpuiai_aic_qwen_key', ''),
            'qwen_model' => get_option('wpuiai_aic_qwen_model', ''),
            'webhook_secret' => get_option('wpuiai_aic_webhook_secret', ''),
            'rate_limit_per_hour' => get_option('wpuiai_aic_rate_limit_per_hour', 100),
            'rate_limit_per_day' => get_option('wpuiai_aic_rate_limit_per_day', 1000),
            
            'tier_free_screenshots' => get_option('wpuiai_aic_tier_free_screenshots', 10),
            'tier_dev_screenshots' => get_option('wpuiai_aic_tier_dev_screenshots', 500),
            'tier_pro_screenshots' => get_option('wpuiai_aic_tier_pro_screenshots', 2000),
            'tier_agency_screenshots' => get_option('wpuiai_aic_tier_agency_screenshots', 10000),
            'tier_ent_screenshots' => get_option('wpuiai_aic_tier_ent_screenshots', '-1'),
            
            'tier_free_critiques' => get_option('wpuiai_aic_tier_free_critiques', 0),
            'tier_dev_critiques' => get_option('wpuiai_aic_tier_dev_critiques', 10),
            'tier_pro_critiques' => get_option('wpuiai_aic_tier_pro_critiques', 50),
            'tier_agency_critiques' => get_option('wpuiai_aic_tier_agency_critiques', 200),
            'tier_ent_critiques' => get_option('wpuiai_aic_tier_ent_critiques', '-1'),
            
            'tier_free_ui_reverse' => get_option('wpuiai_aic_tier_free_ui_reverse', 0),
            'tier_dev_ui_reverse' => get_option('wpuiai_aic_tier_dev_ui_reverse', 10),
            'tier_pro_ui_reverse' => get_option('wpuiai_aic_tier_pro_ui_reverse', 25),
            'tier_agency_ui_reverse' => get_option('wpuiai_aic_tier_agency_ui_reverse', 100),
            'tier_ent_ui_reverse' => get_option('wpuiai_aic_tier_ent_ui_reverse', '-1'),

            'tier_free_copilot' => get_option('wpuiai_aic_tier_free_copilot', 0),
            'tier_dev_copilot' => get_option('wpuiai_aic_tier_dev_copilot', 20),
            'tier_pro_copilot' => get_option('wpuiai_aic_tier_pro_copilot', 100),
            'tier_agency_copilot' => get_option('wpuiai_aic_tier_agency_copilot', 500),
            'tier_ent_copilot' => get_option('wpuiai_aic_tier_ent_copilot', '-1'),
            
            'tier_free_batch' => get_option('wpuiai_aic_tier_free_batch', 1),
            'tier_dev_batch' => get_option('wpuiai_aic_tier_dev_batch', 3),
            'tier_pro_batch' => get_option('wpuiai_aic_tier_pro_batch', 5),
            'tier_agency_batch' => get_option('wpuiai_aic_tier_agency_batch', 20),
            'tier_ent_batch' => get_option('wpuiai_aic_tier_ent_batch', 100),
            
            'tier_free_share' => get_option('wpuiai_aic_tier_free_share', 0),
            'tier_dev_share' => get_option('wpuiai_aic_tier_dev_share', 1440),
            'tier_pro_share' => get_option('wpuiai_aic_tier_pro_share', 1440),
            'tier_agency_share' => get_option('wpuiai_aic_tier_agency_share', 10080),
            'tier_ent_share' => get_option('wpuiai_aic_tier_ent_share', 43200),
            
            'tier_free_comparison' => get_option('wpuiai_aic_tier_free_comparison', ''),
            'tier_dev_comparison' => get_option('wpuiai_aic_tier_dev_comparison', '1'),
            'tier_pro_comparison' => get_option('wpuiai_aic_tier_pro_comparison', '1'),
            'tier_agency_comparison' => get_option('wpuiai_aic_tier_agency_comparison', '1'),
            'tier_ent_comparison' => get_option('wpuiai_aic_tier_ent_comparison', '1'),
            
            'tier_free_batch_enabled' => get_option('wpuiai_aic_tier_free_batch_enabled', ''),
            'tier_dev_batch_enabled' => get_option('wpuiai_aic_tier_dev_batch_enabled', '1'),
            'tier_pro_batch_enabled' => get_option('wpuiai_aic_tier_pro_batch_enabled', '1'),
            'tier_agency_batch_enabled' => get_option('wpuiai_aic_tier_agency_batch_enabled', '1'),
            'tier_ent_batch_enabled' => get_option('wpuiai_aic_tier_ent_batch_enabled', '1'),
        ];
    }

    // ═══════════════════════════════════════════════════════════
    // Dev Mode AJAX Handlers (A6-A11)
    // ═══════════════════════════════════════════════════════════

    /**
     * A6: Toggle dev mode on/off.
     */
    public function ajax_toggle_dev_mode(): void {
        check_ajax_referer( 'wpuiai_aic_dev_mode_nonce', 'nonce' );
        if ( ! current_user_can( 'manage_options' ) ) {
            wp_send_json_error( 'Permission denied' );
        }

        $enable = ! empty( $_POST['enable'] );

        if ( $enable ) {
            update_option( 'uiai_dev_mode', '1' );
            update_option( 'uiai_use_ai_cloud', '1' );

            // Auto-generate webhook secret if empty
            $secret_set = true;
            if ( empty( get_option( 'wpuiai_aic_webhook_secret', '' ) ) ) {
                $secret = bin2hex( random_bytes( 32 ) );
                update_option( 'wpuiai_aic_webhook_secret', $secret );
                $secret_set = false; // new secret — remind to sync
            }

            wp_send_json_success( [
                'status'            => 'enabled',
                'webhook_secret_set' => $secret_set,
            ] );
        } else {
            update_option( 'uiai_dev_mode', '0' );
            wp_send_json_success( [ 'status' => 'disabled' ] );
        }
    }

    /**
     * A7: Test cloud critique — fires a real AI call through Go engine.
     */
    public function ajax_test_critique(): void {
        check_ajax_referer( 'wpuiai_aic_dev_mode_nonce', 'nonce' );
        if ( ! current_user_can( 'manage_options' ) ) {
            wp_send_json_error( 'Permission denied' );
        }

        $secret = get_option( 'wpuiai_aic_webhook_secret', '' );
        if ( empty( $secret ) ) {
            wp_send_json_error( [ 'message' => 'Webhook secret not configured' ] );
        }

        $url  = 'https://ai.wpuiai.com/api/critique';
        $body = wp_json_encode( [
            'websiteUrl' => home_url(),
            'dimensions' => [ 'layout' ],
        ] );

        $start    = microtime( true );
        $response = wp_remote_post( $url, [
            'method'  => 'POST',
            'timeout' => 60,
            'headers' => [
                'Content-Type'     => 'application/json',
                'X-Webhook-Secret' => $secret,
            ],
            'body'    => $body,
        ] );
        $duration = round( microtime( true ) - $start, 2 );

        if ( is_wp_error( $response ) ) {
            wp_send_json_error( [ 'message' => $response->get_error_message() ] );
        }

        $code     = wp_remote_retrieve_response_code( $response );
        $resp     = json_decode( wp_remote_retrieve_body( $response ), true );
        $model    = $resp['model'] ?? $resp['metadata']['model'] ?? 'unknown';
        $cost     = $resp['cost'] ?? $resp['metadata']['cost'] ?? null;
        $preview  = substr( wp_json_encode( $resp['critique'] ?? $resp ), 0, 200 );

        wp_send_json_success( [
            'http_status'      => $code,
            'model'            => $model,
            'cost'             => $cost,
            'duration_seconds' => $duration,
            'preview'          => $preview,
        ] );
    }

    /**
     * A8: Ecosystem health checks.
     */
    public function ajax_dev_mode_health(): void {
        check_ajax_referer( 'wpuiai_aic_dev_mode_nonce', 'nonce' );
        if ( ! current_user_can( 'manage_options' ) ) {
            wp_send_json_error( 'Permission denied' );
        }

        $cached = get_transient( 'wpuiai_aic_dev_health' );
        if ( $cached ) {
            wp_send_json_success( $cached );
        }

        $secret = get_option( 'wpuiai_aic_webhook_secret', '' );
        $checks = [];

        // Check 1: Go Engine
        $r = wp_remote_get( 'http://127.0.0.1:7456/', [ 'timeout' => 5 ] );
        if ( is_wp_error( $r ) ) {
            $checks['go_engine'] = [ 'status' => 'error', 'message' => $r->get_error_message() ];
        } else {
            $b = json_decode( wp_remote_retrieve_body( $r ), true );
            $checks['go_engine'] = [ 'status' => 'ok', 'version' => $b['version'] ?? 'unknown' ];
        }

        // Check 2: Cloudflare Tunnel
        $r = wp_remote_get( 'https://ai.wpuiai.com/', [ 'timeout' => 10 ] );
        if ( is_wp_error( $r ) ) {
            $checks['tunnel'] = [ 'status' => 'error', 'message' => $r->get_error_message() ];
        } else {
            $code = wp_remote_retrieve_response_code( $r );
            $checks['tunnel'] = ( $code >= 200 && $code < 500 )
                ? [ 'status' => 'ok' ]
                : [ 'status' => 'error', 'http_code' => $code ];
        }

        // Check 3: WP REST ai-settings
        $r = wp_remote_get( rest_url( 'wpuiai-ai-cloud/v1/ai-settings' ), [
            'timeout' => 5,
            'headers' => [ 'X-Webhook-Secret' => $secret ],
        ] );
        if ( is_wp_error( $r ) ) {
            $checks['rest'] = [ 'status' => 'error', 'message' => $r->get_error_message() ];
        } else {
            $code = wp_remote_retrieve_response_code( $r );
            if ( $code === 200 ) {
                $b = json_decode( wp_remote_retrieve_body( $r ), true );
                $checks['rest'] = [ 'status' => 'ok', 'provider' => $b['default_provider'] ?? '', 'model' => $b['default_model'] ?? '' ];
            } else {
                $checks['rest'] = [ 'status' => 'error', 'http_code' => $code ];
            }
        }

        // Check 4: Webhook secret sync
        $r = wp_remote_get( 'http://127.0.0.1:7456/api/health', [
            'timeout' => 5,
            'headers' => [ 'X-Webhook-Secret' => $secret ],
        ] );
        if ( is_wp_error( $r ) ) {
            $checks['webhook'] = [ 'status' => 'error', 'message' => 'Engine unreachable' ];
        } else {
            $code = wp_remote_retrieve_response_code( $r );
            $checks['webhook'] = ( $code === 200 )
                ? [ 'status' => 'ok' ]
                : [ 'status' => 'error', 'message' => 'Secret mismatch (HTTP ' . $code . ')' ];
        }

        // Check 5: Rod pool
        $r = wp_remote_get( 'http://127.0.0.1:7456/api/screenshot/health', [
            'timeout' => 5,
            'headers' => [ 'X-Webhook-Secret' => $secret ],
        ] );
        if ( is_wp_error( $r ) ) {
            $checks['rod'] = [ 'status' => 'error', 'message' => $r->get_error_message() ];
        } else {
            $code = wp_remote_retrieve_response_code( $r );
            if ( $code === 200 ) {
                $b = json_decode( wp_remote_retrieve_body( $r ), true );
                $checks['rod'] = [ 'status' => 'ok', 'pages' => $b['pool_size'] ?? $b['pages'] ?? 0 ];
            } else {
                $checks['rod'] = [ 'status' => 'error', 'http_code' => $code ];
            }
        }

        set_transient( 'wpuiai_aic_dev_health', $checks, 30 );
        wp_send_json_success( $checks );
    }

    /**
     * A9: Rotate webhook secret.
     */
    public function ajax_rotate_webhook_secret(): void {
        check_ajax_referer( 'wpuiai_aic_dev_mode_nonce', 'nonce' );
        if ( ! current_user_can( 'manage_options' ) ) {
            wp_send_json_error( 'Permission denied' );
        }

        $new_secret = bin2hex( random_bytes( 32 ) );
        update_option( 'wpuiai_aic_webhook_secret', $new_secret );

        // Write to env file
        $env_file    = '/etc/wpuiai/ai-api.env';
        $env_written = false;
        if ( is_writable( $env_file ) || ( file_exists( dirname( $env_file ) ) && is_writable( dirname( $env_file ) ) ) ) {
            $contents = file_exists( $env_file ) ? file_get_contents( $env_file ) : '';
            if ( preg_match( '/^WEBHOOK_SECRET=.*/m', $contents ) ) {
                $contents = preg_replace( '/^WEBHOOK_SECRET=.*/m', 'WEBHOOK_SECRET=' . $new_secret, $contents );
            } else {
                $contents .= "\nWEBHOOK_SECRET=" . $new_secret . "\n";
            }
            $env_written = (bool) file_put_contents( $env_file, $contents );
        }

        // Restart Go engine
        $restart_ok = false;
        $output     = [];
        if ( function_exists( 'exec' ) ) {
            exec( 'systemctl restart uiai-engine 2>&1', $output, $code );
            $restart_ok = ( $code === 0 );
        }

        $result = [
            'secret_preview'   => substr( $new_secret, 0, 10 ) . '...',
            'option_updated'   => true,
            'env_written'      => $env_written,
            'engine_restarted' => $restart_ok,
        ];

        // If env write failed, expose full secret for manual copy
        if ( ! $env_written ) {
            $result['full_secret'] = $new_secret;
            $result['env_file']    = $env_file;
        }

        wp_send_json_success( $result );
    }

    /**
     * A10: Clear credit cache.
     */
    public function ajax_clear_credit_cache(): void {
        check_ajax_referer( 'wpuiai_aic_dev_mode_nonce', 'nonce' );
        if ( ! current_user_can( 'manage_options' ) ) {
            wp_send_json_error( 'Permission denied' );
        }

        $had_cache = (bool) get_transient( 'wpuiai_credits_exhausted' );
        delete_transient( 'wpuiai_credits_exhausted' );
        delete_transient( 'wpuiai_cloud_degraded' );
        delete_transient( 'wpuiai_cloud_circuit_breaker' );

        wp_send_json_success( [
            'cleared' => $had_cache,
            'message' => $had_cache ? 'Credit cache cleared — cloud calls will retry' : 'No credit cache was set',
        ] );
    }

    /**
     * A11: Force settings sync (restart Go engine).
     */
    public function ajax_force_settings_sync(): void {
        check_ajax_referer( 'wpuiai_aic_dev_mode_nonce', 'nonce' );
        if ( ! current_user_can( 'manage_options' ) ) {
            wp_send_json_error( 'Permission denied' );
        }

        $output = [];
        $code   = 1;
        if ( function_exists( 'exec' ) ) {
            exec( 'systemctl restart uiai-engine 2>&1', $output, $code );
        }

        $success = ( $code === 0 );
        delete_transient( 'wpuiai_aic_dev_health' );

        wp_send_json_success( [
            'restarted' => $success,
            'message'   => $success
                ? 'Go engine restarted — WP settings will be fetched in ~5 seconds'
                : 'Restart failed: ' . implode( ' ', $output ),
        ] );
    }

    /**
     * A12+A13: Dev Mode JavaScript + inline CSS.
     */
    private function get_dev_mode_js(): string {
        return <<<'JS'
(function($){
    var nonce, ajaxUrl;

    function init() {
        nonce = $('#wpuiai-dev-nonce').val();
        ajaxUrl = wpuiaiAICSettings.ajaxurl;
        if (!nonce) return;

        $('#wpuiai-dev-toggle').on('click', toggleDevMode);
        $('#wpuiai-dev-test-critique').on('click', testCritique);
        $('#wpuiai-dev-health-refresh').on('click', refreshHealth);
        $('#wpuiai-dev-rotate-secret').on('click', rotateSecret);
        $('#wpuiai-dev-clear-cache').on('click', clearCache);
        $('#wpuiai-dev-force-sync').on('click', forceSync);

        // Auto-fire health check when dev mode tab is first shown
        $('a[href="#devmode"]').one('click', function(){ setTimeout(refreshHealth, 300); });
        // If tab is already active (URL hash), fire immediately
        if (window.location.hash === '#devmode') setTimeout(refreshHealth, 500);
    }

    function toggleDevMode() {
        var $btn = $(this);
        var enable = $btn.data('enabled') === 0 || $btn.data('enabled') === '0';
        $btn.prop('disabled', true).text('⏳ ...');
        $.post(ajaxUrl, { action:'wpuiai_aic_toggle_dev_mode', nonce:nonce, enable:enable?1:0 }, function(r){
            if (r.success) location.reload();
            else { $btn.prop('disabled', false).text('Error'); alert(r.data); }
        }).fail(function(){ $btn.prop('disabled', false).text('Error'); });
    }

    function testCritique() {
        var $btn = $('#wpuiai-dev-test-critique');
        var $res = $('#wpuiai-dev-test-result');
        $btn.prop('disabled', true);
        $res.html('<span style="color:#6b7280">🔄 Testing... (up to 60s)</span>');
        $.post(ajaxUrl, { action:'wpuiai_aic_test_critique', nonce:nonce }, function(r){
            $btn.prop('disabled', false);
            if (r.success) {
                var d = r.data;
                var costStr = d.cost ? ' · $' + parseFloat(d.cost).toFixed(4) : '';
                $res.html(
                    '<span style="color:#059669">✅ ' + d.http_status + '</span> — ' +
                    d.model + costStr + ' · ' + d.duration_seconds + 's'
                );
            } else {
                $res.html('<span style="color:#ef4444">❌ ' + (r.data.message || 'Failed') + '</span>');
            }
        }).fail(function(){ $btn.prop('disabled', false); $res.html('<span style="color:#ef4444">❌ Request failed</span>'); });
    }

    function refreshHealth() {
        var $rows = $('#wpuiai-dev-health-rows');
        $rows.find('.health-status').html('<span style="color:#9ca3af">⏳ Checking...</span>');
        $.post(ajaxUrl, { action:'wpuiai_aic_dev_health', nonce:nonce }, function(r){
            if (!r.success) return;
            var d = r.data;
            var checks = {
                go_engine: d.go_engine,
                tunnel: d.tunnel,
                rest: d.rest,
                webhook: d.webhook,
                rod: d.rod
            };
            for (var key in checks) {
                var c = checks[key];
                var $el = $rows.find('[data-check="'+key+'"] .health-status');
                if (c.status === 'ok') {
                    var detail = '';
                    if (c.version) detail = ' v' + c.version;
                    if (c.pages !== undefined) detail = ' (' + c.pages + ' pages)';
                    if (c.provider) detail = ' · ' + c.provider + '/' + (c.model||'');
                    $el.html('<span style="color:#059669">✅ OK' + detail + '</span>');
                } else {
                    var msg = c.message || ('HTTP ' + (c.http_code||'?'));
                    $el.html('<span style="color:#ef4444">❌ ' + msg + '</span>');
                }
            }
        });
    }

    function rotateSecret() {
        if (!confirm('Generate new webhook secret and restart Go engine?')) return;
        var $btn = $('#wpuiai-dev-rotate-secret');
        var $res = $('#wpuiai-dev-rotate-result');
        $btn.prop('disabled', true);
        $.post(ajaxUrl, { action:'wpuiai_aic_rotate_secret', nonce:nonce }, function(r){
            $btn.prop('disabled', false);
            if (r.success) {
                var d = r.data;
                var parts = ['New: ' + d.secret_preview];
                parts.push(d.env_written ? 'env ✅' : 'env ❌ (manual copy needed)');
                parts.push(d.engine_restarted ? 'restart ✅' : 'restart ❌');
                $res.html('<span style="color:#059669">' + parts.join(' · ') + '</span>');
                if (d.full_secret) {
                    $res.append('<br><input type="text" value="'+d.full_secret+'" style="width:100%;font-family:monospace;font-size:11px;margin-top:4px;" readonly onclick="this.select()">');
                }
            } else {
                $res.html('<span style="color:#ef4444">❌ ' + (r.data || 'Failed') + '</span>');
            }
        });
    }

    function clearCache() {
        var $res = $('#wpuiai-dev-cache-result');
        $.post(ajaxUrl, { action:'wpuiai_aic_clear_credit_cache', nonce:nonce }, function(r){
            if (r.success) $res.html('<span style="color:#059669">✅ ' + r.data.message + '</span>');
            else $res.html('<span style="color:#ef4444">❌ Failed</span>');
        });
    }

    function forceSync() {
        var $res = $('#wpuiai-dev-sync-result');
        $res.html('<span style="color:#6b7280">⏳ Restarting...</span>');
        $.post(ajaxUrl, { action:'wpuiai_aic_force_sync', nonce:nonce }, function(r){
            if (r.success) {
                $res.html('<span style="color:#059669">✅ ' + r.data.message + '</span>');
                setTimeout(refreshHealth, 6000);
            } else {
                $res.html('<span style="color:#ef4444">❌ ' + (r.data || 'Failed') + '</span>');
            }
        });
    }

    $(document).ready(init);
})(jQuery);
JS;
    }
}
