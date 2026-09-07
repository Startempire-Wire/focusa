<?php
declare(strict_types=1);
require __DIR__ . '/../docs/contracts/spec152e-edd-bound-lease-issuer.v1.php';
function check_signer(bool $ok, string $message): void { if (!$ok) throw new RuntimeException($message); }
$clock = static fn() => '2026-09-07T00:00:00Z';
$config = [
    'schema'=>'focusa.authority_signing_config.v1',
    'root_seed_b64'=>base64_encode(random_bytes(32)), 'lease_seed_b64'=>base64_encode(random_bytes(32)),
    'root_key_id'=>'isolated-test-root-8', 'lease_key_id'=>'isolated-test-lease-8', 'key_set_sequence'=>8,
    'issued_at'=>'2026-09-01T00:00:00Z', 'not_before'=>'2026-09-01T00:00:00Z',
    'expires_at'=>'2027-09-01T00:00:00Z', 'not_after'=>'2027-09-01T00:00:00Z',
];
$signer = FocusaSpec152eAuthorityKeySetSeam::fromProtectedConfiguration($config, $clock);
$envelope = $signer->configuredKeySetEnvelope();
$payload = FocusaSpec152eAuthorityKeySetSeam::decodeJson($envelope['payload_b64']);
check_signer($envelope['signer_key_id'] === $config['root_key_id'], 'root identity must come from provider configuration');
check_signer($payload['keys'][0]['key_id'] === $config['lease_key_id'], 'lease identity must come from provider configuration');
check_signer($payload['sequence'] === 8 && $payload['keys'][0]['not_after'] === $config['not_after'], 'configured sequence and lifetime preserved');
$bad = [
    [[], 'AUTHORITY_SIGNING_CONFIG_REQUIRED'],
    [array_replace($config,['root_seed_b64'=>base64_encode(implode('',array_map('chr',range(0,31))))]),'AUTHORITY_FIXTURE_KEY_FORBIDDEN'],
    [array_replace($config,['lease_seed_b64'=>$config['root_seed_b64']]),'AUTHORITY_SIGNING_KEYS_NOT_SEPARATED'],
    [array_replace($config,['key_set_sequence'=>7]),'AUTHORITY_SIGNING_SEQUENCE_INVALID'],
    [array_replace($config,['root_key_id'=>FocusaSpec152eAuthorityKeySetSeam::ROOT_KEY_ID]),'AUTHORITY_RETIRED_KEY_ID'],
    [array_replace($config,['not_after'=>'2026-09-06T00:00:00Z']),'AUTHORITY_SIGNING_WINDOW_INVALID'],
];
foreach ($bad as [$input,$reason]) {
    try { FocusaSpec152eAuthorityKeySetSeam::fromProtectedConfiguration($input,$clock); throw new RuntimeException('invalid configuration accepted'); }
    catch (DomainException $e) { check_signer($e->getMessage()===$reason,'wrong failure reason'); }
}
echo json_encode(['result'=>'passed','negative_cases'=>count($bad),'fixture_only'=>true,'root_public_key_b64'=>$signer->rootPublicKeyB64(),'key_set_envelope'=>$envelope],JSON_THROW_ON_ERROR),"\n";
