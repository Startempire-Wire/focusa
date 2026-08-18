<?php
/**
 * Plugin Name: WPUIAI AI Cloud Admin
 * Plugin URI: https://wpuiai.com/ai-cloud-admin
 * Description: Management dashboard for WPUIAI AI Cloud service with EDD integration, PostHog analytics, and usage tracking.
 * Version: 1.0.29
 * Author: Verious Smith
 * Author URI: https://philoveracity.com
 * License: GPL-2.0+
 * Text Domain: wpuiai-ai-cloud
 * Domain Path: /languages
 * Requires at least: 6.0
 * Requires PHP: 8.0
 */

defined('ABSPATH') || exit;

// Plugin constants
define('WPUIAI_AIC_VERSION', '1.0.29');
define('WPUIAI_AIC_PLUGIN_FILE', __FILE__);
define('WPUIAI_AIC_PLUGIN_DIR', plugin_dir_path(__FILE__));
define('WPUIAI_AIC_PLUGIN_URL', plugin_dir_url(__FILE__));
define('WPUIAI_AIC_BASENAME', plugin_basename(__FILE__));

// Autoloader
spl_autoload_register(function($class) {
    $prefix = 'WPUIAI_AIC_';
    $len = strlen($prefix);

    if (strncmp($prefix, $class, $len) !== 0) {
        return;
    }

    $relative_class = substr($class, $len);
    $file = WPUIAI_AIC_PLUGIN_DIR . 'includes/class-' . strtolower(str_replace('_', '-', $relative_class)) . '.php';

    if (file_exists($file)) {
        require_once $file;
    }
});

// Plugin activation
function wpuiai_aic_activate(): void {
    global $wpdb;

    require_once ABSPATH . 'wp-admin/includes/upgrade.php';

    $charset_collate = $wpdb->get_charset_collate();
    $prefix          = $wpdb->prefix;

    // Read schema SQL (raw SQL file, not PHP)
    $schema_file = WPUIAI_AIC_PLUGIN_DIR . 'database/schema.php';
    if ( file_exists( $schema_file ) ) {
        $sql = file_get_contents( $schema_file );

        // Replace generic wp_ prefix with actual prefix
        // EDD SL creates wp_edd_licensemeta — our schema references it for FK only
        $sql = str_replace( 'wp_edd_license_meta', $prefix . 'edd_licensemeta', $sql );
        $sql = str_replace( 'wp_uiai_', $prefix . 'uiai_', $sql );
        $sql = str_replace( 'wp_edd_licenses', $prefix . 'edd_licenses', $sql );

        // Strip SQL comments (dbDelta doesn't handle them)
        $sql = preg_replace( '/--[^\n]*/', '', $sql );

        // Split into individual statements
        $queries = array_filter( array_map( 'trim', explode( ';', $sql ) ) );
        foreach ( $queries as $query ) {
            if ( stripos( $query, 'CREATE TABLE' ) !== false ) {
                dbDelta( $query . ';' );
            } elseif ( stripos( $query, 'CREATE INDEX' ) !== false ) {
                // dbDelta doesn't support standalone CREATE INDEX — run directly
                $wpdb->query( $query );
            }
        }
    }

    flush_rewrite_rules();

    // Set default options
    add_option( 'wpuiai_aic_version', WPUIAI_AIC_VERSION );
    add_option( 'wpuiai_aic_installed_date', current_time( 'mysql' ) );
}

// Plugin deactivation
function wpuiai_aic_deactivate(): void {
    flush_rewrite_rules();

    // Clear scheduled cron jobs
    wp_clear_scheduled_hook('wpuiai_aic_daily_stats');
    wp_clear_scheduled_hook('wpuiai_aic_cleanup_expired_shares');
    wp_clear_scheduled_hook('wpuiai_aic_sync_posthog_events');
    WPUIAI_AIC_Health_Monitor::deactivate();
}

