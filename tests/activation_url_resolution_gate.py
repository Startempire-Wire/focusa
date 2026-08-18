#!/usr/bin/env python3
"""315: activation client preserves WordPress namespace path."""
import re
from pathlib import Path
p = Path("crates/focusa-license/src/activation_http.rs").read_text()
assert "trim_start_matches('/')" in p, "endpoint must strip leading slash for Url::join"
assert "X-Request-Id" in p and "Idempotency-Key" in p, "315/316 headers missing"
assert "registration_id" in p, "WireStartReply must carry registration_id"
# Verify constants still frozen but endpoint now relative via trim
assert 'pub const START: &str = "/v1/activation/start"' in p, "constants stay frozen"
# Check client consumes server id
c = Path("crates/focusa-license/src/activation_client.rs").read_text()
assert "server_registration_id" in c and "unwrap_or_else" in c, "must consume authority registration_id"
print("315/316 minimal gate PASS")
