<?php
// Transactional mail adapter: sends branded verification challenges using the facade
// registry sender identity and templates. Does not store or log plaintext verifiers or
// unmasked email addresses.
declare(strict_types=1);

final class FocusaSpec152eTransactionalMailAdapter
{
    public const SCHEMA = 'focusa.spec152e.transactional_mail_adapter.v1';
    public const DELIVERY_TTL_SECONDS = 300;

    /** @var Closure(string, string, string, string, string): bool */
    private Closure $send;

    /**
     * @param callable $send Signature: (string $to, string $subject, string $htmlBody, string $textBody, string $senderIdentity) -> bool
     */
    public function __construct(callable $send)
    {
        $this->send = Closure::fromCallable($send);
    }

    /**
     * Send a branded verification challenge email.
     *
     * Required fields:
     *   - facade:         full facade registry entry
     *   - to:             recipient email address (never logged)
     *   - challenge_kind: 'magic_link' or 'otp'
     *   - magic_link:     full branded magic link URL (only when kind is magic_link)
     *   - otp_code:       human-readable OTP code (only when kind is otp)
     *   - expires_at:     canonical UTC expiry timestamp
     *   - registration_id: opaque registration UUID
     *   - product_code:   public product code
     *
     * Returns:
     *   - sent:           true when delivery was attempted
     *   - delivery_status: 'attempted' | 'suppressed' | 'bounced'
     *   - attempted_at:   canonical UTC timestamp
     */
    public function sendVerificationChallenge(array $input): array
    {
        $facade = $input['facade'] ?? [];
        if (!is_array($facade) || !isset($facade['facade_id'], $facade['sender'], $facade['brand'], $facade['exact_origins'])) {
            throw new InvalidArgumentException('registered facade entry required');
        }
        $to = (string) ($input['to'] ?? '');
        if ($to === '' || filter_var($to, FILTER_VALIDATE_EMAIL) === false) {
            throw new InvalidArgumentException('valid recipient email required');
        }
        $kind = (string) ($input['challenge_kind'] ?? '');
        if (!in_array($kind, ['magic_link', 'otp'], true)) {
            throw new InvalidArgumentException('bounded challenge kind required');
        }
        $senderIdentity = (string) ($facade['sender']['identity'] ?? '');
        $senderName = (string) ($facade['sender']['display_name'] ?? '');
        $brandName = (string) ($facade['brand']['name'] ?? '');
        $brandLogoPath = (string) ($facade['brand']['logo_path'] ?? '');

        if ($senderIdentity === '' || $senderName === '' || $brandName === '') {
            throw new InvalidArgumentException('registered facade sender identity required');
        }

        $now = (new DateTimeImmutable('now', new DateTimeZone('UTC')))->format('Y-m-d\TH:i:s\Z');
        FocusaSpec152eActivationRegistrationMigration::assertTimestamp($input['expires_at'] ?? '');

        $subject = $this->buildSubject($kind, $brandName);
        $htmlBody = $this->buildHtmlBody($kind, $brandName, $brandLogoPath, $input);
        $textBody = $this->buildTextBody($kind, $brandName, $input);

        $sent = ($this->send)($to, $subject, $htmlBody, $textBody, $senderIdentity);

        return [
            'sent' => $sent,
            'delivery_status' => $sent ? 'attempted' : 'suppressed',
            'attempted_at' => $now,
            'sender_identity' => $senderIdentity,
        ];
    }

    /**
     * Send a branded delivery confirmation or license delivery email.
     */
    public function sendDeliveryNotification(array $input): array
    {
        $facade = $input['facade'] ?? [];
        if (!is_array($facade) || !isset($facade['sender'])) {
            throw new InvalidArgumentException('registered facade entry required');
        }
        $to = (string) ($input['to'] ?? '');
        if ($to === '' || filter_var($to, FILTER_VALIDATE_EMAIL) === false) {
            throw new InvalidArgumentException('valid recipient email required');
        }
        $notificationKind = (string) ($input['notification_kind'] ?? '');
        if (!in_array($notificationKind, ['entitlement_issued', 'delivery_ready', 'refunded', 'revoked'], true)) {
            throw new InvalidArgumentException('bounded notification kind required');
        }
        $senderIdentity = (string) ($facade['sender']['identity'] ?? '');
        $senderName = (string) ($facade['sender']['display_name'] ?? '');
        $brandName = (string) ($facade['brand']['name'] ?? '');

        $now = (new DateTimeImmutable('now', new DateTimeZone('UTC')))->format('Y-m-d\TH:i:s\Z');
        $subject = $this->buildNotificationSubject($notificationKind, $brandName);
        $htmlBody = $this->buildNotificationHtml($notificationKind, $brandName, $input);
        $textBody = $this->buildNotificationText($notificationKind, $brandName, $input);

        $sent = ($this->send)($to, $subject, $htmlBody, $textBody, $senderIdentity);
        return [
            'sent' => $sent,
            'delivery_status' => $sent ? 'attempted' : 'suppressed',
            'attempted_at' => $now,
        ];
    }

