#!/usr/bin/env python3
"""Private Google Messages adapter behind connector-neutral Focusa contracts."""
from __future__ import annotations

from datetime import datetime, timezone
import fcntl
import hashlib
import hmac
import json
import os
from pathlib import Path
import re
import secrets
import signal
import threading
import time
import urllib.parse
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import websocket

PORT = int(os.environ.get("FOCUSA_SMS_BROKER_PORT", "8794"))
CDP_PORT = int(os.environ.get("FOCUSA_SMS_CDP_PORT", "9334"))
TOKEN_FILE = Path(os.environ.get("FOCUSA_SMS_BROKER_TOKEN_FILE", "/run/credentials/focusa-sms-appliance/sms-broker-token"))
STATE_DIR = Path(os.environ.get("FOCUSA_SMS_STATE_DIR", "/var/lib/focusa-sms-broker"))
RUNTIME_DIR = Path(os.environ.get("FOCUSA_SMS_RUNTIME_DIR", "/run/focusa-sms-broker"))
GRANTS_FILE = Path(os.environ.get("FOCUSA_SMS_GRANTS_FILE", "/run/credentials/focusa-sms-appliance/sms-grants"))
TARGETS_FILE = Path(os.environ.get("FOCUSA_SMS_TARGETS_FILE", "/run/credentials/focusa-sms-appliance/sms-targets"))
POLICY_FILE = Path(os.environ.get("FOCUSA_SMS_PROVIDER_POLICY_FILE", "/etc/focusa/sms-provider-policy.json"))
GRANT_USAGE_FILE = STATE_DIR / "grant-usage.json"
GRANT_USAGE_LOCK = STATE_DIR / ".grant-usage.lock"
SEND_LEDGER_FILE = STATE_DIR / "send-idempotency.json"
SEND_LEDGER_LOCK = STATE_DIR / ".send-idempotency.lock"
AUDIT_FILE = STATE_DIR / "audit.jsonl"
CONNECTOR_ID = os.environ.get("FOCUSA_SMS_CONNECTOR_ID", "communications-1")


def secure_file(path: Path, *, minimum_size: int = 1) -> None:
    info = path.lstat()
    if not path.is_file() or path.is_symlink() or info.st_uid != os.geteuid() or info.st_mode & 0o077:
        raise SystemExit("broker authority file permissions invalid")
    if info.st_size < minimum_size:
        raise SystemExit("broker authority file unavailable")


secure_file(TOKEN_FILE, minimum_size=32)
TOKEN = TOKEN_FILE.read_text(encoding="utf-8").strip()
THREADS: dict[str, int] = {}
CHALLENGES: dict[str, dict] = {}
EVENTS: list[dict] = []
LOCK = threading.RLock()


def envelope(ok: bool, status: str, summary: str, **data: object) -> dict:
    return {"schema": "focusa.tool_result_v1", "canonical": True, "ok": ok, "status": status, "summary": summary, **data}


def opaque(kind: str, value: str) -> str:
    return kind + "-" + hmac.new(TOKEN.encode(), value.encode(), hashlib.sha256).hexdigest()[:24]


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def private_state_dir() -> None:
    STATE_DIR.mkdir(mode=0o700, parents=True, exist_ok=True)
    info = STATE_DIR.lstat()
    if not STATE_DIR.is_dir() or STATE_DIR.is_symlink() or info.st_uid != os.geteuid() or info.st_mode & 0o077:
        raise PermissionError("broker state directory unsafe")


def audit(action: str, status: str, *, consumer_ref: str = "", grant_id: str = "", target_handle: str | None = None, failure_class: str | None = None) -> None:
    event = {
        "schema": "focusa.sms_audit.v1", "audit_id": secrets.token_hex(12), "action": action,
        "consumer_ref": consumer_ref, "grant_id": grant_id, "connector_id": CONNECTOR_ID,
        "target_handle": target_handle, "status": status, "failure_class": failure_class,
        "occurred_at": now_iso(),
    }
    private_state_dir()
    fd = os.open(AUDIT_FILE, os.O_CREAT | os.O_APPEND | os.O_WRONLY | os.O_NOFOLLOW, 0o600)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        os.write(fd, (json.dumps(event, separators=(",", ":")) + "\n").encode())
        os.fsync(fd)
    finally:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)
    with LOCK:
        EVENTS.append(event)
        del EVENTS[:-500]


