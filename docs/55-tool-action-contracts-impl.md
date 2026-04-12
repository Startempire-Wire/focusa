# Tool and Action Contracts — Implementation Notes

## Purpose
This document details the implementation of SPEC 55 requirements.

## Contract Requirements Implementation

### Input Schema ✅
- All tool inputs are explicit, typed, and validated via serde deserialization
- Minimal inputs required for each action

### Output Schema ✅
- All tool outputs include result status, affected objects, evidence refs
- Side-effect summary included where applicable

### Side Effects ✅
| Tool | Side Effects |
|------|---------------|
| bash | File system changes, process execution |
| read | No side effects (read-only) |
| edit | File modifications |
| write | File creation/overwrite |
| grep/find | No side effects |
| mcp | External tool invocation |

### Failure Modes ✅
| Failure Type | Status | Handling |
|-------------|--------|----------|
| Validation failure | ✅ | 400 Bad Request with details |
| Permission failure | ✅ | 403 Forbidden |
| Execution failure | ✅ | 500 Internal Server Error |
| Timeout | ✅ | Request timeout handling |
| Partial success | ✅ | Partial result with status |

### Idempotency ✅
| Tool | Idempotent | Policy |
|------|-----------|--------|
| read | ✅ | Same input → same output |
| edit | ⚠️ | Idempotent with -y flag after 10s |
| write | ❌ | Creates/overwrites |
| bash | ❌ | Depends on command |
| grep/find | ✅ | Same input → same output |

### Verification Hooks ✅
- Pre-execution: Input validation
- Post-execution: Result verification via return status
- Integration: Focus state update after tool execution

### Degraded Fallback ✅
- Focusa unavailable → graceful degradation
- API timeout → fallback to local operations
- Error signals emitted for observability

## Success Condition
SPEC 55 requirements implemented: typed inputs/outputs, failure modes, idempotency, verification hooks.
