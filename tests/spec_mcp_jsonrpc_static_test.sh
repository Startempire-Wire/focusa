#!/usr/bin/env bash
# spec_mcp_jsonrpc_static_test.sh
#
# Static guard for focusa-112-mcp-jsonrpc.
# Backward compatibility: route is additive (/mcp and /v1/mcp); existing HTTP
# routes remain the authority. Spec141 projects the curated generated catalog
# and bridges calls through scoped REST so auth/permission checks are preserved.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MCP="$ROOT_DIR/crates/focusa-api/src/routes/mcp.rs"
MOD="$ROOT_DIR/crates/focusa-api/src/routes/mod.rs"
SERVER="$ROOT_DIR/crates/focusa-api/src/server.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[ -f "$MCP" ] || fail "missing routes/mcp.rs"
pass "mcp.rs route file exists"

# Additive routes mounted
grep -q 'route("/mcp", post(handle_jsonrpc))' "$MCP" \
  || fail "mcp.rs missing POST /mcp route"
grep -q 'route("/v1/mcp", post(handle_jsonrpc))' "$MCP" \
  || fail "mcp.rs missing POST /v1/mcp route"
grep -q 'pub mod mcp;' "$MOD" \
  || fail "routes/mod.rs missing pub mod mcp"
grep -q 'routes::mcp::router()' "$SERVER" \
  || fail "server.rs missing routes::mcp::router() merge"
pass "MCP JSON-RPC routes mounted additively"

# JSON-RPC 2.0 envelope support
grep -q 'jsonrpc' "$MCP" \
  || fail "mcp.rs missing jsonrpc envelope markers"
grep -q 'Invalid Request' "$MCP" \
  || fail "mcp.rs missing invalid request error"
grep -q 'Method not found' "$MCP" \
  || fail "mcp.rs missing method-not-found error"
grep -q -- '-32600' "$MCP" \
  || fail "mcp.rs missing JSON-RPC invalid request code -32600"
grep -q -- '-32601' "$MCP" \
  || fail "mcp.rs missing JSON-RPC method/tool not found code -32601"
grep -q -- '-32602' "$MCP" \
  || fail "mcp.rs missing JSON-RPC invalid params code -32602"
pass "JSON-RPC 2.0 success/error envelope markers present"

# Required MCP methods
grep -q '"initialize"' "$MCP" \
  || fail "MCP bridge missing initialize method"
grep -q '"tools/list"' "$MCP" \
  || fail "MCP bridge missing tools/list method"
grep -q '"tools/call"' "$MCP" \
  || fail "MCP bridge missing tools/call method"
pass "MCP initialize/tools/list/tools/call methods present"

# Spec141 generated catalog, pagination, structured calls, and scope preservation
PROJECTION="$ROOT_DIR/docs/contracts/spec141/generated-capability-v2/mcp-tools.json"
[ -f "$PROJECTION" ] || fail "missing generated MCP capability projection"
jq -e '.schema == "focusa.mcp_tool_projection.v2" and (.tools | length) > 1' "$PROJECTION" >/dev/null \
  || fail "generated MCP projection must expose the curated catalog"
jq -e 'all(.tools[]; has("inputSchema") and has("outputSchema") and has("annotations") and (._meta.rest | length) > 0)' "$PROJECTION" >/dev/null \
  || fail "every MCP tool needs schemas, annotations, and a REST authority binding"
grep -q 'listChanged' "$MCP" \
  || fail "MCP bridge must advertise listChanged"
grep -q 'nextCursor' "$MCP" \
  || fail "MCP bridge must paginate tools/list"
grep -q 'structuredContent' "$MCP" \
  || fail "MCP bridge must return structuredContent"
grep -q '\.get(AUTHORIZATION)' "$MCP" && grep -q 'request = request.header(AUTHORIZATION.as_str(), value)' "$MCP" \
  || fail "MCP bridge must forward authorization to scoped REST authority"
grep -q 'bind_path' "$MCP" \
  || fail "MCP bridge must bind and validate generated route parameters"
pass "MCP bridge exposes generated structured tools through scoped REST authority"

echo "✓ All MCP JSON-RPC static checks passed"