def audit_events(consumer_ref: str, allow_all: bool, limit: int) -> list[dict]:
    if not AUDIT_FILE.exists():
        return []
    secure_file(AUDIT_FILE)
    with AUDIT_FILE.open("rb") as stream:
        stream.seek(max(0, AUDIT_FILE.stat().st_size - 1_048_576))
        if stream.tell(): stream.readline()
        rows = [json.loads(line) for line in stream if line.strip()]
    if not allow_all:
        rows = [row for row in rows if row.get("consumer_ref") == consumer_ref]
    return rows[-limit:]


def load_json_authority(path: Path) -> dict:
    secure_file(path)
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError("authority payload invalid")
    return value


def atomic_json(path: Path, value: dict) -> None:
    temporary = path.with_name(f".{path.name}.{os.getpid()}.{secrets.token_hex(4)}")
    fd = os.open(temporary, os.O_CREAT | os.O_EXCL | os.O_WRONLY | os.O_NOFOLLOW, 0o600)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as stream:
            json.dump(value, stream, separators=(",", ":"))
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        if temporary.exists():
            temporary.unlink()


def authorize(grant_id: str, consumer_ref: str, capability: str, *, target_handle: str | None = None, provider: str | None = None, thread_handle: str | None = None, recipient_handle: str | None = None, consume: bool = False) -> dict:
    payload = load_json_authority(GRANTS_FILE)
    grants = payload.get("grants", [])
    grant = next((item for item in grants if item.get("grant_id") == grant_id), None)
    if not isinstance(grant, dict):
        raise PermissionError("grant unavailable")
    STATE_DIR.mkdir(mode=0o700, parents=True, exist_ok=True)
    if STATE_DIR.stat().st_uid != os.geteuid() or STATE_DIR.stat().st_mode & 0o077:
        raise PermissionError("grant usage state unsafe")
    fd = os.open(GRANT_USAGE_LOCK, os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        usage = {"schema": "focusa.sms_grant_usage.v1", "uses": {}}
        if GRANT_USAGE_FILE.exists():
            secure_file(GRANT_USAGE_FILE)
            usage = json.loads(GRANT_USAGE_FILE.read_text(encoding="utf-8"))
        used = max(int(grant.get("use_count_used", 0)), int(usage.get("uses", {}).get(grant_id, 0)))
        scope = grant.get("scope", {})
        reasons = []
        if grant.get("schema") != "focusa.sms_grant.v1": reasons.append("schema")
        if grant.get("status") != "active": reasons.append("status")
        if grant.get("consumer_ref") != consumer_ref: reasons.append("consumer")
        if capability not in grant.get("capabilities", []): reasons.append("capability")
        if scope.get("connector_id") != CONNECTOR_ID: reasons.append("connector")
        if grant.get("expires_at", "") <= now_iso(): reasons.append("expired")
        if used >= int(grant.get("use_count_allowed", 0)): reasons.append("exhausted")
        if target_handle is not None and scope.get("target_handle") != target_handle: reasons.append("target")
        if provider is not None and scope.get("provider") != provider: reasons.append("provider")
        if thread_handle is not None and thread_handle not in scope.get("thread_handles", []): reasons.append("thread")
        if recipient_handle is not None and recipient_handle not in scope.get("recipient_handles", []): reasons.append("recipient")
        if reasons:
            raise PermissionError("grant rejected")
        if consume:
            usage.setdefault("uses", {})[grant_id] = used + 1
            atomic_json(GRANT_USAGE_FILE, usage)
        return grant
    finally:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)



