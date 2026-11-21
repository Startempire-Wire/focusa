#!/usr/bin/env bash
# Safe, guarded WordPress publication of the owner-reviewed FAQ contract.
# Run on the authenticated origin (not via a public unauthenticated API).
set -euo pipefail

usage() {
  echo "Usage: WP_ROOT=/path/to/wordpress $0 --dry-run|--apply"
  echo "  --apply requires FOCUSA_FAQ_OWNER_APPROVED=YES-REVIEWED-PR-726"
}
mode="${1:---dry-run}"
if [[ "${mode}" != "--dry-run" && "${mode}" != "--apply" ]]; then usage; exit 2; fi
: "${WP_ROOT:?Set WP_ROOT to the exact existing WordPress installation path}"
command -v wp >/dev/null || { echo "WP-CLI not found; refusing publication" >&2; exit 2; }
command -v python3 >/dev/null || { echo "python3 not found; refusing publication" >&2; exit 2; }
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT
before="${tmp}/page-14-before.html"
after="${tmp}/page-14-approved.html"
observed="${tmp}/page-14-recheck.html"
wp --path="${WP_ROOT}" post get 14 --field=post_content > "${before}"
baseline_modified="$(wp --path="${WP_ROOT}" post get 14 --field=post_modified_gmt)"
python3 "${root}/scripts/focusa_public_claims.py" --check
python3 "${root}/scripts/focusa_public_claims.py" --patch-wordpress "${before}" --output "${after}"
echo "DRY-RUN DIFF (all 55 questions must remain in the same order):"
diff -u "${before}" "${after}" || test "$?" -eq 1
if [[ "${mode}" == "--dry-run" ]]; then
  echo "No WordPress writes. Review PR #726 and re-run with --apply after explicit owner approval."
  exit 0
fi
[[ "${FOCUSA_FAQ_OWNER_APPROVED:-}" == "YES-REVIEWED-PR-726" ]] || { echo "Missing exact owner approval flag; refusing write" >&2; exit 2; }
# Re-read immediately before applying to reject concurrent edits or unreviewed drift.
wp --path="${WP_ROOT}" post get 14 --field=post_content > "${observed}"
now_modified="$(wp --path="${WP_ROOT}" post get 14 --field=post_modified_gmt)"
if [[ "${baseline_modified}" != "${now_modified}" ]] || ! cmp -s "${before}" "${observed}"; then
  echo "Page 14 changed since dry-run; refusing write" >&2
  exit 2
fi
# WP creates its own revision. Keep a timestamped backup outside the webroot.
backup_root="${FOCUSA_FAQ_BACKUP_DIR:-$HOME/focusa-faq-publish-backups}"
mkdir -p "${backup_root}"
chmod 700 "${backup_root}"
backup="${backup_root}/focusa-faq-page14-$(date -u +%Y%m%dT%H%M%SZ).html"
cp "${before}" "${backup}"
chmod 600 "${backup}"
wp --path="${WP_ROOT}" post update 14 --post_content="$(cat "${after}")" >/dev/null
wp --path="${WP_ROOT}" post get 14 --field=post_content > "${observed}"
python3 "${root}/scripts/focusa_public_claims.py" --patch-wordpress "${observed}" --output "${tmp}/page-14-postverify.html"
cmp -s "${observed}" "${tmp}/page-14-postverify.html" || { echo "Post-publish verification failed; inspect immediately and restore backup if required" >&2; exit 2; }
echo "WordPress page 14 updated, preserved native markup and all 55 questions."
echo "Backup: ${backup}"
echo "Next: independently verify live https://focusa.dev/faqs/ and public machine endpoints. This script does not provision JSON-LD/.well-known files."
