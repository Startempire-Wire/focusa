#!/usr/bin/env python3
import argparse
import json
import sys
import time
import urllib.error
import urllib.request
import uuid
from datetime import datetime, timezone


def req(base_url, path, method="GET", body=None, headers=None, timeout=5):
    data = None
    if body is not None:
        data = json.dumps(body).encode("utf-8")
    req_headers = {"Content-Type": "application/json"}
    if headers:
        req_headers.update(headers)
    request = urllib.request.Request(
        url=f"{base_url}{path}", data=data, method=method, headers=req_headers
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as resp:
            raw = resp.read().decode("utf-8")
            return resp.status, dict(resp.headers), raw
    except urllib.error.HTTPError as e:
        raw = e.read().decode("utf-8")
        return e.code, dict(e.headers), raw


def parse_json(raw):
    try:
        return json.loads(raw), None
    except Exception as e:
        return None, str(e)


def check_fields(obj, required):
    return [k for k in required if k not in obj]


def main():
    parser = argparse.ArgumentParser(description="Focusa API contract probe")
    parser.add_argument("--base-url", default="http://127.0.0.1:8787")
    parser.add_argument("--out", default="/tmp/focusa_api_spec_audit.json")
    args = parser.parse_args()

    report = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "base_url": args.base_url,
        "checks": [],
        "failures": [],
        "pass": False,
    }

    def add_check(name, ok, details):
        report["checks"].append({"name": name, "ok": ok, "details": details})
        if not ok:
            report["failures"].append({"name": name, "details": details})

    # 1) /v1/health fields
    status, headers, raw = req(args.base_url, "/v1/health")
    body, err = parse_json(raw)
    ok = status == 200 and body is not None and not check_fields(body, ["ok", "version", "uptime_ms"])
    add_check("health_contract", ok, {"status": status, "body": body if body else raw, "parse_error": err})

    # 2) /v1/status fields
    status, headers, raw = req(args.base_url, "/v1/status")
    body, err = parse_json(raw)
    required = ["active_frame", "stack_depth", "worker_status", "last_event_ts", "prompt_stats"]
    missing = [] if body is None else check_fields(body, required)
    ok = status == 200 and body is not None and not missing
    add_check("status_contract", ok, {"status": status, "missing": missing, "body": body if body else raw, "parse_error": err})

    # 3) prompt assemble key compatibility
    payload = {
        "turn_id": f"probe-{uuid.uuid4()}",
        "raw_user_input": "Contract probe input",
        "format": "string",
        "budget": 500,
    }
    status, headers, raw = req(args.base_url, "/v1/prompt/assemble", method="POST", body=payload)
    body, err = parse_json(raw)
    required = ["assembled", "stats", "handles_used", "assembled_prompt", "context_stats"]
    missing = [] if body is None else check_fields(body, required)
    ok = status == 200 and body is not None and not missing
    add_check("prompt_assemble_contract", ok, {"status": status, "missing": missing, "body": body if body else raw, "parse_error": err})

    # 4) error envelope for framework/body parse rejection
    status, headers, raw = req(args.base_url, "/v1/prompt/assemble", method="POST", body={"turn_id": "bad"})
    body, err = parse_json(raw)
    required = ["code", "message", "correlation_id"]
    missing = [] if body is None else check_fields(body, required)
    ok = status == 422 and body is not None and not missing
    add_check("error_envelope_contract", ok, {"status": status, "missing": missing, "body": body if body else raw, "parse_error": err})

    # 5) fixture endpoints should not 422
    fixture_calls = [
        ("/v1/turn/start", {"turn_id": f"fixture-{uuid.uuid4()}", "harness_name": "openclaw", "adapter_id": "openclaw", "timestamp": datetime.now(timezone.utc).isoformat()}),
        ("/v1/gate/signal", {"kind": "user_input_received", "summary": "probe"}),
        ("/v1/ecs/store", {"kind": "text", "label": "probe", "content_b64": "cHJvYmU="}),
        ("/v1/memory/procedural/reinforce", {"rule_id": "probe-rule"}),
    ]
    fixture_results = []
    fixture_ok = True
    for path, body_in in fixture_calls:
        st, _, rw = req(args.base_url, path, method="POST", body=body_in)
        fixture_results.append({"path": path, "status": st, "body": rw[:300]})
        if st == 422:
            fixture_ok = False
    add_check("fixture_no_422", fixture_ok, fixture_results)

    # 6) turn_complete idempotency
    turn_id = f"probe-turn-{uuid.uuid4()}"
    start_body = {
        "turn_id": turn_id,
        "harness_name": "probe",
        "adapter_id": "probe",
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }
    req(args.base_url, "/v1/turn/start", method="POST", body=start_body)

    complete_body = {
        "turn_id": turn_id,
        "assistant_output": "done",
        "artifacts": [],
        "errors": [],
    }
    s1, _, r1 = req(args.base_url, "/v1/turn/complete", method="POST", body=complete_body)
    time.sleep(0.05)
    s2, _, r2 = req(args.base_url, "/v1/turn/complete", method="POST", body=complete_body)
    b2, e2 = parse_json(r2)
    ok = s1 == 200 and s2 == 200 and b2 is not None and b2.get("duplicate") is True
    add_check("turn_complete_idempotency", ok, {"first_status": s1, "second_status": s2, "second_body": b2 if b2 else r2, "parse_error": e2})

    # 7) reflection status surface
    s, _, r = req(args.base_url, "/v1/reflect/status")
    b, e = parse_json(r)
    miss = [] if b is None else check_fields(b, ["enabled", "scheduler", "guardrails", "telemetry"])
    guardrail_miss = [] if b is None else check_fields(b.get("guardrails", {}), ["low_confidence_threshold", "no_delta_min_event_delta"])
    telemetry_miss = [] if b is None else check_fields(b.get("telemetry", {}), ["stop_reason_counts"])
    ok = s == 200 and b is not None and not miss and not guardrail_miss and not telemetry_miss
    add_check("reflection_status_contract", ok, {"status": s, "missing": miss, "guardrail_missing": guardrail_miss, "telemetry_missing": telemetry_miss, "body": b if b else r, "parse_error": e})

    # 8) reflection idempotency + history
    rk = f"reflect-{uuid.uuid4()}"
    body = {"mode": "manual", "idempotency_key": rk, "window": "1h", "budget": 300}
    s1, _, r1 = req(args.base_url, "/v1/reflect/run", method="POST", body=body)
    s2, _, r2 = req(args.base_url, "/v1/reflect/run", method="POST", body=body)
    b1, e1 = parse_json(r1)
    b2, e2 = parse_json(r2)
    hist_s, _, hist_r = req(args.base_url, "/v1/reflect/history?limit=20")
    hist_b, hist_e = parse_json(hist_r)
    has_key = False
    if isinstance(hist_b, dict):
        for it in hist_b.get("items", []):
            if it.get("idempotency_key") == rk:
                has_key = True
                break
    ok = (
        s1 == 200 and s2 == 200 and
        isinstance(b1, dict) and isinstance(b2, dict) and
        b1.get("duplicate") is False and b2.get("duplicate") is True and
        hist_s == 200 and has_key
    )
    add_check("reflection_idempotency", ok, {
        "first_status": s1,
        "second_status": s2,
        "first_parse_error": e1,
        "second_parse_error": e2,
        "history_status": hist_s,
        "history_parse_error": hist_e,
        "history_contains_key": has_key,
    })

    # 9) reflection history stop_reason filter + time-range filter
    hs, _, hr = req(args.base_url, "/v1/reflect/history?limit=20&stop_reason=low_confidence")
    hb, he = parse_json(hr)
    ms, _, mr = req(args.base_url, "/v1/reflect/history?limit=20&mode=scheduled")
    mb, me = parse_json(mr)
    cs, _, cr = req(args.base_url, "/v1/reflect/history?limit=2")
    cb, ce = parse_json(cr)
    ts, _, tr = req(args.base_url, "/v1/reflect/history?limit=20&since=2999-01-01T00:00:00Z&until=2999-01-02T00:00:00Z")
    tb, te = parse_json(tr)
    bs, _, br = req(args.base_url, "/v1/reflect/history?limit=20&since=not-a-time")
    bb, be = parse_json(br)
    ims, _, imr = req(args.base_url, "/v1/reflect/history?limit=20&mode=bogus")
    imb, ime = parse_json(imr)

    valid = (
        hs == 200 and isinstance(hb, dict) and isinstance(hb.get("items", []), list) and isinstance(hb.get("applied_filters", {}), dict) and ("limit" in hb.get("applied_filters", {}))
        and ms == 200 and isinstance(mb, dict) and isinstance(mb.get("items", []), list)
        and cs == 200 and isinstance(cb, dict) and "next_cursor" in cb
        and ts == 200 and isinstance(tb, dict) and isinstance(tb.get("items", []), list)
        and bs == 400 and isinstance(bb, dict) and bb.get("code") == "invalid_time_filter"
        and ims == 400 and isinstance(imb, dict) and imb.get("code") == "invalid_mode_filter"
    )
    add_check("reflection_history_filter_contract", valid, {
        "status": hs,
        "parse_error": he,
        "mode_filter_status": ms,
        "mode_filter_parse_error": me,
        "cursor_status": cs,
        "cursor_parse_error": ce,
        "time_filter_status": ts,
        "time_filter_parse_error": te,
        "bad_time_status": bs,
        "bad_time_parse_error": be,
        "bad_mode_status": ims,
        "bad_mode_parse_error": ime,
    })

    # 10) reflection scheduler config + tick
    gs, _, gr = req(args.base_url, "/v1/reflect/scheduler")
    gb, ge = parse_json(gr)
    miss = [] if gb is None else check_fields(gb, ["enabled", "interval_seconds", "max_iterations_per_window", "cooldown_seconds", "low_confidence_threshold", "no_delta_min_event_delta", "telemetry"])

    us, _, ur = req(args.base_url, "/v1/reflect/scheduler", method="POST", body={"enabled": True, "cooldown_seconds": 3600})
    ub, ue = parse_json(ur)

    t1s, _, t1r = req(args.base_url, "/v1/reflect/scheduler/tick", method="POST", body={"window": "1h"})
    t2s, _, t2r = req(args.base_url, "/v1/reflect/scheduler/tick", method="POST", body={"window": "1h"})
    t1b, t1e = parse_json(t1r)
    t2b, t2e = parse_json(t2r)

    ok = (
        gs == 200 and gb is not None and not miss and
        us == 200 and isinstance(ub, dict) and ub.get("status") == "updated" and
        t1s == 200 and isinstance(t1b, dict) and t1b.get("status") in ["accepted", "skipped"] and
        t2s == 200 and isinstance(t2b, dict) and t2b.get("status") in ["accepted", "skipped"]
    )
    add_check("reflection_scheduler_contract", ok, {
        "get_status": gs,
        "get_missing": miss,
        "get_parse_error": ge,
        "update_status": us,
        "update_parse_error": ue,
        "tick1_status": t1s,
        "tick1_parse_error": t1e,
        "tick2_status": t2s,
        "tick2_parse_error": t2e,
        "tick2_status_value": None if not isinstance(t2b, dict) else t2b.get("status"),
        "tick2_reason": None if not isinstance(t2b, dict) else t2b.get("reason"),
    })

    # 11) trust metrics PATCH endpoint (§35.7)
    s, _, r = req(args.base_url, "/v1/trust/metrics", method="PATCH", body={"event": "test_correction", "detail": "probe test"})
    b, e = parse_json(r)
    ok = s == 200 and b is not None and b.get("status") == "recorded" and b.get("event") == "test_correction"
    add_check("trust_metrics_contract", ok, {"status": s, "body": b if b else r, "parse_error": e})


    # 12) ECS store returns handle.id and handle appears in handles list
    label = f"probe-{uuid.uuid4()}"
    s, _, r = req(args.base_url, "/v1/ecs/store", method="POST", body={"kind": "text", "label": label, "content_b64": "cHJvYmU="})
    b, e = parse_json(r)
    handle_id = None if b is None else b.get("id")
    ok = s == 200 and b is not None and b.get("status") == "accepted" and b.get("id") is not None
    # Verify handle appears in handles list
    hs, _, hr = req(args.base_url, "/v1/ecs/handles")
    hb, he = parse_json(hr)
    found = False
    if isinstance(hb, dict) and isinstance(hb.get("handles"), list):
        for h in hb["handles"]:
            if h.get("id") == handle_id or h.get("label") == label:
                found = True
                break
    add_check("ecs_store_handles_contract", ok and found, {"store_status": s, "handle_id": handle_id, "found_in_list": found, "store_body": b if b else r, "store_error": e, "handles_status": hs, "handles_body": hb if hb else hr, "handles_error": he})

    # 13) SSE turn tracking — turn start emits TurnStarted event
    # SSE is a streaming endpoint — use a short read to verify it responds 200
    try:
        req_headers = {"Accept": "text/event-stream"}
        request = urllib.request.Request(
            url=f"{args.base_url}/v1/events/stream",
            method="GET",
            headers=req_headers
        )
        with urllib.request.urlopen(request, timeout=3) as resp:
            ok = resp.status == 200
            add_check("sse_turn_tracking_contract", ok, {"status": resp.status})
    except Exception as ex:
        add_check("sse_turn_tracking_contract", False, {"error": str(ex)})

    report["pass"] = len(report["failures"]) == 0


    with open(args.out, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    print(json.dumps({"pass": report["pass"], "failures": len(report["failures"]), "out": args.out}))
    return 0 if report["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