class Cdp:
    def __init__(self, endpoint: str | None = None):
        base = endpoint or f"http://127.0.0.1:{CDP_PORT}"
        meta = json.load(urllib.request.urlopen(base.rstrip("/") + "/json/version", timeout=3))
        self.ws = websocket.create_connection(meta["webSocketDebuggerUrl"], suppress_origin=True, timeout=10)
        self.next_id = 0

    def call(self, method: str, params: dict | None = None, session: str | None = None) -> dict:
        self.next_id += 1
        request = {"id": self.next_id, "method": method, "params": params or {}}
        if session:
            request["sessionId"] = session
        self.ws.send(json.dumps(request))
        while True:
            response = json.loads(self.ws.recv())
            if response.get("id") == self.next_id:
                if "error" in response or "exceptionDetails" in response.get("result", {}):
                    raise RuntimeError("cdp request rejected")
                return response.get("result", {})

    def attach(self, origin: str) -> str:
        targets = self.call("Target.getTargets").get("targetInfos", [])
        page = next((item for item in targets if item.get("type") == "page" and item.get("url", "").startswith(origin)), None)
        if not page:
            raise RuntimeError("bound target unavailable")
        return self.call("Target.attachToTarget", {"targetId": page["targetId"], "flatten": True})["sessionId"]

    def evaluate(self, session: str, expression: str) -> object:
        return self.call("Runtime.evaluate", {"expression": expression, "returnByValue": True}, session).get("result", {}).get("value")

    def close(self) -> None:
        self.ws.close()


def connector_state() -> dict:
    state_path = STATE_DIR / "connector-state.json"
    secure_file(state_path)
    state = json.loads(state_path.read_text(encoding="utf-8"))
    if state.get("schema") != "focusa.sms_connector_state.v1":
        raise RuntimeError("connector state invalid")
    return state


def browser_health() -> dict:
    state = connector_state()
    if state.get("status") != "ready" or state.get("checkpoint_status") not in {"paired_persisted", "verified_standby"}:
        raise RuntimeError("connector not durably ready")
    client = Cdp()
    try:
        session = client.attach("https://messages.google.com/")
        semantic = client.evaluate(session, "(()=>({path:location.pathname,unable:!!document.querySelector('mw-unable-to-connect-container'),list:!!document.querySelector('mws-conversations-list'),count:document.querySelectorAll('mws-conversation-list-item').length}))()") or {}
        if "/conversations" not in semantic.get("path", "") or semantic.get("unable") or not semantic.get("list"):
            raise RuntimeError("connector semantic readiness failed")
        return {"thread_count": int(semantic.get("count", 0)), "generation": int(state.get("verified_generation", 0)), "checkpoint_status": state.get("checkpoint_status")}
    finally:
        client.close()


def list_threads(limit: int, *, include_snippet: bool = True) -> list[dict]:
    client = Cdp()
    try:
        session = client.attach("https://messages.google.com/")
        rows = client.evaluate(session, "(()=>[...document.querySelectorAll('mws-conversation-list-item')].map((x,i)=>({index:i,name:(x.querySelector('h2.name')?.innerText||'').trim(),snippet:(x.querySelector('.snippet-text')?.innerText||'').trim(),timestamp:(x.querySelector('mws-relative-timestamp')?.innerText||'').trim(),unread:x.classList.contains('unread')||x.querySelector('[aria-label*=unread i]')!==null})))()") or []
        output = []
        with LOCK:
            THREADS.clear()
            for row in rows[:limit]:
                token = opaque("thread", f"{row['index']}\0{row['name']}")
                THREADS[token] = int(row["index"])
                output.append({"thread_handle": token, "display_name": row["name"], "participant_handles": [], "unread_count": 1 if row["unread"] else 0, "last_message_at": None, "snippet": row["snippet"] if include_snippet else None, "relative_timestamp": row["timestamp"]})
        return output
    finally:
        client.close()