    // ── template builders ──────────────────────────────────────────────

    private function buildSubject(string $kind, string $brandName): string
    {
        if ($kind === 'magic_link') {
            return "{$brandName} — Verify your email to continue activation";
        }
        return "{$brandName} — Your verification code: [code]";
    }

    private function buildHtmlBody(string $kind, string $brandName, string $brandLogoPath, array $input): string
    {
        $logo = $brandLogoPath !== '' ? '<img src="' . self::escape($brandLogoPath) . '" alt="' . self::escape($brandName) . '" style="max-width:180px;height:auto;">' : '';
        $product = self::escape((string) ($input['product_code'] ?? ''));
        $expires = self::escape((string) ($input['expires_at'] ?? ''));

        if ($kind === 'magic_link') {
            $link = self::escape((string) ($input['magic_link'] ?? ''));
            return <<<HTML
<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>Verify your email</title></head>
<body style="font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;max-width:560px;margin:0 auto;padding:24px;">
{$logo}
<h1 style="font-size:20px;color:#1a1a2e;">Verify your email for {$brandName}</h1>
<p>You requested activation for <strong>{$product}</strong>. Click the button below to verify your email address.</p>
<p style="margin:24px 0;">
  <a href="{$link}" style="display:inline-block;padding:12px 24px;background:#1a1a2e;color:#ffffff;text-decoration:none;border-radius:6px;font-weight:600;">Verify Email</a>
</p>
<p style="font-size:12px;color:#6b7280;">This link expires at {$expires}. If you did not request this activation, you can safely ignore this message.</p>
</body>
</html>
HTML;
        }

        // OTP code
        $code = self::escape((string) ($input['otp_code'] ?? ''));
        return <<<HTML
<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>Your verification code</title></head>
<body style="font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;max-width:560px;margin:0 auto;padding:24px;">
{$logo}
<h1 style="font-size:20px;color:#1a1a2e;">Your verification code for {$brandName}</h1>
<p>You requested activation for <strong>{$product}</strong>. Enter this code in your terminal to continue:</p>
<div style="margin:24px 0;padding:16px 24px;background:#f3f4f6;border-radius:6px;text-align:center;">
  <span style="font-size:28px;font-family:monospace;letter-spacing:4px;font-weight:700;">{$code}</span>
</div>
<p style="font-size:12px;color:#6b7280;">This code expires at {$expires}. If you did not request this activation, you can safely ignore this message.</p>
</body>
</html>
HTML;
    }

    private function buildTextBody(string $kind, string $brandName, array $input): string
    {
        $product = (string) ($input['product_code'] ?? '');
        $expires = (string) ($input['expires_at'] ?? '');

        if ($kind === 'magic_link') {
            $link = (string) ($input['magic_link'] ?? '');
            return "{$brandName} — Verify your email\n\n"
                . "You requested activation for {$product}.\n\n"
                . "Verify your email: {$link}\n\n"
                . "This link expires at {$expires}. If you did not request this activation, ignore this message.\n";
        }

        $code = (string) ($input['otp_code'] ?? '');
        return "{$brandName} — Your verification code: {$code}\n\n"
            . "You requested activation for {$product}.\n\n"
            . "Enter this code in your terminal: {$code}\n\n"
            . "This code expires at {$expires}. If you did not request this activation, ignore this message.\n";
    }

    private function buildNotificationSubject(string $kind, string $brandName): string
    {
        return match ($kind) {
            'entitlement_issued' => "{$brandName} — Your license is ready",
            'delivery_ready' => "{$brandName} — Your license is available for terminal delivery",
            'refunded' => "{$brandName} — Your order has been refunded",
            'revoked' => "{$brandName} — License status update",
            default => "{$brandName} — Account notification",
        };
    }

    private function buildNotificationHtml(string $kind, string $brandName, array $input): string
    {
        $message = match ($kind) {
            'entitlement_issued' => "Your license for {$brandName} has been issued. You can find your license key in your account dashboard.",
            'delivery_ready' => "Your license is ready for terminal delivery. Return to your terminal session to continue activation.",
            'refunded' => "Your order for {$brandName} has been refunded. Access will be revoked within the active lease window.",
            'revoked' => "Your license for {$brandName} has been updated. Please check your account for details.",
            default => "Your {$brandName} account has been updated.",
        };
        return "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"></head>"
            . "<body style=\"font-family:sans-serif;max-width:560px;margin:0 auto;padding:24px;\">"
            . "<p>{$message}</p></body></html>";
    }

    private function buildNotificationText(string $kind, string $brandName, array $input): string
    {
        return match ($kind) {
            'entitlement_issued' => "{$brandName} — Your license has been issued.",
            'delivery_ready' => "{$brandName} — Your license is ready for terminal delivery.",
            'refunded' => "{$brandName} — Your order has been refunded.",
            'revoked' => "{$brandName} — License status update.",
            default => "{$brandName} — Account notification.",
        };
    }

    private static function escape(string $value): string
    {
        return htmlspecialchars($value, ENT_QUOTES | ENT_HTML5, 'UTF-8');
    }
}