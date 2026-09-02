#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import pathlib
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify-callgraph-validator.py"
spec = importlib.util.spec_from_file_location("callgraph_probe", SCRIPT)
probe_module = importlib.util.module_from_spec(spec)
assert spec and spec.loader
spec.loader.exec_module(probe_module)


class Handler(BaseHTTPRequestHandler):
    status = 200
    response = {"status": "valid", "valid": True, "canonical": True, "issues": [], "graph_id": "install-probe"}
    observed = None

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        type(self).observed = json.loads(self.rfile.read(length))
        body = json.dumps(type(self).response).encode()
        self.send_response(type(self).status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_args):
        pass


class CallGraphValidatorProbeTest(unittest.TestCase):
    def setUp(self):
        Handler.status = 200
        Handler.response = {"status": "valid", "valid": True, "canonical": True, "issues": [], "graph_id": "install-probe"}
        Handler.observed = None
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.url = f"http://127.0.0.1:{self.server.server_port}/v1/callgraphs/validate"

    def tearDown(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=2)

    def test_accepts_only_canonical_valid_envelope(self):
        payload = probe_module.probe(self.url)
        self.assertTrue(payload["valid"])
        self.assertEqual(Handler.observed, probe_module.GOLDEN_GRAPH)

    def test_rejects_missing_route(self):
        Handler.status = 404
        with self.assertRaisesRegex(probe_module.ProbeError, "HTTP 404"):
            probe_module.probe(self.url)

    def test_rejects_structurally_invalid_or_noncanonical_envelope(self):
        Handler.response = {"status": "invalid", "valid": False, "canonical": True, "issues": [{"path": "frames"}]}
        with self.assertRaisesRegex(probe_module.ProbeError, "non-canonical or invalid"):
            probe_module.probe(self.url)

    def test_rejects_protocol_drift(self):
        Handler.response = {"status": "valid", "valid": True, "issues": []}
        with self.assertRaisesRegex(probe_module.ProbeError, "non-canonical or invalid"):
            probe_module.probe(self.url)

    def test_installer_fails_closed_and_invokes_probe(self):
        installer = (ROOT / "scripts" / "install-daemon.sh").read_text()
        self.assertIn('rollback "health verification failed', installer)
        self.assertNotIn("proceeding despite health check failure", installer)
        self.assertIn("verify-callgraph-validator.py", installer)
        self.assertIn('rollback "CallGraph validator verification failed', installer)


if __name__ == "__main__":
    unittest.main()