def select_thread(thread_handle: str) -> tuple[Cdp, str]:
    with LOCK:
        index = THREADS.get(thread_handle)
    if index is None:
        list_threads(200)
        with LOCK:
            index = THREADS.get(thread_handle)
    if index is None:
        raise KeyError("thread handle unavailable")
    client = Cdp()
    session = client.attach("https://messages.google.com/")
    selected = client.evaluate(session, f"(()=>{{const x=document.querySelectorAll('mws-conversation-list-item a.list-item')[{index}];if(!x)return false;x.click();return true}})()")
    if not selected:
        client.close()
        raise RuntimeError("thread selection failed")
    time.sleep(1)
    return client, session


def read_messages(thread_handle: str, limit: int) -> list[dict]:
    client, session = select_thread(thread_handle)
    try:
        rows = client.evaluate(session, "(()=>[...document.querySelectorAll('[data-e2e-message],mws-message-wrapper,.message-row,.text-msg')].map((x,i)=>({index:i,body:(x.innerText||'').trim(),direction:x.classList.contains('outgoing')?'outgoing':x.classList.contains('incoming')?'incoming':'unknown'})).filter(x=>x.body))()") or []
        if not rows:
            raise RuntimeError("message DOM adapter unavailable")
        return [{"message_handle": opaque("message", f"{thread_handle}\0{x['index']}\0{x['body']}"), "thread_handle": thread_handle, "direction": x["direction"], "sender_handle": None, "recipient_handles": [], "body": x["body"], "sent_at": None} for x in rows[-limit:]]
    finally:
        client.close()


def idempotent_send(grant_id: str, consumer_ref: str, idempotency_key: str, recipient: str, body: str) -> tuple[str, bool]:
    private_state_dir()
    key = hashlib.sha256(f"{grant_id}\0{consumer_ref}\0{idempotency_key}".encode()).hexdigest()
    request_digest = hashlib.sha256(f"{recipient}\0{body}".encode()).hexdigest()
    fd = os.open(SEND_LEDGER_LOCK, os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        ledger = {"schema": "focusa.sms_send_idempotency.v1", "entries": {}}
        if SEND_LEDGER_FILE.exists():
            secure_file(SEND_LEDGER_FILE)
            ledger = json.loads(SEND_LEDGER_FILE.read_text(encoding="utf-8"))
        prior = ledger.get("entries", {}).get(key)
        if prior:
            if prior.get("request_digest") != request_digest:
                raise PermissionError("idempotency key payload mismatch")
            return str(prior["send_handle"]), True
        receipt = send_message(recipient, body)
        ledger.setdefault("entries", {})[key] = {"request_digest": request_digest, "send_handle": receipt, "occurred_at": now_iso()}
        atomic_json(SEND_LEDGER_FILE, ledger)
        return receipt, False
    finally:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)


def send_message(thread_handle: str, body: str) -> str:
    client, session = select_thread(thread_handle)
    try:
        found = client.call("Runtime.evaluate", {"expression": "document.querySelector('mws-message-compose textarea,textarea')"}, session).get("result", {})
        object_id = found.get("objectId")
        if not object_id:
            raise RuntimeError("compose input unavailable")
        client.call("Runtime.callFunctionOn", {"objectId": object_id, "functionDeclaration": "function(v){const s=Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set;s.call(this,v);this.dispatchEvent(new Event('input',{bubbles:true}));return true}", "arguments": [{"value": body}], "returnByValue": True}, session)
        sent = client.evaluate(session, "(()=>{const x=document.querySelector('mws-message-send-button button,button[aria-label*=send i]');if(!x)return false;x.click();return true})()")
        if not sent:
            raise RuntimeError("send control unavailable")
        return opaque("send", secrets.token_hex(16))
    finally:
        client.close()


