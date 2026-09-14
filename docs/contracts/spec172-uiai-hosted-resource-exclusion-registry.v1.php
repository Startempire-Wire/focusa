<?php
// Spec 172 UIAI hosted-resource exclusion registry (atom focusa-vbcqu.20.15.14; addendum
// sections 6.3, 7.2, 8.3, 15, and 21). Server-owned and frozen at first sale: the UIAI
// Operator Lifetime v1 License Type explicitly excludes unlimited hosted compute, paid
// proxies, third-party API consumption, paid model usage, managed hosting, resale,
// redistribution, and product embedding unless those rights are separately granted.
// The UIAI local/product capability families (browser observation, browser action, local
// persistence, diagnostics, proof packets, batch/responsive workflows, supported
// integrations) are the granted local surface; this registry never includes a hosted or
// metered resource in v1.
//
//   - `EXCLUSIONS` is the canonical frozen deny list with stable reason codes. Each
//     excluded resource is carried verbatim into the UIAI projection and the
//     grant/child-token fixture so no runtime can re-derive hosted rights from local or
//     caller tables.
//   - `isIncluded()` fails closed: every registered hosted/metered resource is excluded
//     and any unknown resource is denied (Spec 172 sections 7.2 and 8.3 default
//     exclusion). Only an explicit operator-approved future registry version may add a
//     granted hosted resource.
//   - `assertIncluded()` raises the public-safe stable code `HOSTED_RESOURCE_NOT_INCLUDED`
//     (Spec 172 section 21) with the documented recovery action; it never exposes raw
//     email, keys, tokens, customer identifiers, or card data.
//   - `digest()` is a deterministic SHA-256 over the canonical frozen record, identical
//     for every v1 license; projections and fixtures carry it as the explicit
//     local/hosted boundary digest.
declare(strict_types=1);

final class UiaiSpec172HostedResourceExclusionRegistry
{
    public const SCHEMA = 'focusa.spec172.uiai_hosted_resource_exclusion_registry.v1';
    public const VERSION = 1;
    public const PRODUCT = 'uiai_engine';
    public const LICENSE_TYPE = 'uiai_operator_lifetime_v1';
    public const AUTHORITY = 'docs/172-focusa-spec152-license-type-and-surface-entitlement-governance-addendum.md';
    public const SECTION = '7.2';
    public const ERROR_CODE = 'HOSTED_RESOURCE_NOT_INCLUDED';

    /** Frozen first-sale hosted/metered deny list (Spec 172 section 7.2). */
    public const EXCLUSIONS = [
        'unlimited_hosted_compute' => 'not_included_unless_explicitly_listed',
        'paid_proxies' => 'not_included_unless_explicitly_listed',
        'third_party_api_consumption' => 'not_included_unless_explicitly_listed',
        'paid_model_usage' => 'not_included_unless_explicitly_listed',
        'managed_hosting' => 'not_included_unless_explicitly_listed',
        'resale' => 'not_included_unless_explicitly_listed',
        'redistribution' => 'not_included_unless_explicitly_listed',
        'product_embedding' => 'not_included_unless_explicitly_listed',
    ];

    /**
     * Explicitly granted hosted resources. Empty for v1: a hosted/metered right is
     * included only when an explicit operator-approved registry version lists it.
     */
    public const GRANTED = [];

    /** Canonical ordered exclusion list for fixtures and evidence. */
    public static function exclusionList(): array
    {
        return array_keys(self::EXCLUSIONS);
    }

    /** Fail-closed inclusion check: unknown and unlisted resources are denied. */
    public static function isIncluded(string $resource): bool
    {
        return in_array($resource, self::GRANTED, true);
    }

    /**
     * Public-safe stable denial: raises HOSTED_RESOURCE_NOT_INCLUDED with the documented
     * recovery action (`use_local_execution_or_obtain_hosted_resource_access`).
     */
    public static function assertIncluded(string $resource): void
    {
        if (!self::isIncluded($resource)) {
            throw new DomainException(self::ERROR_CODE);
        }
    }

    /** Deterministic digest over the canonical frozen record; identical for every v1 license. */
    public static function digest(): string
    {
        return hash('sha256', FocusaSpec172LicenseTypeProjectionMigration::encodeCanonical([
            'schema' => self::SCHEMA,
            'version' => self::VERSION,
            'product' => self::PRODUCT,
            'license_type' => self::LICENSE_TYPE,
            'authority' => self::AUTHORITY,
            'section' => self::SECTION,
            'exclusions' => self::EXCLUSIONS,
            'granted' => self::GRANTED,
        ]));
    }
}
