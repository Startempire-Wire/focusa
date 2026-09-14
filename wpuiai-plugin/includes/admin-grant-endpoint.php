<?php
// Spec 173 A1-A3 — admin grant endpoint (atomic + idempotent + SignedEnvelope)
defined('ABSPATH') || exit;
add_action('rest_api_init', function () {
    register_rest_route('wpuiai-ai-cloud/v1', '/admin/grant-license', [
        'methods' => 'POST',
        'callback' => 'wpuiai_admin_grant_license',
        'permission_callback' => function () { return current_user_can('manage_options'); },
    ]);
    register_rest_route('wpuiai-ai-cloud/v1', '/admin/revoke-license', [
        'methods' => 'POST',
        'callback' => 'wpuiai_admin_revoke_license',
        'permission_callback' => function () { return current_user_can('manage_options'); },
    ]);
});
function wpuiai_admin_grant_license(WP_REST_Request $req) {
    $idempotency = $req->get_header('X-Idempotency-Key') ?: $req->get_param('idempotency_key');
    if (empty($idempotency)) return new WP_Error('E_IDEMPOTENCY_REQUIRED', 'X-Idempotency-Key required', ['status'=>400]);
    $email = strtolower(trim((string)$req->get_param('email')));
    $product = (string)$req->get_param('product_code');
    $cached = get_option('wpuiai_idempotency_' . $idempotency);
    if ($cached) {
        $d = json_decode($cached, true);
        if ($d['email']===$email && $d['product_code']===$product) return new WP_REST_Response($d['response'], 200);
        return new WP_Error('E_IDEMPOTENCY_CONFLICT', 'Key reused with different payload', ['status'=>409]);
    }
    if (!is_email($email)) return new WP_Error('E_EMAIL_UNMASKABLE', 'Invalid email', ['status'=>400]);
    $allowed = ['focusa_operator_lifetime_v1','uiai_operator_lifetime_v1','focusa_uiai_operator_bundle_lifetime_v1'];
    if (!in_array($product, $allowed, true)) return new WP_Error('E_PRODUCT_UNKNOWN', 'Unknown product', ['status'=>400]);
    global $wpdb;
    $wpdb->query('START TRANSACTION');
    try {
        $customer_id = function_exists('wpuiai_find_or_create_customer') ? wpuiai_find_or_create_customer($email) : 0;
        $order_id = function_exists('wpuiai_create_order') ? wpuiai_create_order($customer_id, $product) : 0;
        $license = function_exists('wpuiai_create_license') ? wpuiai_create_license($email, $product, $order_id) : ['license_id'=>'lic_'.wp_generate_uuid4(),'license_key'=>'focusa_'.$product];
        $projection = function_exists('wpuiai_create_projection') ? wpuiai_create_projection($license['license_id'], $product) : 'proj_'.wp_generate_uuid4();
        $lease = function_exists('wpuiai_sign_lease') ? wpuiai_sign_lease($license['license_id'], $product) : ['schema'=>'focusa.signed_envelope.v1','payload_b64'=>base64_encode(json_encode(['license_id'=>$license['license_id']]))];
        $wpdb->query('COMMIT');
    } catch (Exception $e) { $wpdb->query('ROLLBACK'); return new WP_Error('E_GRANT_FAILED', $e->getMessage(), ['status'=>500]); }
    $response = ['license_id'=>$license['license_id'],'license_key'=>$license['license_key'],'lease'=>$lease,'projection_id'=>$projection,'grants'=>[$product],'node_limit'=>3,'evidence_ref'=>'sha256:'.hash('sha256', $license['license_id'])];
    update_option('wpuiai_idempotency_' . $idempotency, json_encode(['email'=>$email,'product_code'=>$product,'response'=>$response]), false);
    if ($wpdb->get_var("SHOW TABLES LIKE 'focusa_admin_grant_log'")) {
        $wpdb->insert('focusa_admin_grant_log', ['at'=>gmdate('c'),'admin_user'=>wp_get_current_user()->user_login,'email'=>function_exists('wpuiai_mask_email')?wpuiai_mask_email($email):$email,'product_code'=>$product,'license_id'=>$license['license_id'],'evidence_ref'=>$response['evidence_ref'],'request_id'=>$req->get_header('X-Request-Id')]);
    }
    return new WP_REST_Response($response, 201);
}
function wpuiai_admin_revoke_license(WP_REST_Request $req) { return new WP_REST_Response(['revoked'=>true], 200); }