def provider_candidates(provider: str) -> list[tuple[str, str]]:
    policy = load_json_authority(POLICY_FILE).get("providers", {}).get(provider)
    if not isinstance(policy, dict):
        raise ValueError("provider policy unavailable")
    thread_pattern = re.compile(policy.get("thread_pattern", "(?!)"), re.I)
    otp_pattern = re.compile(policy.get("otp_pattern", r"(?<!\d)(\d{6})(?!\d)"))
    threads = list_threads(200)
    selected = [item for item in threads if thread_pattern.search((item.get("display_name") or "") + " " + (item.get("snippet") or ""))]
    if len(selected) != 1:
        raise RuntimeError("provider thread ambiguous")
    messages = read_messages(selected[0]["thread_handle"], 50)
    result = []
    for message in messages:
        matches = otp_pattern.findall(message["body"])
        for match in matches:
            code = match if isinstance(match, str) else match[0]
            result.append((message["message_handle"], code))
    return result


def inject_target(target_handle: str, value: str) -> None:
    target = load_json_authority(TARGETS_FILE).get("targets", {}).get(target_handle)
    if not isinstance(target, dict):
        raise RuntimeError("target adapter unavailable")
    origin = target.get("origin", "")
    endpoint = target.get("cdp_url", "")
    selector = target.get("input_selector", "")
    submit = target.get("submit_selector")
    if not origin.startswith("https://") or not endpoint.startswith("http://127.0.0.1:") or not selector:
        raise RuntimeError("target adapter invalid")
    client = Cdp(endpoint)
    try:
        session = client.attach(origin)
        found = client.call("Runtime.evaluate", {"expression": f"document.querySelector({json.dumps(selector)})"}, session).get("result", {})
        object_id = found.get("objectId")
        if not object_id:
            raise RuntimeError("target input unavailable")
        client.call("Runtime.callFunctionOn", {"objectId": object_id, "functionDeclaration": "function(v){const s=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;s.call(this,v);this.dispatchEvent(new Event('input',{bubbles:true}));this.dispatchEvent(new Event('change',{bubbles:true}));return true}", "arguments": [{"value": value}], "returnByValue": True}, session)
        if submit:
            clicked = client.evaluate(session, f"(()=>{{const x=document.querySelector({json.dumps(submit)});if(!x)return false;x.click();return true}})()")
            if not clicked:
                raise RuntimeError("target submit unavailable")
    finally:
        client.close()


def supervisor_request(kind: str, timeout: float = 20.0) -> dict:
    pid_path = RUNTIME_DIR / "supervisor.pid"
    secure_file(pid_path)
    pid = int(pid_path.read_text().strip())
    before = connector_state()
    os.kill(pid, signal.SIGUSR1 if kind == "checkpoint" else signal.SIGUSR2)
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        time.sleep(0.25)
        if kind == "revoke" and not (STATE_DIR / "connector-state.json").exists():
            return {"status": "revoked"}
        current = connector_state()
        if int(current.get("current_generation", 0)) > int(before.get("current_generation", 0)):
            return current
    raise TimeoutError("supervisor request timed out")


