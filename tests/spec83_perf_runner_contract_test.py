#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
TESTS = ROOT / "tests"
RUNNER = TESTS / "spec83_perf_runner.py"
FIXTURE = TESTS / "fixtures/spec83_pi_fixture.py"
SCRIPT = TESTS / "spec83_perf_benchmark_runtime_test.sh"
sys.path.insert(0, str(TESTS))
import spec83_perf_runner as runner  # noqa: E402
import spec83_perf_benchmark as benchmark  # noqa: E402


class Spec83RunnerContract(unittest.TestCase):
    def run_mode(self, mode: str, *, extension: bool = False) -> dict:
        with tempfile.TemporaryDirectory() as directory:
            timing = Path(directory) / "timing.json"
            env = os.environ.copy()
            env["SPEC83_FIXTURE_MODE"] = mode
            command = [
                sys.executable,
                str(RUNNER),
                "arm",
                "--label",
                mode,
                "--program",
                sys.executable,
                "--program-arg",
                str(FIXTURE),
                "--timing",
                str(timing),
                "--timeout-seconds",
                "0.2",
            ]
            if extension:
                command.extend(["--extension-path", str(FIXTURE)])
            completed = subprocess.run(command, check=True, capture_output=True, text=True, env=env)
            self.assertTrue(completed.stdout.strip())
            return json.loads(timing.read_text(encoding="utf-8"))

    def test_success_requires_zero_exit_exact_output_and_scoped_metrics(self) -> None:
        result = self.run_mode("ok")
        self.assertEqual(result["exit_code"], 0)
        self.assertTrue(result["output_ok"])
        self.assertGreaterEqual(result["elapsed_s"], 0)
        self.assertIn("platform", result["environment"])
        if runner.resource is not None and sys.platform.startswith(("linux", "darwin")):
            self.assertTrue(result["ok"])
            self.assertIsNotNone(result["cpu_pct"])
            self.assertIsNotNone(result["maxrss_kb"])
        else:
            self.assertFalse(result["ok"])
            self.assertIn("maxrss_unavailable", result["errors"])
        self.assertEqual(result["stdout_output"], "OK\n")

    def test_nonzero_exit_cannot_become_success(self) -> None:
        result = self.run_mode("fail")
        self.assertFalse(result["ok"])
        self.assertEqual(result["exit_code"], 7)
        self.assertIn("exit_code:7", result["errors"])

    def test_wrong_output_cannot_become_success(self) -> None:
        result = self.run_mode("wrong-output")
        self.assertFalse(result["ok"])
        self.assertIn("output_mismatch", result["errors"])

    def test_timeout_cannot_become_success(self) -> None:
        result = self.run_mode("timeout")
        self.assertFalse(result["ok"])
        self.assertTrue(result["timed_out"])
        self.assertIn("timeout", result["errors"])

    def test_extension_load_failure_cannot_become_success(self) -> None:
        result = self.run_mode("extension-fail", extension=True)
        self.assertFalse(result["ok"])
        self.assertIn("extension_load_failure", result["errors"])

    def test_rpc_probe_requires_focusa_extension_command_registration(self) -> None:
        healthy = runner.probe_extension(sys.executable, str(FIXTURE), 0.2, [str(FIXTURE)])
        self.assertEqual(healthy["status"], "healthy")
        self.assertTrue(healthy["registered"])
        self.assertEqual(healthy["provider_calls"], 0)
        failed = self.run_probe_extension("extension-fail")
        self.assertEqual(failed["status"], "failed")
        self.assertIn("focusa_status_command_not_registered", failed["errors"])

    def run_probe_extension(self, mode: str) -> dict:
        env = os.environ.copy()
        env["SPEC83_FIXTURE_MODE"] = mode
        with patch.dict(os.environ, env, clear=True):
            return runner.probe_extension(sys.executable, str(FIXTURE), 0.2, [str(FIXTURE)])

    def test_threshold_and_regression_violations_fail_aggregate(self) -> None:
        def arm(elapsed: float, cpu: float = 20, rss: int = 100) -> dict:
            return {"ok": True, "elapsed_s": elapsed, "cpu_pct": cpu, "maxrss_kb": rss, "errors": []}

        result = runner.aggregate_results(
            {
                "none": arm(1),
                "focusa_event_driven": arm(2, cpu=250),
                "focusa_polling": arm(3, rss=2000),
            },
            {"status": "passed", "p95_ms": 10},
            {"status": "healthy", "registered": True},
            {
                "max_arm_elapsed_s": 10,
                "max_arm_rss_kb": 1000,
                "max_arm_cpu_pct": 200,
                "max_status_p95_ms": 100,
                "max_event_delta_s": 0.5,
                "max_polling_delta_s": 0.5,
            },
            {"model_id": "fixture/model", "pi_runtime_version": "fixture 1"},
        )
        self.assertEqual(result["status"], "failed")
        self.assertFalse(result["performance_evidence"])
        self.assertTrue(any("cpu_threshold" in failure for failure in result["failures"]))
        self.assertTrue(any("rss_threshold" in failure for failure in result["failures"]))
        self.assertIn("event_driven:regression_threshold", result["failures"])
        self.assertIn("polling:regression_threshold", result["failures"])

    def test_malformed_measurements_fail_closed_without_parser_crash(self) -> None:
        result = runner.aggregate_results(
            {
                "none": {"ok": True, "elapsed_s": "fast", "cpu_pct": float("nan"), "maxrss_kb": None},
                "focusa_event_driven": {"ok": True, "elapsed_s": None, "cpu_pct": 0, "maxrss_kb": 0},
                "focusa_polling": {"ok": True, "elapsed_s": 1, "cpu_pct": 0, "maxrss_kb": 1},
            },
            {"status": "passed", "p95_ms": "not-a-number"},
            {"status": "healthy", "registered": True},
            {
                "max_arm_elapsed_s": 10,
                "max_arm_rss_kb": 1000,
                "max_arm_cpu_pct": 200,
                "max_status_p95_ms": 100,
                "max_event_delta_s": 5,
                "max_polling_delta_s": 5,
            },
            {},
        )
        self.assertEqual(result["status"], "failed")
        self.assertFalse(result["performance_evidence"])
        self.assertIn("status_latency:p95_threshold_or_unavailable", result["failures"])
        self.assertIn("event_driven:comparison_unavailable", result["failures"])

    def test_status_probe_rejects_missing_runtime_counters(self) -> None:
        class Response:
            status = 200

            def __enter__(self) -> "Response":
                return self

            def __exit__(self, *_args: object) -> None:
                return None

            def read(self, _limit: int) -> bytes:
                return b'{"runtime_perf":{"supervisor_ticks_total":1}}'

        with patch("urllib.request.urlopen", return_value=Response()):
            result = runner.probe_status("http://example.test/v1/status", samples=1)
        self.assertEqual(result["status"], "failed")
        self.assertIn("runtime_perf_schema_incomplete", result["errors"])

    def test_status_probe_requires_runtime_counters(self) -> None:
        names = runner.REQUIRED_STATUS_COUNTERS
        payload = {"runtime_perf": {name: 0 for name in names}}

        class Handler(BaseHTTPRequestHandler):
            def do_GET(self) -> None:  # noqa: N802
                body = json.dumps(payload).encode()
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)

            def log_message(self, *_args: object) -> None:
                return

        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            result = runner.probe_status(f"http://127.0.0.1:{server.server_port}/v1/status", samples=2)
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=2)
        self.assertEqual(result["status"], "passed")
        self.assertEqual(result["samples"], 2)
        self.assertEqual(set(result["counter_snapshot"]), set(names))

    def test_linux_and_macos_rss_units_normalize_to_kib(self) -> None:
        self.assertEqual(runner.normalize_maxrss_kb(1024, "linux"), 1024)
        self.assertEqual(runner.normalize_maxrss_kb(1048576, "darwin"), 1024)

    def test_unrelated_process_does_not_replace_benchmark_child(self) -> None:
        unrelated = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(1)"])
        try:
            result = self.run_mode("ok")
            self.assertTrue(result["ok"])
            self.assertNotEqual(result["pid"], unrelated.pid)
            source = RUNNER.read_text(encoding="utf-8")
            self.assertNotIn("ps -eo", source)
            self.assertNotIn("shell=True", source)
        finally:
            unrelated.terminate()
            unrelated.wait(timeout=2)

    def test_missing_resource_metrics_fail_closed(self) -> None:
        original = runner.resource
        runner.resource = None
        try:
            result = runner.run_arm(
                label="no-resource",
                program=sys.executable,
                program_args=[str(FIXTURE)],
                arguments=[],
                timeout_seconds=1,
                env=os.environ.copy(),
            )
        finally:
            runner.resource = original
        self.assertFalse(result["ok"])
        self.assertIn("cpu_pct_unavailable", result["errors"])
        self.assertIn("maxrss_unavailable", result["errors"])

    def test_status_url_must_be_exact_focusa_status_route(self) -> None:
        self.assertEqual(benchmark.safe_endpoint_label("http://127.0.0.1:8787/v1/status"), "http://127.0.0.1:8787/v1/status")
        self.assertIsNone(benchmark.safe_endpoint_label("http://127.0.0.1:8787/admin"))
        rejected = runner.probe_status("http://127.0.0.1:8787/admin", samples=1)
        self.assertEqual(rejected["status"], "unavailable")

    def test_live_evidence_path_must_be_new_json_in_project_docs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            evidence = project / "docs/evidence"
            evidence.mkdir(parents=True)
            valid = evidence / "spec83-perf-run.json"
            self.assertTrue(benchmark.evidence_output_path(str(valid), project))
            self.assertFalse(benchmark.evidence_output_path(str(project / "tmp/result.json"), project))
            valid.write_text("{}", encoding="utf-8")
            self.assertFalse(benchmark.evidence_output_path(str(valid), project))

    def test_benchmark_is_unavailable_without_explicit_live_authority(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "result.json"
            env = os.environ.copy()
            env.pop("SPEC83_LIVE_PROVIDER", None)
            env["FOCUSA_PI_BIN"] = sys.executable
            env["SPEC83_OUT_JSON"] = str(output)
            completed = subprocess.run(["bash", str(SCRIPT)], check=False, capture_output=True, text=True, env=env)
            self.assertEqual(completed.returncode, 2)
            result = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(result["status"], "unavailable")
            self.assertFalse(result["performance_evidence"])
            self.assertEqual(result["provider_calls"], 0)


if __name__ == "__main__":
    unittest.main()