// Plugin initialization
function wpuiai_aic_init(): void {
    // Always load Settings for REST API (needed by Bun server)
    WPUIAI_AIC_Settings::instance();

    // Health monitor runs via WP-Cron (must be outside is_admin check)
    WPUIAI_AIC_Health_Monitor::instance();

    // Always load Keys API for CLI tool - register routes on rest_api_init
    add_action('rest_api_init', function() {
        WPUIAI_AIC_Service_Manager::instance()->register_rest_routes();
        WPUIAI_AIC_Keys_API::instance()->register_routes();
    }, 5);

    // Always load Service Manager for REST API support
    WPUIAI_AIC_Service_Manager::instance();

    // Deprecation headers for legacy REST namespace
    add_filter('rest_post_dispatch', 'wpuiai_aic_add_deprecation_headers', 10, 3);

    // No-cache headers for Services page and Dashboard (real-time updates)
    add_action('admin_head-admin.php', function() {
        global $pagenow;
        $page = $_GET['page'] ?? '';
        if ($pagenow === 'admin.php' && ($page === 'wpuiai-ai-cloud-services' || $page === 'wpuiai-ai-cloud')) {
            nocache_headers();
            header('Cache-Control: no-cache, no-store, must-revalidate, max-age=0');
            header('Pragma: no-cache');
            header('Expires: 0');
            // LiteSpeed cache control
            if (function_exists('lscache_flush')) {
                header('X-LiteSpeed-Cache-Control: no-cache');
            }
        }
    });

    if (!is_admin()) {
        return;
    }


    WPUIAI_AIC_Admin_Dashboard::instance();
    WPUIAI_AIC_Client_Keys::instance();
    WPUIAI_AIC_EDD_Integration::instance();
    WPUIAI_AIC_PostHog_Tracking::instance();
    WPUIAI_AIC_Usage_Analytics::instance();
    WPUIAI_AIC_Licenses::instance();
    WPUIAI_AIC_Revenue::instance();
    WPUIAI_AIC_Service_Health::instance();
}

/**
 * Add deprecation headers to legacy REST namespace responses.
 *
 * The canonical namespace is wpuiai-ai-cloud/v1. Any additional namespaces
 * registered via the wpuiai_aic_rest_namespaces filter are considered legacy
 * and receive Deprecation + Sunset headers so callers can migrate.
 *
 * @param WP_HTTP_Response $response Result to send to the client.
 * @param WP_REST_Server   $server   Server instance.
 * @param WP_REST_Request  $request  Request used to generate the response.
 * @return WP_HTTP_Response
 */
function wpuiai_aic_add_deprecation_headers( $response, $server, $request ) {
    $route = $request->get_route();

    // Only tag routes outside the canonical namespace
    $canonical = '/wpuiai-ai-cloud/v1';
    if ( strpos( $route, $canonical ) === 0 ) {
        return $response;
    }

    // Check if the route belongs to any of our registered (legacy) namespaces
    $namespaces = apply_filters( 'wpuiai_aic_rest_namespaces', [ 'wpuiai-ai-cloud/v1' ] );
    foreach ( $namespaces as $ns ) {
        if ( $ns === 'wpuiai-ai-cloud/v1' ) {
            continue; // skip canonical
        }
        if ( strpos( $route, '/' . $ns ) === 0 ) {
            $response->header( 'Deprecation', 'true' );
            $response->header( 'Sunset', '2027-01-01T00:00:00Z' );
            $response->header( 'Link', '<https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1>; rel="successor-version"' );
            break;
        }
    }

    return $response;
}

// Global responsive patch for ALL AIC admin screens (1440/1024/768/375) — must load AFTER page-specific CSS
add_action('admin_enqueue_scripts', function($hook) {
    $page = $_GET['page'] ?? '';
    $is_aic = str_starts_with($page, 'wpuiai-ai-cloud') || str_starts_with($page, 'focusa-uiai-licenses') || str_contains($hook, 'wpuiai');
    if (!$is_aic && $hook !== 'toplevel_page_wpuiai-ai-cloud') return;
    // Enqueue after individual page CSS (priority 99 vs default 10)
    $ver = WPUIAI_AIC_VERSION . '.' . @filemtime(WPUIAI_AIC_PLUGIN_DIR . 'assets/css/aic-responsive-global.css');
    wp_enqueue_style('wpuiai-aic-responsive-global', WPUIAI_AIC_PLUGIN_URL . 'assets/css/aic-responsive-global.css', [], $ver);
}, 99);