class Handler(BaseHTTPRequestHandler):
    server_version = "FocusaSmsBroker/2"

    def log_message(self, *_args: object) -> None:
        pass

    def reply(self, status: int, value: dict) -> None:
        data = json.dumps(value, separators=(",", ":")).encode()
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("cache-control", "no-store")
        self.send_header("content-length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def auth(self) -> bool:
        return hmac.compare_digest(self.headers.get("authorization", ""), f"Bearer {TOKEN}")

    def body(self) -> dict:
        size = int(self.headers.get("content-length", "0"))
        return json.loads(self.rfile.read(size)) if 0 < size <= 65536 else {}

    def route(self) -> urllib.parse.SplitResult:
        return urllib.parse.urlsplit(self.path)

    def blocked(self, failure: str, status: int = 403) -> None:
        self.reply(status, envelope(False, "blocked", "SMS action rejected", failure_class=failure))

    def do_GET(self) -> None:
        if not self.auth(): return self.blocked("unauthorized", 401)
        url = self.route(); query = urllib.parse.parse_qs(url.query)
        try:
            if url.path == "/v1/sms/health":
                health = browser_health(); state = connector_state(); return self.reply(200, envelope(True, "ready", "SMS connector is paired, persisted, and semantically ready", connector={"schema":"focusa.sms_health.v1","connector_id":CONNECTOR_ID,"connector_kind":"google_messages","status":"ready","checkpoint_generation":health["generation"],"checkpoint_status":health["checkpoint_status"],"restored_at":state.get("restored_at"),"last_probe_at":now_iso(),"capabilities":["health","checkpoint","revoke","otp_challenge","inject_otp","list_threads","read_thread","search","send","events"],"thread_count":health["thread_count"]}))
            if url.path == "/v1/sms/enrollment":
                state = connector_state(); ready = state.get("status") == "ready" and int(state.get("ready_proof_count", 0)) >= 2
                return self.reply(200 if ready else 409, envelope(ready, "paired_persisted" if ready else "enrolling", "Enrollment accepted only after fresh restored readiness", connector_id=CONNECTOR_ID))
            grant_id = query.get("grant_id", [""])[0]; consumer = query.get("consumer_ref", [""])[0]
            if url.path == "/v1/sms/threads":
                authorize(grant_id, consumer, "list_threads"); rows = list_threads(min(int(query.get("limit", ["50"])[0]), 200)); audit("list_threads", "ok", consumer_ref=consumer, grant_id=grant_id); return self.reply(200, envelope(True, "ok", "Authorized thread summaries", threads=rows))
            match = re.fullmatch(r"/v1/sms/threads/([A-Za-z0-9_.:-]+)/messages", url.path)
            if match:
                authorize(grant_id, consumer, "read_thread", thread_handle=match.group(1)); rows = read_messages(match.group(1), min(int(query.get("limit", ["50"])[0]), 200)); audit("read_thread", "ok", consumer_ref=consumer, grant_id=grant_id); return self.reply(200, envelope(True, "ok", "Authorized bounded thread read", messages=rows))
            if url.path == "/v1/sms/search":
                authorize(grant_id, consumer, "search"); needle = query.get("query", [""])[0].casefold(); rows = [item for item in list_threads(200) if needle and needle in ((item.get("display_name") or "") + " " + (item.get("snippet") or "")).casefold()][:min(int(query.get("limit", ["50"])[0]), 200)]; audit("search", "ok", consumer_ref=consumer, grant_id=grant_id); return self.reply(200, envelope(True, "ok", "Authorized bounded search", matches=rows))
            if url.path == "/v1/sms/events":
                grant = authorize(grant_id, consumer, "events"); limit = min(int(query.get("limit", ["100"])[0]), 500); rows = audit_events(consumer, bool(grant.get("scope", {}).get("all_events")), limit)
                return self.reply(200, envelope(True, "ok", "Value-free broker events", events=rows))
            return self.blocked("not_found", 404)
        except PermissionError: return self.blocked("grant_rejected")
        except Exception as error:
            audit(url.path, "blocked", failure_class=type(error).__name__); return self.reply(503, envelope(False, "degraded", "Connector operation unavailable", failure_class="connector_operation_unavailable"))

    def do_POST(self) -> None:
        if not self.auth(): return self.blocked("unauthorized", 401)
        url = self.route(); body = self.body(); grant_id = str(body.get("grant_id", "")); consumer = str(body.get("consumer_ref", ""))
        try:
            if url.path == "/v1/sms/otp/challenges":
                provider = str(body.get("provider", "")); target = str(body.get("target_handle", "")); authorize(grant_id, consumer, "otp_challenge", target_handle=target, provider=provider)
                with LOCK:
                    active = [item for item in CHALLENGES.values() if item["grant_id"] == grant_id and item["target_handle"] == target and item["status"] == "waiting" and item["expires"] > time.time()]
                if active: return self.blocked("active_challenge_exists", 409)
                baseline = {item[0] for item in provider_candidates(provider)}; handle = opaque("challenge", secrets.token_hex(16)); ttl = min(max(int(body.get("ttl_seconds", 300)), 30), 600)
                with LOCK: CHALLENGES[handle] = {"provider": provider, "target_handle": target, "consumer_ref": consumer, "grant_id": grant_id, "baseline": baseline, "expires": time.time() + ttl, "status": "waiting"}
                audit("otp_challenge", "ok", consumer_ref=consumer, grant_id=grant_id, target_handle=target); return self.reply(200, envelope(True, "waiting", "OTP challenge registered", challenge_handle=handle, expires_in_seconds=ttl))
            if url.path == "/v1/sms/otp/inject":
                handle = str(body.get("challenge_handle", "")); target = str(body.get("target_handle", ""))
                with LOCK: challenge = CHALLENGES.get(handle)
                if not challenge or challenge["status"] != "waiting" or challenge["expires"] <= time.time() or challenge["consumer_ref"] != consumer or challenge["target_handle"] != target or challenge["grant_id"] != grant_id: return self.blocked("challenge_ineligible")
                authorize(grant_id, consumer, "inject_otp", target_handle=target, provider=challenge["provider"])
                candidates = [(fingerprint, code) for fingerprint, code in provider_candidates(challenge["provider"]) if fingerprint not in challenge["baseline"]]
                if len(candidates) != 1: return self.blocked("otp_candidate_ambiguous", 409)
                with LOCK:
                    if challenge["status"] != "waiting": return self.blocked("challenge_ineligible", 409)
                    challenge["status"] = "injecting"
                try:
                    authorize(grant_id, consumer, "inject_otp", target_handle=target, provider=challenge["provider"], consume=True)
                    inject_target(target, candidates[0][1])
                except Exception:
                    with LOCK: challenge["status"] = "blocked"
                    raise
                with LOCK: challenge["status"] = "consumed"
                audit("otp_inject", "ok", consumer_ref=consumer, grant_id=grant_id, target_handle=target); return self.reply(200, envelope(True, "injected", "OTP injected into exact bound target", injected=True, challenge_handle=handle))
            if url.path == "/v1/sms/send":
                if body.get("confirm") is not True: return self.blocked("approval_required")
                recipients = body.get("recipient_handles") or []; message = str(body.get("body", "")); idem = str(body.get("idempotency_key", ""))
                if len(recipients) != 1 or not message.strip() or not idem: return self.blocked("validation_rejected", 400)
                authorize(grant_id, consumer, "send"); key = (grant_id, consumer, idem)
                with LOCK: prior = SENDS.get(key)
                if prior: return self.reply(200, envelope(True, "sent", "Idempotent send receipt replayed", send_handle=prior, idempotency_key=idem, replayed=True))
                receipt = send_message(str(recipients[0]), message); authorize(grant_id, consumer, "send", consume=True)
                with LOCK: SENDS[key] = receipt
                audit("send", "ok", consumer_ref=consumer, grant_id=grant_id); return self.reply(200, envelope(True, "sent", "Message sent", send_handle=receipt, idempotency_key=idem, replayed=False))
            if url.path == "/v1/sms/checkpoint":
                authorize(grant_id, consumer, "checkpoint"); state = supervisor_request("checkpoint"); audit("checkpoint", "ok", consumer_ref=consumer, grant_id=grant_id); return self.reply(200, envelope(True, "verified_standby", "Encrypted connector generation checkpointed", generation=state.get("current_generation")))
            if url.path == "/v1/sms/revoke":
                if body.get("confirm") != "REVOKE": return self.blocked("explicit_revoke_confirmation_required")
                authorize(grant_id, consumer, "revoke", consume=True); audit("revoke", "accepted", consumer_ref=consumer, grant_id=grant_id); supervisor_request("revoke", 10); return self.reply(200, envelope(True, "revoked", "Connector cryptographically revoked"))
            return self.blocked("not_found", 404)
        except PermissionError: return self.blocked("grant_rejected")
        except Exception as error:
            audit(url.path, "blocked", consumer_ref=consumer, grant_id=grant_id, failure_class=type(error).__name__); return self.reply(503, envelope(False, "degraded", "Connector operation unavailable", failure_class="connector_operation_unavailable"))


def main() -> None:
    server = ThreadingHTTPServer(("127.0.0.1", PORT), Handler)
    server.serve_forever()


if __name__ == "__main__":
    main()
