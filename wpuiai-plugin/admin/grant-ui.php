<?php
// Spec 173 B1-B2 — WP-Admin Focusa•UIAI Licenses
add_action('admin_menu', function () {
    add_menu_page('Focusa • UIAI Licenses','Focusa • UIAI Licenses','manage_options','focusa-uiai-licenses','focusa_uiai_licenses_page','dashicons-admin-network',31);
});
function focusa_uiai_licenses_page() { echo '<div id="focusa-grant-root"></div>'; }
add_action('admin_enqueue_scripts', function ($hook) {
    if (strpos($hook, 'focusa-uiai-licenses')===false) return;
    wp_enqueue_script('focusa-grant', plugins_url('assets/grant.js', __FILE__), ['wp-api-fetch'], '1.0', true);
});