// Register admin menu
add_action('admin_menu', function() {
    add_menu_page(
        'AI Cloud',
        'AI Cloud',
        'manage_options',
        'wpuiai-ai-cloud',
        'wpuiai_aic_render_dashboard',
        'dashicons-cloud',
        30
    );

    add_submenu_page(
        'wpuiai-ai-cloud',
        'AI Cloud Dashboard',
        'Dashboard',
        'manage_options',
        'wpuiai-ai-cloud',
        'wpuiai_aic_render_dashboard'
    );

    add_submenu_page(
        'wpuiai-ai-cloud',
        'AI Cloud Licenses',
        'Licenses',
        'manage_options',
        'wpuiai-ai-cloud-licenses',
        'wpuiai_aic_render_licenses'
    );

    add_submenu_page(
        'wpuiai-ai-cloud',
        'AI Cloud Usage Analytics',
        'Usage Analytics',
        'manage_options',
        'wpuiai-ai-cloud-usage',
        'wpuiai_aic_render_usage'
    );

    add_submenu_page(
        'wpuiai-ai-cloud',
        'AI Cloud Client Keys',
        'Client Keys',
        'manage_options',
        'wpuiai-ai-cloud-client-keys',
        'wpuiai_aic_render_client_keys'
    );

    add_submenu_page(
        'wpuiai-ai-cloud',
        'AI Cloud Revenue',
        'Revenue',
        'manage_options',
        'wpuiai-ai-cloud-revenue',
        'wpuiai_aic_render_revenue'
    );

    add_submenu_page(
        'wpuiai-ai-cloud',
        'AI Cloud Service Health',
        'Service Health',
        'manage_options',
        'wpuiai-ai-cloud-health',
        'wpuiai_aic_render_health'
    );

    add_submenu_page(
        'wpuiai-ai-cloud',
        'AI Cloud Services',
        'Services',
        'manage_options',
        'wpuiai-ai-cloud-services',
        'wpuiai_aic_render_services'
    );

    add_submenu_page(
        'wpuiai-ai-cloud',
        'AI Cloud Settings',
        'Settings',
        'manage_options',
        'wpuiai-ai-cloud-settings',
        'wpuiai_aic_render_settings'
    );

    add_submenu_page(
        'wpuiai-ai-cloud',
        'AI Cloud Training Jobs',
        'Training Jobs',
        'manage_options',
        'wpuiai-ai-cloud-training',
        'wpuiai_aic_render_training_jobs'
    );

    add_submenu_page(
        'wpuiai-ai-cloud',
        'AI Cloud Evaluations',
        'Evaluations',
        'manage_options',
        'wpuiai-ai-cloud-evaluations',
        'wpuiai_aic_render_evaluations'
    );

    add_submenu_page(
        'wpuiai-ai-cloud',
        'AI Cloud Model Registry',
        'Model Registry',
        'manage_options',
        'wpuiai-ai-cloud-models',
        'wpuiai_aic_render_models'
    );

    add_submenu_page(
        'wpuiai-ai-cloud',
        'PostHog Analytics',
        'PostHog Analytics',
        'manage_options',
        'wpuiai-ai-cloud-posthog',
        'wpuiai_aic_render_posthog_analytics'
    );

    add_submenu_page(
        'wpuiai-ai-cloud',
        'Algorithm Parity',
        'Algorithm Parity',
        'manage_options',
        'wpuiai-ai-cloud-parity',
        'wpuiai_aic_render_algorithm_parity'
    );
});

// Render functions
function wpuiai_aic_render_dashboard(): void {
    WPUIAI_AIC_Admin_Dashboard::instance()->render();
}

function wpuiai_aic_render_licenses(): void {
    WPUIAI_AIC_Licenses::instance()->render_page();
}

function wpuiai_aic_render_usage(): void {
    WPUIAI_AIC_Usage_Analytics::instance()->render_page();
}

function wpuiai_aic_render_client_keys(): void {
    WPUIAI_AIC_Client_Keys::instance()->render_page();
}

function wpuiai_aic_render_revenue(): void {
    WPUIAI_AIC_Revenue::instance()->render_page();
}

function wpuiai_aic_render_health(): void {
    WPUIAI_AIC_Service_Health::instance()->render_page();
}

