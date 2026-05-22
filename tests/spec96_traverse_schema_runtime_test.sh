#!/bin/bash
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
TMP_DIR="${TMPDIR:-/tmp}/focusa-spec96-traverse-schema.$$"
mkdir -p "$TMP_DIR"
trap 'rm -rf "$TMP_DIR"' EXIT

curl -fsS --max-time 5 -H 'content-type: application/json' \
  -d '{"surface":"workpoints","selector":"current","limit":1,"include_rehydrate_refs":true}' \
  "${BASE_URL}/v1/traverse" >"$TMP_DIR/current.json"

jq -e '.status=="completed" and .items[0].anchor and .items[0].tag and .items[0].freshness=="live" and .items[0].data.workpoint_id and .traversal.caps.limit and (.traversal|has("rehydrate_refs")) and .tag_scheme.includes_anchor==true and .tag_scheme.includes_surface_version==true' "$TMP_DIR/current.json" >/dev/null

TAG="$(jq -r '.items[0].tag' "$TMP_DIR/current.json")"
ANCHOR="$(jq -r '.items[0].anchor' "$TMP_DIR/current.json")"
cat >"$TMP_DIR/verify-body.json" <<JSON
{"surface":"workpoints","selector":"tags_verify","tags":[{"anchor":"${ANCHOR}","tag":"${TAG}","ordinal":0}]}
JSON
curl -fsS --max-time 5 -H 'content-type: application/json' \
  -d @"$TMP_DIR/verify-body.json" \
  "${BASE_URL}/v1/traverse/verify-tags" >"$TMP_DIR/verify.json"

jq -e '.status=="completed" and (.verified_tags|length)==1 and (.stale_tags|length)==0 and .traversal.verified_tags[0].verified==true' "$TMP_DIR/verify.json" >/dev/null

curl -fsS --max-time 5 -H 'content-type: application/json' \
  -d '{"surface":"ontology","selector":"window","limit":1,"include_payload":true,"include_rehydrate_refs":true,"budget_tokens":1000}' \
  "${BASE_URL}/v1/traverse" >"$TMP_DIR/include-payload.json"

jq -e '.traversal.metadata.include_full_payload==true or .failure_class=="resource_exhausted"' "$TMP_DIR/include-payload.json" >/dev/null

curl -fsS --max-time 5 -H 'content-type: application/json' \
  -d '{"surface":"not_a_surface","selector":"window"}' \
  "${BASE_URL}/v1/traverse" >"$TMP_DIR/invalid.json"

jq -e '.status=="validation_rejected" and .failure_class=="validation_rejected"' "$TMP_DIR/invalid.json" >/dev/null

echo "SPEC96 traverse schema runtime test: PASS"
