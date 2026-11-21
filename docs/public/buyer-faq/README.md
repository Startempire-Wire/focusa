# Focusa buyer FAQ — authoritative public-copy package

This package is a **proposed correction**. The WordPress page is still live independently; merging the repository PR does not change it.

- `authority.v1.json` + `authority.schema.v1.json`: founder/ownership, product/legal provenance, offer, confidence/release and outbound-send policy with JSON Schema.
- `buyer-faq.v1.json` + `buyer-faq.schema.v1.json`: 55 existing visible buyer questions, original order, question-level provenance and proof gates; old answers retained as SHA-256 migration digests only, not deprecated public machine answers.
- `FAQPage.v1.jsonld`: Schema.org FAQPage semantic representation; JSON-LD does not guarantee search rich results, and must exactly match visible copy.
- `wordpress-page-14.proposed.html`: WordPress-rendered-source snapshot with a surgical correction of 13 answer bodies and introduction; reference only, **not a blind POST replacement of native post_content**.
- `audit-2026-11-20.md`: per-question audit and provenance.

## Release steps

1. Human owner reviews identity and terms. DBA legal registrant is intentionally not invented.
2. Implementing agent exports WordPress page 14 source (prefer WP-CLI `wp post get 14 --field=post_content`) and takes a backup/version record.
3. Apply corrected answers **by matching each exact question and current answer**, refusing to write if the live content drifted. Preserve styles, details/summary tags, page layout, all 55 questions and approved pricing/support scope.
4. Deploy machine artifacts from the same approved source to the WordPress host with `application/json` and a current version. Add `FAQPage` JSON-LD only when it matches visitor-visible answers. Use `FAQPage` semantics, **never** `QAPage` for this multi-question editorial FAQ.
5. Verify `/faqs/` and both machine endpoints over HTTPS, counts, exact answers, schema, 200 responses, metadata/stray footer separately, and a safe rollback.
6. Plug the outbound gate into every real email, CRM, and agent send action. An advisory file or a prompt is not a substitute for a pre-send enforced check and human approval.

## Hard rules

- Never describe Verious Smith III as **only** or **merely** a licensing contact. State he is the creator and principal architect of Focusa and owner directing Startempire Wire's development.
- Do not treat Philoveracity as unrelated; it is the same founder's longstanding agency/DBA since 2010.
- Keep business structures distinct from unverified legal DBA registration details.
- Never downgrade approved responsibilities or commercially accepted terms without explicit owner approval; preserve 12 months of Operator Success.
- Classify product facts as specified, source-implemented, release-verified, environment-verified or customer-accepted. No upward inference.

## Safe WordPress publication commands

From the exact existing WordPress origin, with WP-CLI access and the Focusa repository checked out:

```bash
WP_ROOT=/absolute/path/to/wordpress bash scripts/publish_focusa_buyer_faq_wordpress.sh --dry-run
# Only after owner review of PR #726 and source diff:
FOCUSA_FAQ_OWNER_APPROVED=YES-REVIEWED-PR-726 WP_ROOT=/absolute/path/to/wordpress \\
  bash scripts/publish_focusa_buyer_faq_wordpress.sh --apply
```

The script snapshots, compares revision/content before writes, patches only accepted question/answer bodies, saves a private backup, and verifies exact post-write parity. It does **not** silently publish the proposed machine endpoints or inject JSON-LD: those require an authorized host integration and a live reader/agent parity test.