function wpuiai_aic_render_services(): void {
    WPUIAI_AIC_Service_Manager::instance()->render_page();
}

function wpuiai_aic_render_settings(): void {
    WPUIAI_AIC_Settings::instance()->render_page();
}

function wpuiai_aic_render_training_jobs(): void {
    WPUIAI_AIC_Training_Jobs::instance()->render();
}

function wpuiai_aic_render_evaluations(): void {
    WPUIAI_AIC_Evaluation_Results::instance()->render();
}

function wpuiai_aic_render_models(): void {
    WPUIAI_AIC_Model_Registry::instance()->render();
}

function wpuiai_aic_render_posthog_analytics(): void {
    WPUIAI_AIC_PostHog_Analytics::instance()->render_page();
}

function wpuiai_aic_render_algorithm_parity(): void {
    WPUIAI_AIC_Algorithm_Parity::instance()->render();
}

// Register hooks
register_activation_hook(__FILE__, 'wpuiai_aic_activate');
register_deactivation_hook(__FILE__, 'wpuiai_aic_deactivate');
add_action('plugins_loaded', 'wpuiai_aic_init');

// ─── Sync AIC AI settings → canonical uiai_* options ───
// AIC is server infra; the main plugin (uiai_*) is the single source of truth.
// When AIC settings are saved, mirror AI keys/models to canonical options
// so the main plugin always finds them.
$_aic_ai_sync_map = [
    'wpuiai_aic_openrouter_key'   => 'uiai_openrouter_key',
    'wpuiai_aic_openrouter_model' => 'uiai_openrouter_model',
    'wpuiai_aic_anthropic_key'    => 'uiai_anthropic_key',
    'wpuiai_aic_anthropic_model'  => 'uiai_anthropic_model',
    'wpuiai_aic_openai_key'       => 'uiai_openai_key',
    'wpuiai_aic_openai_model'     => 'uiai_openai_model',
];
foreach ( $_aic_ai_sync_map as $aic_opt => $canonical_opt ) {
    add_action( "update_option_{$aic_opt}", function( $old, $new ) use ( $canonical_opt ) {
        update_option( $canonical_opt, $new );
    }, 10, 2 );
}
unset( $_aic_ai_sync_map );

// PostHog sync cron
add_action('wpuiai_aic_sync_posthog_events', ['WPUIAI_AIC_PostHog_Tracking', 'sync_buffered_events']);

// ─── Credit refresh cron: grant monthly credits to active licenses ───
add_action( 'wpuiai_aic_credit_refresh', function() {
    if ( ! class_exists( 'WPUIAI_Credit_Service' ) || ! class_exists( 'EDD_Software_Licensing' ) ) {
        return;
    }

    global $wpdb;
    $credit_svc = WPUIAI_Credit_Service::instance();

    // Find all active licenses
    $licenses = $wpdb->get_results(
        "SELECT l.id AS license_id, l.download_id, b.last_grant_at
         FROM {$wpdb->prefix}edd_licenses l
         LEFT JOIN {$wpdb->prefix}uiai_credit_balances b ON b.license_id = l.id
         WHERE l.status = 'active'"
    );

    $refreshed = 0;
    $edd_int = WPUIAI_AIC_EDD_Integration::instance();

    foreach ( $licenses as $lic ) {
        // Skip if granted within last 29 days (buffer for timing)
        if ( $lic->last_grant_at && strtotime( $lic->last_grant_at ) > strtotime( '-29 days' ) ) {
            continue;
        }

        // Resolve tier from download ID
        $tier = $edd_int->get_tier_from_download_public( (int) $lic->download_id );
        if ( ! $tier ) {
            continue;
        }

        $credit_svc->grant_monthly( (int) $lic->license_id, $tier );
        $refreshed++;
    }

    if ( $refreshed > 0 ) {
        error_log( "[wpuiai-credits] Refreshed credits for {$refreshed} licenses" );
    }

    // Also expire old credits
    $credit_svc->expire_credits();
} );

// Schedule the credit refresh cron (daily)
if ( ! wp_next_scheduled( 'wpuiai_aic_credit_refresh' ) ) {
    wp_schedule_event( time(), 'daily', 'wpuiai_aic_credit_refresh' );
}

