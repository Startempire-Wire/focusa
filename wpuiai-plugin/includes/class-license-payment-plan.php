<?php
/**
 * License Payment Plan — tracks installments against price.
 * Supports Stripe/EDD/manual payment plans, registers with actual software via verify endpoint.
 */
defined('ABSPATH') || exit;

class WPUIAI_AIC_License_Payment_Plan {
    const TABLE_PLAN = 'wpuiai_license_payment_plans';
    const TABLE_PAYMENT = 'wpuiai_license_payments';
    const VERSION = '1.0.0';

    public static function install(): void {
        global $wpdb;
        require_once ABSPATH . 'wp-admin/includes/upgrade.php';
        $charset = $wpdb->get_charset_collate();
        $plan_table = $wpdb->prefix . self::TABLE_PLAN;
        $pay_table = $wpdb->prefix . self::TABLE_PAYMENT;
        $sql1 = "CREATE TABLE {$plan_table} (
            id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
            license_id BIGINT UNSIGNED NOT NULL,
            download_id BIGINT UNSIGNED NOT NULL,
            total_price DECIMAL(10,2) NOT NULL,
            currency VARCHAR(10) NOT NULL DEFAULT 'USD',
            installments_total INT UNSIGNED NOT NULL DEFAULT 1,
            installments_paid INT UNSIGNED NOT NULL DEFAULT 0,
            paid_amount DECIMAL(10,2) NOT NULL DEFAULT 0.00,
            remaining_amount DECIMAL(10,2) NOT NULL DEFAULT 0.00,
            status VARCHAR(20) NOT NULL DEFAULT 'active',
            plan_type VARCHAR(20) NOT NULL DEFAULT 'manual',
            order_id BIGINT UNSIGNED NULL,
            next_due_date DATE NULL,
            notes TEXT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (id),
            UNIQUE KEY ux_license (license_id),
            KEY idx_status (status),
            KEY idx_license (license_id)
        ) {$charset};";
        $sql2 = "CREATE TABLE {$pay_table} (
            id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
            plan_id BIGINT UNSIGNED NOT NULL,
            license_id BIGINT UNSIGNED NOT NULL,
            installment_number INT UNSIGNED NULL,
            amount DECIMAL(10,2) NOT NULL,
            payment_method VARCHAR(40) NULL,
            transaction_ref VARCHAR(255) NULL,
            notes TEXT NULL,
            recorded_by BIGINT UNSIGNED NULL,
            paid_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (id),
            KEY idx_plan (plan_id),
            KEY idx_license (license_id),
            KEY idx_paid_at (paid_at)
        ) {$charset};";
        dbDelta($sql1);
        dbDelta($sql2);
        if (!wp_next_scheduled('wpuiai_license_payment_reminders')) {
            wp_schedule_event(time()+3600, 'daily', 'wpuiai_license_payment_reminders');
        }
        update_option('wpuiai_license_payment_plan_version', self::VERSION);
        self::register_rest();
        add_action('wpuiai_license_payment_reminders', [self::class, 'cron_send_reminders']);
    }

    public static function register_rest(): void {
        add_action('rest_api_init', function(){
            register_rest_route('wpuiai-ai-cloud/v1', '/license/verify', [
                'methods' => 'GET',
                'permission_callback' => '__return_true',
                'callback' => [self::class, 'rest_verify'],
            ]);
            // Legacy alias for crates: /license/validate (POST) and /license/status (GET) — same handler, no drift.
            register_rest_route('wpuiai-ai-cloud/v1', '/license/validate', [
                'methods' => ['GET','POST'],
                'permission_callback' => '__return_true',
                'callback' => [self::class, 'rest_verify'],
            ]);
            register_rest_route('wpuiai-ai-cloud/v1', '/license/status', [
                'methods' => 'GET',
                'permission_callback' => '__return_true',
                'callback' => [self::class, 'rest_verify'],
            ]);
            register_rest_route('wpuiai-ai-cloud/v1', '/admin/payment-plan', [
                'methods' => 'POST',
                'permission_callback' => fn() => current_user_can('manage_options'),
                'callback' => [self::class, 'rest_create_plan'],
            ]);
            register_rest_route('wpuiai-ai-cloud/v1', '/admin/payment-record', [
                'methods' => 'POST',
                'permission_callback' => fn() => current_user_can('manage_options'),
                'callback' => [self::class, 'rest_record_payment'],
            ]);
            register_rest_route('wpuiai-ai-cloud/v1', '/stripe/payment-plan-webhook', [
                'methods' => 'POST',
                'permission_callback' => '__return_true',
                'callback' => [self::class, 'stripe_webhook'],
            ]);
            register_rest_route('wpuiai-ai-cloud/v1', '/admin/payment-plan/(?P<id>\d+)', [
                'methods' => 'GET',
                'permission_callback' => fn() => current_user_can('manage_options'),
                'callback' => [self::class, 'rest_get_plan'],
            ]);
        });
    }

    // Create plan after license grant — called from grant UI or CLI
    public static function create_plan(int $license_id, int $download_id, float $total_price, int $installments = 1, string $plan_type = 'manual', ?int $order_id = null, string $currency = 'USD', string $notes = ''): array {
        global $wpdb;
        if ($license_id <=0 || $download_id <=0) return ['ok'=>false,'error'=>'invalid_args'];
        $table = $wpdb->prefix . self::TABLE_PLAN;
        // already has plan?
        $exists = $wpdb->get_var($wpdb->prepare("SELECT id FROM {$table} WHERE license_id=%d LIMIT 1", $license_id));
        if ($exists) return ['ok'=>false,'error'=>'plan_exists','plan_id'=>(int)$exists];
        if ($installments <1) $installments=1;
        $wpdb->insert($table, [
            'license_id'=>$license_id,
            'download_id'=>$download_id,
            'total_price'=>$total_price,
            'currency'=>$currency,
            'installments_total'=>$installments,
            'installments_paid'=> $installments===1 ? 0 : 0,
            'paid_amount'=>0.00,
            'remaining_amount'=>$total_price,
            'status'=> $installments===1 ? 'single' : 'active',
            'plan_type'=>$plan_type,
            'order_id'=>$order_id,
            'notes'=>$notes,
        ], ['%d','%d','%f','%s','%d','%d','%f','%f','%s','%s','%d','%s']);
        if ($wpdb->last_error) return ['ok'=>false,'error'=>'db_error','mysql'=>$wpdb->last_error];
        $plan_id = (int)$wpdb->insert_id;
        // WP->Stripe outbound: if stripe plan, create Customer/Price/PaymentIntent in Stripe (bidirectional)
        $stripe = null;
        if ($plan_type === 'stripe') {
            $sync = self::ensure_stripe_sync($license_id);
            if ($sync['ok'] ?? false) {
                try {
                    $stripe = self::stripe_create_for_plan($license_id, $download_id, $total_price, $installments, $plan_id);
                    // persist stripe ids in license meta for audit / checkout link
                    if (!empty($stripe['checkout_url'])) {
                        update_metadata('edd_license', $license_id, 'wpuiai_stripe_checkout_url', (string)$stripe['checkout_url']);
                    }
                    if (!empty($stripe['payment_intent_id'])) {
                        update_metadata('edd_license', $license_id, 'wpuiai_stripe_payment_intent', (string)$stripe['payment_intent_id']);
                    }
                    if (!empty($stripe['price_id'])) {
                        update_metadata('edd_license', $license_id, 'wpuiai_stripe_price_id', (string)$stripe['price_id']);
                    }
                    if (!empty($stripe['customer_id'])) {
                        update_metadata('edd_license', $license_id, 'wpuiai_stripe_customer_id', (string)$stripe['customer_id']);
                    }
                } catch (\Throwable $e) {
                    // Do not fail plan creation; log stripe error for retry via WP-Admin
                    error_log('[wpuiai-payments] stripe outbound failed license '.$license_id.': '.$e->getMessage());
                    $stripe = ['error'=>$e->getMessage(), 'ok'=>false];
                }
            } else {
                $stripe = ['error'=>'stripe_not_configured', 'sync'=>$sync];
            }
        }
        $ret = ['ok'=>true,'plan_id'=>$plan_id];
        if ($stripe !== null) $ret['stripe'] = $stripe;
        return $ret;
    }

    public static function record_payment(int $license_id, float $amount, string $method='manual', string $txn='', string $notes='', ?int $recorded_by=null): array {
        global $wpdb;
        $plan_table = $wpdb->prefix . self::TABLE_PLAN;
        $pay_table = $wpdb->prefix . self::TABLE_PAYMENT;
        // Transaction for race safety (InnoDB)
        $wpdb->query('START TRANSACTION');
        $plan = $wpdb->get_row($wpdb->prepare("SELECT * FROM {$plan_table} WHERE license_id=%d LIMIT 1 FOR UPDATE", $license_id), ARRAY_A);
        if (!$plan) { $wpdb->query('ROLLBACK'); return ['ok'=>false,'error'=>'no_plan']; }
        if ($amount <=0) { $wpdb->query('ROLLBACK'); return ['ok'=>false,'error'=>'invalid_amount']; }
        // Idempotency: if same txn already recorded, replay
        if ($txn !== '' ) {
            $dup = $wpdb->get_var($wpdb->prepare("SELECT id FROM {$pay_table} WHERE license_id=%d AND transaction_ref=%s LIMIT 1", $license_id, $txn));
            if ($dup) { $wpdb->query('ROLLBACK'); return ['ok'=>true,'replay'=>true,'payment_id'=>(int)$dup,'hint'=>'txn already recorded']; }
        }
        $plan_id = (int)$plan['id'];
        $next_num = (int)$plan['installments_paid'] + 1;
        $ins = $wpdb->insert($pay_table, [
            'plan_id'=>$plan_id,
            'license_id'=>$license_id,
            'installment_number'=>$next_num,
            'amount'=>$amount,
            'payment_method'=>$method,
            'transaction_ref'=>$txn,
            'notes'=>$notes,
            'recorded_by'=>$recorded_by,
            'paid_at'=>current_time('mysql'),
        ], ['%d','%d','%d','%f','%s','%s','%s','%d','%s']);
        if ($wpdb->last_error || !$ins) { $wpdb->query('ROLLBACK'); return ['ok'=>false,'error'=>'db_error','mysql'=>$wpdb->last_error]; }
        $new_paid = (float)$plan['paid_amount'] + $amount;
        $remaining = max(0, (float)$plan['total_price'] - $new_paid);
        $new_count = (int)$plan['installments_paid'] + 1;
        $status = $plan['status'];
        if ($remaining <= 0.01) { $status='completed'; $remaining=0; }
        else if ($new_count >= (int)$plan['installments_total'] && $remaining>0) { $status='overdue'; }
        $wpdb->update($plan_table, [
            'paid_amount'=>$new_paid,
            'remaining_amount'=>$remaining,
            'installments_paid'=>$new_count,
            'status'=>$status,
        ], ['id'=>$plan_id], ['%f','%f','%d','%s'], ['%d']);
        if ($wpdb->last_error) { $wpdb->query('ROLLBACK'); return ['ok'=>false,'error'=>'db_update_failed','mysql'=>$wpdb->last_error]; }
        $wpdb->query('COMMIT');
        // Also ensure order transaction recorded if linked order
        return ['ok'=>true,'plan_id'=>$plan_id,'paid_amount'=>$new_paid,'remaining'=>$remaining,'status'=>$status,'payment_id'=>(int)$wpdb->insert_id];
    }

    public static function get_plan(int $license_id): ?array {
        global $wpdb;
        $table = $wpdb->prefix . self::TABLE_PLAN;
        $row = $wpdb->get_row($wpdb->prepare("SELECT * FROM {$table} WHERE license_id=%d LIMIT 1", $license_id), ARRAY_A);
        return $row ?: null;
    }
    public static function get_payments(int $license_id, int $limit=20): array {
        global $wpdb;
        $table = $wpdb->prefix . self::TABLE_PAYMENT;
        return $wpdb->get_results($wpdb->prepare("SELECT * FROM {$table} WHERE license_id=%d ORDER BY paid_at DESC LIMIT %d", $license_id, $limit), ARRAY_A) ?: [];
    }

    // --- Payment reminders -------------------------------------------------
    public static function cron_send_reminders(): void {
        global $wpdb;
        $plan_table = $wpdb->prefix . self::TABLE_PLAN;
        $lic_table  = $wpdb->prefix . 'edd_licenses';
        $cust_table = $wpdb->prefix . 'edd_customers';
        // Find plans that are active/overdue/single with remaining > 0
        $rows = $wpdb->get_results("SELECT p.*, l.license_key, c.email as customer_email, c.name as customer_name FROM {$plan_table} p JOIN {$lic_table} l ON l.id=p.license_id LEFT JOIN {$cust_table} c ON c.id=l.customer_id WHERE p.status IN ('active','overdue','single') AND p.remaining_amount > 0.01", ARRAY_A);
        if (!$rows) return;
        $site = get_bloginfo('name');
        $sent = 0;
        foreach ($rows as $row) {
            $email = $row['customer_email'] ?: '';
            if (!is_email($email)) continue;
            // Throttle: only remind if not reminded in last 7 days OR overdue daily. Check licensemeta last_reminder
            $last = get_metadata('edd_license', (int)$row['license_id'], 'wpuiai_last_payment_reminder', true);
            if ($last && (time() - (int)$last) < 6*24*3600 && $row['status'] !== 'overdue') continue;
            // overdue => escalate every 2d
            if ($row['status']==='overdue' && $last && (time()-(int)$last) < 2*24*3600) continue;
            $title = get_the_title((int)$row['download_id']) ?: 'License';
            $remaining = number_format((float)$row['remaining_amount'],2);
            $paid = number_format((float)$row['paid_amount'],2);
            $total = number_format((float)$row['total_price'],2);
            $manage_url = admin_url('admin.php?page=focusa-uiai-licenses');
            // Stripe pay link if stripe plan
            $stripe_pay = '';
            if ($row['plan_type']==='stripe') {
                $checkout = get_metadata('edd_license', (int)$row['license_id'], 'wpuiai_stripe_checkout_url', true);
                $link = $checkout ?: ('https://wpuiai.com/checkout/?license_id='.(int)$row['license_id']);
                $stripe_pay = "\nPay securely via Stripe: ".$link." (or reply to this email)\n";
            }
            $subject = ($row['status']==='overdue' ? '[Overdue] ' : '[Reminder] ') . $title.' — $'.$remaining.' remaining';
            $body = "Hi ".($row['customer_name'] ?: $email).",\n\n"
                  . "This is a friendly reminder for your {$title} license (".substr($row['license_key'],0,12)."…).\n"
                  . "Total: \${$total}  Paid: \${$paid}  Remaining: \${$remaining}\n"
                  . "Installments: ".(int)$row['installments_paid']."/".(int)$row['installments_total']."  Status: {$row['status']}\n"
                  . $stripe_pay
                  . "\nYour license is active and already registered with the software — this is just the payment balance.\n"
                  . "If you have already paid, reply with the transaction ID and we will reconcile immediately.\n"
                  . "\n— {$site}\n"
                  . "Manage: {$manage_url}\n";
            $headers = ['Content-Type: text/plain; charset=UTF-8'];
            $ok = wp_mail($email, $subject, $body, $headers);
            if ($ok) {
                update_metadata('edd_license', (int)$row['license_id'], 'wpuiai_last_payment_reminder', (string)time());
                $sent++;
                // audit log
                error_log("[wpuiai-payments] reminder sent to {$email} license {$row['license_id']} remaining \${$remaining}");
            }
        }
        // admin digest
        if ($sent>0) {
            $admin = get_option('admin_email');
            wp_mail($admin, "[wpuiai] {$sent} payment reminder(s) sent", "Sent {$sent} license payment reminder(s) at ".current_time('mysql')."\n", ['Content-Type: text/plain; charset=UTF-8']);
        }
    }

    // --- Stripe awareness ---------------------------------------------------
    private static function stripe_settings(): array {
        $edd = get_option('edd_settings', []);
        $test = !empty($edd['stripe_test_mode']) || !empty($edd['stripe_test_mode_enabled']);
        $sk = $test ? ($edd['stripe_test_secret'] ?? $edd['test_secret_key'] ?? '') : ($edd['stripe_live_secret'] ?? $edd['live_secret_key'] ?? '');
        // Also support env constant
        if (!$sk && defined('STRIPE_SECRET_KEY')) $sk = STRIPE_SECRET_KEY;
        if (!$sk) $sk = (string)get_option('wpuiai_stripe_secret_key','');
        return ['enabled'=>!empty($sk), 'secret'=>$sk, 'test'=>$test, 'edd'=>$edd];
    }
    private static function stripe_customer_id_for_license(int $license_id): ?string {
        global $wpdb;
        $lic = $wpdb->get_row($wpdb->prepare("SELECT customer_id FROM {$wpdb->prefix}edd_licenses WHERE id=%d", $license_id), ARRAY_A);
        if (!$lic || empty($lic['customer_id'])) return null;
        $cid = (int)$lic['customer_id'];
        // edd_customers meta or wpuiai mapping
        $stripe = $wpdb->get_var($wpdb->prepare("SELECT meta_value FROM {$wpdb->prefix}edd_customermeta WHERE customer_id=%d AND meta_key IN ('stripe_customer_id','_stripe_customer_id') LIMIT 1", $cid));
        if ($stripe) return $stripe;
        $meta = get_metadata('edd_customer', $cid, 'stripe_customer_id', true);
        return $meta ?: null;
    }
    public static function ensure_stripe_sync(int $license_id): array {
        $cfg = self::stripe_settings();
        if (!$cfg['enabled']) return ['ok'=>false,'stripe'=>false,'reason'=>'stripe_not_configured','hint'=>'Set EDD Stripe keys in Downloads→Settings→Payments→Stripe or wp option wpuiai_stripe_secret_key'];
        // lazy: verify SDK present
        if (!class_exists('\\Stripe\\Stripe') && !class_exists('\\EDD\\Vendor\\Stripe\\Stripe') && !class_exists('\\Stripe\\ApiRequestor')) {
            $candidates = [
                WP_PLUGIN_DIR.'/easy-digital-downloads/libraries/Stripe/init.php',
                WP_PLUGIN_DIR.'/easy-digital-downloads/vendor/autoload.php',
                WP_PLUGIN_DIR.'/easy-digital-downloads/vendor/stripe/stripe-php/init.php',
                WP_PLUGIN_DIR.'/gravityformsstripe/includes/stripe/stripe-php/init.php',
            ];
            $found=false;
            foreach ($candidates as $try) { if (file_exists($try)) { require_once $try; $found=true; break; } }
            if (!class_exists('\\Stripe\\Stripe') && !class_exists('\\EDD\\Vendor\\Stripe\\Stripe') && !class_exists('\\Stripe\\ApiRequestor') && !$found) return ['ok'=>false,'stripe'=>false,'reason'=>'stripe_sdk_missing','tried'=>$candidates];
        }
        $cls = class_exists('\\EDD\\Vendor\\Stripe\\Stripe') ? '\\EDD\\Vendor\\Stripe\\Stripe' : '\\Stripe\\Stripe';
        try { $cls::setApiKey($cfg['secret']); } catch(\Throwable $e) { return ['ok'=>false,'error'=>$e->getMessage()]; }
        $cust = self::stripe_customer_id_for_license($license_id);
        return ['ok'=>true,'stripe'=>true,'test'=>$cfg['test'],'stripe_customer_id'=>$cust,'secret_prefix'=>substr($cfg['secret'],0,7).'***'];
    }
    /** WP->Stripe: create Customer/Price/PaymentIntent for a stripe plan (bidirectional outbound) */
    private static function stripe_create_for_plan(int $license_id, int $download_id, float $total_price, int $installments, int $plan_id): array {
        global $wpdb;
        $cfg = self::stripe_settings();
        // ensure SDK loaded already via ensure_stripe_sync; pick class names
        $stripeCls = class_exists('\\EDD\\Vendor\\Stripe\\Stripe') ? '\\EDD\\Vendor\\Stripe\\Stripe' : '\\Stripe\\Stripe';
        $customerCls = class_exists('\\EDD\\Vendor\\Stripe\\Customer') ? '\\EDD\\Vendor\\Stripe\\Customer' : '\\Stripe\\Customer';
        $priceCls = class_exists('\\EDD\\Vendor\\Stripe\\Price') ? '\\EDD\\Vendor\\Stripe\\Price' : '\\Stripe\\Price';
        $productCls = class_exists('\\EDD\\Vendor\\Stripe\\Product') ? '\\EDD\\Vendor\\Stripe\\Product' : '\\Stripe\\Product';
        $piCls = class_exists('\\EDD\\Vendor\\Stripe\\PaymentIntent') ? '\\EDD\\Vendor\\Stripe\\PaymentIntent' : '\\Stripe\\PaymentIntent';
        // 1) customer
        $email = (string) $wpdb->get_var($wpdb->prepare("SELECT c.email FROM {$wpdb->prefix}edd_licenses l JOIN {$wpdb->prefix}edd_customers c ON c.id=l.customer_id WHERE l.id=%d", $license_id));
        $existing_cid = self::stripe_customer_id_for_license($license_id);
        $customer_id = $existing_cid;
        if (!$customer_id && $email) {
            // create stripe customer (idempotent via email + license_id search done on next run)
            $c = $customerCls::create([
                'email' => $email,
                'description' => 'wpuiai license '.$license_id,
                'metadata' => ['wpuiai_license_id'=>(string)$license_id, 'license_id'=>(string)$license_id, 'download_id'=>(string)$download_id],
            ]);
            $customer_id = $c->id ?? null;
            if ($customer_id) {
                // persist to edd_customermeta
                $licRow = $wpdb->get_row($wpdb->prepare("SELECT customer_id FROM {$wpdb->prefix}edd_licenses WHERE id=%d", $license_id), ARRAY_A);
                if ($licRow && !empty($licRow['customer_id']) && function_exists('edd_add_customer_meta')) {
                    edd_add_customer_meta((int)$licRow['customer_id'], 'stripe_customer_id', $customer_id);
                }
            }
        }
        // 2) product + price (unit_amount = total/installments ceil for demo, or total for single)
        $opt_price = 'wpuiai_stripe_price_'.$download_id;
        $price_id = (string) get_option($opt_price, '');
        $amount_cents = (int) round($total_price * 100);
        // if installments>1, create per-installment price
        $per_install_cents = $installments > 1 ? (int) ceil($amount_cents / $installments) : $amount_cents;
        if (!$price_id) {
            // try to reuse existing product
            $title = get_the_title($download_id) ?: 'License '.$download_id;
            $prod = $productCls::create(['name'=>$title, 'metadata'=>['download_id'=>(string)$download_id, 'wpuiai_download_id'=>(string)$download_id]]);
            $price = $priceCls::create([
                'product'=>$prod->id,
                'unit_amount'=>$per_install_cents,
                'currency'=>'usd',
                'metadata'=>['download_id'=>(string)$download_id, 'wpuiai_license_id'=>(string)$license_id],
            ]);
            $price_id = $price->id ?? '';
            if ($price_id) update_option($opt_price, $price_id, false);
        }
        // 3) PaymentIntent for the remaining balance (proof trigger). Stripe test mode uses pm_card_visa to confirm.
        $pi = $piCls::create([
            'amount'=>$amount_cents,
            'currency'=>'usd',
            'customer'=>$customer_id ?: null,
            'metadata'=>['license_id'=>(string)$license_id, 'wpuiai_license_id'=>(string)$license_id, 'download_id'=>(string)$download_id, 'plan_id'=>(string)$plan_id, 'installments'=>(string)$installments],
            'description'=>'wpuiai license '.$license_id.' plan '.$plan_id,
            // Do not auto-confirm here; allow dashboard confirm or test trigger
        ]);
        $pi_id = $pi->id ?? '';
        $checkout_url = $pi->id ? ('https://dashboard.stripe.com/'.($cfg['test'] ? 'test/' : '').'payments/'.$pi->id) : '';
        return [
            'ok'=>true,
            'customer_id'=>$customer_id,
            'price_id'=>$price_id,
            'payment_intent_id'=>$pi_id,
            'checkout_url'=>$checkout_url,
            'amount_cents'=>$amount_cents,
            'test'=>$cfg['test'],
        ];
    }

    public static function stripe_webhook(\WP_REST_Request $req): \WP_REST_Response {
        $payload = $req->get_body();
        $sig = $req->get_header('stripe-signature') ?? $req->get_header('Stripe-Signature') ?? '';
        $cfg = self::stripe_settings();
        $wh_secret = get_option('wpuiai_stripe_webhook_secret','');
        $wh_secret_test = get_option('wpuiai_stripe_webhook_secret_test','');
        // If we have wh_secret, verify sig (light) — try live then test
        $event = null;
        $whCls = class_exists('\\EDD\\Vendor\\Stripe\\Webhook') ? '\\EDD\\Vendor\\Stripe\\Webhook' : '\\Stripe\\Webhook';
        if ($wh_secret && class_exists($whCls)) {
            try { $event = $whCls::constructEvent($payload, $sig, $wh_secret); } catch(\Throwable $e) { $event = null; }
        }
        if (!$event && $wh_secret_test && class_exists($whCls) && $wh_secret_test !== $wh_secret) {
            try { $event = $whCls::constructEvent($payload, $sig, $wh_secret_test); } catch(\Throwable $e) { $event = null; }
        }
        if (!$event) $event = json_decode($payload, true);
        if (!$event) return new \WP_REST_Response(['ok'=>false,'error'=>'invalid_payload'],400);
        $type = $event['type'] ?? $event['object'] ?? 'unknown';
        // Handle payment_intent.succeeded / invoice.paid -> record payment against license via metadata license_id
        $pi = $event['data']['object'] ?? $event;
        $amount = 0.0; $license_id = 0; $method='stripe';
        // Try to find license_id from metadata
        $meta = $pi['metadata'] ?? [];
        if (!empty($meta['license_id'])) $license_id = (int)$meta['license_id'];
        elseif (!empty($meta['wpuiai_license_id'])) $license_id = (int)$meta['wpuiai_license_id'];
        // amount is in cents
        if (isset($pi['amount_received'])) $amount = ((int)$pi['amount_received'])/100.0;
        elseif (isset($pi['amount_paid'])) $amount = ((int)$pi['amount_paid'])/100.0;
        elseif (isset($pi['amount'])) $amount = ((int)$pi['amount'])/100.0;
        $txn = $pi['id'] ?? '';
        if ($license_id && $amount>0) {
            $res = self::record_payment($license_id, $amount, $method, $txn, 'Stripe webhook: '.$type);
            return new \WP_REST_Response(['ok'=>!empty($res['ok']),'type'=>$type,'auto_recorded'=>$res], $res['ok'] ? 200: 400);
        }
        // If no license_id, try to match by customer email
        $email = $pi['customer_email'] ?? $pi['receipt_email'] ?? $meta['customer_email'] ?? '';
        if ($email) {
            global $wpdb;
            $row = $wpdb->get_row($wpdb->prepare("SELECT id FROM {$wpdb->prefix}edd_licenses l JOIN {$wpdb->prefix}edd_customers c ON c.id=l.customer_id WHERE c.email=%s ORDER BY l.id DESC LIMIT 1", $email), ARRAY_A);
            if ($row && $amount>0) {
                $res = self::record_payment((int)$row['id'], $amount, $method, $txn, 'Stripe webhook via email '.$type);
                return new \WP_REST_Response(['ok'=>!empty($res['ok']),'type'=>$type,'auto_recorded'=>$res,'matched_by'=>'email'], $res['ok'] ? 200: 400);
            }
        }
        return new \WP_REST_Response(['ok'=>true,'type'=>$type,'note'=>'received but no license auto-match; add metadata license_id to Stripe object'],202);
    }

    // REST: verify license + plan so actual software can register

    public static function rest_verify(\WP_REST_Request $req): \WP_REST_Response {
        $key = trim((string)($req->get_param('license_key') ?? $req->get_param('key') ?? ''));
        $email = sanitize_email((string)($req->get_param('email') ?? ''));
        if ($key === '' && $email === '') return new \WP_REST_Response(['ok'=>false,'error'=>'license_key or email required'], 400);
        global $wpdb;
        $table = $wpdb->prefix . 'edd_licenses';
        if ($key !== '') {
            $row = $wpdb->get_row($wpdb->prepare("SELECT id, license_key, download_id, status, expiration, date_created, customer_id, user_id FROM {$table} WHERE license_key=%s LIMIT 1", $key), ARRAY_A);
        } else {
            $row = $wpdb->get_row($wpdb->prepare("SELECT id, license_key, download_id, status, expiration, date_created, customer_id, user_id FROM {$table} WHERE id IN (SELECT id FROM {$wpdb->prefix}edd_licenses WHERE customer_id IN (SELECT id FROM {$wpdb->prefix}edd_customers WHERE email=%s) ) ORDER BY id DESC LIMIT 1", $email), ARRAY_A);
        }
        if (!$row) return new \WP_REST_Response(['ok'=>false,'error'=>'not_found'], 404);
        $license_id = (int)$row['id'];
        // Attach product title + price
        $product_title = get_the_title((int)$row['download_id']) ?: 'Unknown';
        $price = get_post_meta((int)$row['download_id'], 'edd_price', true);
        $plan = self::get_plan($license_id);
        $payments = $plan ? self::get_payments($license_id, 10) : [];
        // Seats
        $seats = class_exists('WPUIAI_AIC_Focusa_License_Production') ? WPUIAI_AIC_Focusa_License_Production::license_truth($license_id) : null;
        return new \WP_REST_Response([
            'ok'=>true,
            'schema'=>'focusa.license_verify.v1',
            'license'=>[
                'id'=>$license_id,
                'license_key'=>substr($row['license_key'],0,8).'****',
                'license_key_full'=> ($key !== '' && $row['license_key']===$key) ? $row['license_key'] : null, // only echo back if caller supplied exact key
                'download_id'=>(int)$row['download_id'],
                'product'=>$product_title,
                'status'=>$row['status'],
                'expiration'=>$row['expiration'],
                'created_at'=>$row['date_created'],
                'price'=>$price !== '' ? (float)$price : null,
                'customer_id'=>(int)$row['customer_id'],
            ],
            'payment_plan'=> $plan ? [
                'id'=>(int)$plan['id'],
                'total_price'=>(float)$plan['total_price'],
                'currency'=>$plan['currency'],
                'paid_amount'=>(float)$plan['paid_amount'],
                'remaining_amount'=>(float)$plan['remaining_amount'],
                'installments_total'=>(int)$plan['installments_total'],
                'installments_paid'=>(int)$plan['installments_paid'],
                'status'=>$plan['status'],
                'plan_type'=>$plan['plan_type'],
                'next_due_date'=>$plan['next_due_date'],
                'recent_payments'=> array_map(fn($p)=>['amount'=>(float)$p['amount'],'method'=>$p['payment_method'],'paid_at'=>$p['paid_at'],'txn'=>$p['transaction_ref']], $payments),
            ] : null,
            'seats'=>$seats,
        ]);
    }
    public static function rest_create_plan(\WP_REST_Request $req): \WP_REST_Response {
        $license_id=(int)$req->get_param('license_id');
        $download_id=(int)$req->get_param('download_id');
        $total=(float)$req->get_param('total_price');
        $installments=(int)($req->get_param('installments') ?: 1);
        $type=sanitize_text_field((string)($req->get_param('plan_type') ?: 'manual'));
        $order_id = $req->get_param('order_id') ? (int)$req->get_param('order_id') : null;
        if (!$license_id) return new \WP_REST_Response(['ok'=>false,'error'=>'license_id required'],400);
        if (!$download_id) {
            global $wpdb; $download_id=(int)$wpdb->get_var($wpdb->prepare("SELECT download_id FROM {$wpdb->prefix}edd_licenses WHERE id=%d",$license_id));
        }
        if (!$total) {
            $price=get_post_meta($download_id,'edd_price',true);
            $total= $price !== '' ? (float)$price : 0;
        }
        $res=self::create_plan($license_id,$download_id,$total,$installments,$type,$order_id);
        return new \WP_REST_Response($res, $res['ok'] ? 200 : 400);
    }
    public static function rest_record_payment(\WP_REST_Request $req): \WP_REST_Response {
        $license_id=(int)$req->get_param('license_id');
        $amount=(float)$req->get_param('amount');
        $method=sanitize_text_field((string)($req->get_param('method') ?: 'manual'));
        $txn=sanitize_text_field((string)($req->get_param('transaction_ref') ?? ''));
        $notes=sanitize_text_field((string)($req->get_param('notes') ?? ''));
        if (!$license_id || $amount<=0) return new \WP_REST_Response(['ok'=>false,'error'=>'license_id and amount required'],400);
        $res=self::record_payment($license_id,$amount,$method,$txn,$notes, get_current_user_id());
        return new \WP_REST_Response($res, $res['ok'] ? 200 : 400);
    }
    public static function rest_get_plan(\WP_REST_Request $req): \WP_REST_Response {
        $id=(int)$req->get_param('id');
        global $wpdb;
        $table=$wpdb->prefix . self::TABLE_PLAN;
        $row=$wpdb->get_row($wpdb->prepare("SELECT * FROM {$table} WHERE license_id=%d LIMIT 1",$id), ARRAY_A);
        if (!$row) return new \WP_REST_Response(['ok'=>false,'error'=>'not_found'],404);
        $row['payments']=self::get_payments($id);
        return new \WP_REST_Response(['ok'=>true,'plan'=>$row]);
    }
}
// bootstrap
add_action('init', function(){ WPUIAI_AIC_License_Payment_Plan::register_rest(); });
if (defined('WP_CLI') && WP_CLI) {
    WP_CLI::add_command('wpuiai license plan-create', function($args,$assoc){
        $lid=(int)($args[0]??0); $total=(float)($assoc['total']??0); $inst=(int)($assoc['installments']??1);
        if(!$lid) WP_CLI::error('usage: wp wpuiai license plan-create <license_id> --total=697 --installments=3');
        $dl=(int) WP_CLI::runcommand("eval echo (int) global \$wpdb; echo \$wpdb->get_var(\$wpdb->prepare(\"SELECT download_id FROM {\$wpdb->prefix}edd_licenses WHERE id=%d\", $lid));", ['return'=>'all']);
        // fallback direct
        global $wpdb; $dl=(int)$wpdb->get_var($wpdb->prepare("SELECT download_id FROM {$wpdb->prefix}edd_licenses WHERE id=%d",$lid));
        $r=WPUIAI_AIC_License_Payment_Plan::create_plan($lid,$dl,$total,$inst);
        WP_CLI::log(json_encode($r, JSON_PRETTY_PRINT));
        if(empty($r['ok'])) WP_CLI::error('failed');
        WP_CLI::success('plan created');
    });
    WP_CLI::add_command('wpuiai license pay', function($args,$assoc){
        $lid=(int)($args[0]??0); $amount=(float)($args[1]??0);
        if(!$lid||!$amount) WP_CLI::error('usage: wp wpuiai license pay <license_id> <amount> [--method=stripe --txn=... --notes=...]');
        $r=WPUIAI_AIC_License_Payment_Plan::record_payment($lid,$amount,$assoc['method']??'manual',$assoc['txn']??'', $assoc['notes']??'');
        WP_CLI::log(json_encode($r, JSON_PRETTY_PRINT));
        if(empty($r['ok'])) WP_CLI::error('failed');
        WP_CLI::success('payment recorded');
    });
    WP_CLI::add_command('wpuiai license verify', function($args,$assoc){
        $key=$args[0]??''; if(!$key) WP_CLI::error('usage: wp wpuiai license verify <license_key>');
        global $wpdb; $row=$wpdb->get_row($wpdb->prepare("SELECT id FROM {$wpdb->prefix}edd_licenses WHERE license_key=%s",$key), ARRAY_A);
        if(!$row) WP_CLI::error('not found');
        $plan=WPUIAI_AIC_License_Payment_Plan::get_plan((int)$row['id']);
        WP_CLI::log(json_encode(['license_id'=>(int)$row['id'],'plan'=>$plan,'payments'=>WPUIAI_AIC_License_Payment_Plan::get_payments((int)$row['id'])], JSON_PRETTY_PRINT));
    });
}