// ─── Credit pack purchase → grant credits ───
add_action( 'edd_complete_purchase', function( $payment_id ) {
    if ( ! class_exists( 'WPUIAI_Credit_Service' ) ) {
        return;
    }

    $credit_packs = [
        455 => 100,   // Boost
        456 => 500,   // Power
        457 => 2000,  // Mega
    ];

    // LTD products → map to tier for credit grants
    $ltd_tiers = [
        452 => 'starter',   // Starter LTD
        453 => 'pro',       // Pro LTD
        454 => 'agency',    // Agency LTD
    ];

    $payment = edd_get_payment( $payment_id );
    if ( ! $payment ) {
        return;
    }

    $cart = $payment->cart_details;
    $credit_svc = WPUIAI_Credit_Service::instance();

    foreach ( $cart as $item ) {
        $download_id = (int) ( $item['id'] ?? 0 );

        // Credit pack: grant credits to the customer's first active license
        if ( isset( $credit_packs[ $download_id ] ) ) {
            $amount = $credit_packs[ $download_id ];
            $customer_email = $payment->email;
            // Find any active license for this customer
            global $wpdb;
            $license_id = (int) $wpdb->get_var( $wpdb->prepare(
                "SELECT id FROM {$wpdb->prefix}edd_licenses WHERE email = %s AND status = 'active' ORDER BY id DESC LIMIT 1",
                $customer_email
            ) );
            if ( $license_id > 0 ) {
                $credit_svc->topup( $license_id, (float) $amount, (string) $payment_id );
                error_log( "[wpuiai-credits] Credit pack: granted {$amount} credits to license {$license_id} (payment {$payment_id})" );
            }
        }

        // LTD: grant initial monthly credits
        if ( isset( $ltd_tiers[ $download_id ] ) ) {
            $tier = $ltd_tiers[ $download_id ];
            // License is created by EDD SL on purchase — find it
            global $wpdb;
            $license_id = (int) $wpdb->get_var( $wpdb->prepare(
                "SELECT id FROM {$wpdb->prefix}edd_licenses WHERE download_id = %d AND payment_id = %d LIMIT 1",
                $download_id, $payment_id
            ) );
            if ( $license_id > 0 ) {
                $credit_svc->grant_monthly( $license_id, $tier );
                error_log( "[wpuiai-credits] LTD: initial grant for license {$license_id} tier={$tier} (payment {$payment_id})" );
            }
        }
    }
} );

// Allow our service manager nonces to work with REST API without requiring cookies
add_filter("rest_authentication_errors", function($result) {
    if ($result === true || is_wp_error($result)) {
        return $result;
    }
    
    $request_uri = $_SERVER["REQUEST_URI"] ?? "";
    if (strpos($request_uri, "wpuiai-ai-cloud/v1") !== false) {
        $nonce = $_SERVER["HTTP_X_WP_NONCE"] ?? $_REQUEST["_wpnonce"] ?? "";
        if (!empty($nonce) && wp_verify_nonce($nonce, "wpuiai_aic_service_manager")) {
            return true;
        }
    }
    
    return $result;
});

if (defined('WP_CLI') && WP_CLI) {
    require_once WPUIAI_AIC_PLUGIN_DIR . 'cli/wp-cli-commands.php';
    if (class_exists('WPCLI_AIC_Keys')) {
        WP_CLI::add_command('aic key', 'WPCLI_AIC_Keys');
        WP_CLI::add_command('aic key', 'WPCLI_AIC_Keys');
    }
}

// V1+V2+V3 — Focusa license production hardening. Loaded after the
// existing classes so it can hook into rest_api_init, edd_complete_purchase,
// and the existing rest_validate_license handler.
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-focusa-license-production.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-focusa-email-templates.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-focusa-activation-handler.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-focusa-management-panel.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-verification-challenge.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-registration-service.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-checkout-intent.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-authority-outbox.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-reconciliation.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-child-token-broker.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-facade-presenter.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-key-quarantine.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-migration-canary.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-terminal-delivery.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-node-registry.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-recovery-gate.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-legacy-migration.php';
WPUIAI_AIC_Focusa_License_Production::install();
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-license-payment-plan.php';
require_once WPUIAI_AIC_PLUGIN_DIR . 'includes/class-admin-license-grant.php';
