#!/usr/bin/env python3
"""Managed pair-once supervisor for a private Focusa connector runtime."""
from __future__ import annotations

import contextlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("focusa_sms_appliance", HERE / "focusa-sms-appliance.py")
if SPEC is None or SPEC.loader is None:
    raise SystemExit("checkpoint authority unavailable")
appliance = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(appliance)


def env_path(name: str) -> Path:
    value = os.environ.get(name, "").strip()
    if not value:
        raise ValueError(f"{name} is required")
    return Path(value)


def command_template(name: str) -> list[str]:
    value = json.loads(os.environ.get(name, "[]"))
    if not isinstance(value, list) or not value or not all(isinstance(item, str) and item for item in value):
        raise ValueError(f"{name} must be a non-empty JSON string array")
    return value


def render(command: list[str], *, profile: Path, cdp_port: int) -> list[str]:
    return [part.format(profile=str(profile), cdp_port=cdp_port) for part in command]


def process(command: list[str]) -> subprocess.Popen[bytes]:
    return subprocess.Popen(
        command,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        close_fds=True,
        start_new_session=True,
    )


def terminate(child: subprocess.Popen[bytes] | None) -> None:
    if child is None or child.poll() is not None:
        return
    child.terminate()
    with contextlib.suppress(subprocess.TimeoutExpired):
        child.wait(timeout=10)
    if child.poll() is None:
        child.kill()
        child.wait(timeout=5)


def notify(value: str) -> None:
    address = os.environ.get("NOTIFY_SOCKET")
    if not address:
        return
    if address.startswith("@"):
        address = "\0" + address[1:]
    with socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM) as client:
        client.connect(address)
        client.sendall(value.encode())


def mount_type(path: Path) -> str | None:
    resolved = path.resolve()
    best: tuple[int, str] | None = None
    for line in Path("/proc/self/mountinfo").read_text(encoding="utf-8").splitlines():
        left, right = line.split(" - ", 1)
        mount = Path(left.split()[4].replace("\\040", " "))
        try:
            resolved.relative_to(mount)
        except ValueError:
            continue
        value = (len(str(mount)), right.split()[0])
        if best is None or value[0] > best[0]:
            best = value
    return best[1] if best else None


class Supervisor:
    def __init__(self) -> None:
        self.state_dir = env_path("FOCUSA_SMS_STATE_DIR")
        self.runtime_dir = env_path("FOCUSA_SMS_RUNTIME_DIR")
        self.key = env_path("FOCUSA_SMS_CHECKPOINT_KEY_FILE")
        self.browser_template = command_template("FOCUSA_SMS_BROWSER_COMMAND_JSON")
        self.probe_template = command_template("FOCUSA_SMS_READY_PROBE_COMMAND_JSON")
        self.broker_template = command_template("FOCUSA_SMS_BROKER_COMMAND_JSON")
        self.active_port = int(os.environ.get("FOCUSA_SMS_ACTIVE_CDP_PORT", "9334"))
        self.standby_port = int(os.environ.get("FOCUSA_SMS_STANDBY_CDP_PORT", "9335"))
        self.interval = max(int(os.environ.get("FOCUSA_SMS_CHECKPOINT_INTERVAL_SECONDS", "300")), 60)
        self.probe_window = max(float(os.environ.get("FOCUSA_SMS_PROBE_WINDOW_SECONDS", "45")), 1.0)
        self.browser: subprocess.Popen[bytes] | None = None
        self.broker: subprocess.Popen[bytes] | None = None
        self.profile: Path | None = None
        self.enrolling = False
        self.running = True
        self.checkpoint_requested = False
        self.revoke_requested = False
        self.last_checkpoint = 0.0

    def prepare(self) -> None:
        appliance._secure_dir(self.state_dir, create=True)
        appliance._secure_dir(self.runtime_dir, create=True)
        appliance._read_key(self.key)
        if os.environ.get("FOCUSA_SMS_REQUIRE_TMPFS", "1") != "0":
            if mount_type(self.runtime_dir) not in {"tmpfs", "ramfs"}:
                raise ValueError("connector runtime must be on tmpfs")

    def profile_path(self, name: str) -> Path:
        return self.runtime_dir / name

    def launch_browser(self, profile: Path, port: int) -> subprocess.Popen[bytes]:
        appliance._secure_dir(profile, create=True)
        return process(render(self.browser_template, profile=profile, cdp_port=port))

    def probe(self, port: int, attempts: int = 2, profile: Path | None = None) -> int:
        return appliance.run_probe(render(self.probe_template, profile=profile or self.profile or self.runtime_dir, cdp_port=port), attempts, self.probe_window)

    def launch_broker(self, port: int) -> None:
        terminate(self.broker)
        self.broker = process(render(self.broker_template, profile=self.profile or self.runtime_dir, cdp_port=port))

    def start(self) -> None:
        self.prepare()
        state = appliance.load_state(self.state_dir)
        if int(state.get("verified_generation", 0)) > 0:
            profile = self.profile_path("active")
            if profile.exists():
                shutil.rmtree(profile)
            receipt = appliance.restore_latest_profile(self.state_dir, self.key, profile)
            self.profile = profile
            self.browser = self.launch_browser(profile, self.active_port)
            proofs = self.probe(self.active_port, 2)
            if proofs < 2:
                terminate(self.browser)
                raise RuntimeError("boot restore failed semantic readiness")
            appliance.mark_ready(self.state_dir, int(receipt["generation"]), proofs)
            self.launch_broker(self.active_port)
            self.enrolling = False
        else:
            profile = self.profile_path("enrollment")
            self.profile = profile
            self.browser = self.launch_browser(profile, self.active_port)
            self.launch_broker(self.active_port)
            state.update({
                "schema": appliance.STATE_SCHEMA,
                "status": "enrolling",
                "checkpoint_status": "absent",
                "source_preserved": True,
                "checked_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            })
            appliance.write_state(self.state_dir, state)
            self.enrolling = True
        self.last_checkpoint = time.monotonic()
        appliance._atomic_write(self.runtime_dir / "supervisor.pid", f"{os.getpid()}\n".encode())
        notify("READY=1\nSTATUS=connector supervisor running")

    def commit_enrollment(self) -> bool:
        if self.browser is None or self.profile is None or self.browser.poll() is not None:
            return False
        if self.probe(self.active_port, 1) != 1:
            return False
        source = self.browser
        source_profile = self.profile
        standby_profile = self.profile_path("standby")
        if standby_profile.exists():
            shutil.rmtree(standby_profile)
        os.kill(source.pid, signal.SIGSTOP)
        standby: subprocess.Popen[bytes] | None = None
        try:
            receipt = appliance.checkpoint_profile(source_profile, self.state_dir, self.key)
            appliance.restore_latest_profile(self.state_dir, self.key, standby_profile)
            standby = self.launch_browser(standby_profile, self.standby_port)
            proofs = self.probe(self.standby_port, 2, standby_profile)
            if proofs < 2:
                raise RuntimeError("standby failed semantic readiness")
            appliance.mark_ready(self.state_dir, int(receipt["generation"]), proofs)
            os.kill(source.pid, signal.SIGCONT)
            terminate(source)
            if source_profile.exists():
                shutil.rmtree(source_profile)
            self.browser = standby
            self.profile = standby_profile
            self.active_port, self.standby_port = self.standby_port, self.active_port
            self.launch_broker(self.active_port)
            self.enrolling = False
            self.last_checkpoint = time.monotonic()
            notify("STATUS=paired_persisted successor ready")
            return True
        except Exception:
            terminate(standby)
            if standby_profile.exists():
                shutil.rmtree(standby_profile)
            os.kill(source.pid, signal.SIGCONT)
            state = appliance.load_state(self.state_dir)
            state.update({
                "status": "enrolling",
                "checkpoint_status": "handoff_rolled_back",
                "failure_class": "restored_connector_unavailable",
                "source_preserved": True,
                "checked_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            })
            appliance.write_state(self.state_dir, state)
            return False

    def checkpoint(self) -> None:
        if self.enrolling or self.browser is None or self.profile is None or self.browser.poll() is not None:
            return
        os.kill(self.browser.pid, signal.SIGSTOP)
        try:
            appliance.checkpoint_profile(self.profile, self.state_dir, self.key)
        finally:
            os.kill(self.browser.pid, signal.SIGCONT)
        self.last_checkpoint = time.monotonic()

    def recover(self) -> None:
        terminate(self.broker)
        if self.profile and self.profile.exists():
            shutil.rmtree(self.profile)
        profile = self.profile_path("recovered")
        receipt = appliance.restore_latest_profile(self.state_dir, self.key, profile)
        self.profile = profile
        self.browser = self.launch_browser(profile, self.active_port)
        proofs = self.probe(self.active_port, 2)
        if proofs < 2:
            terminate(self.browser)
            raise RuntimeError("automatic recovery failed semantic readiness")
        appliance.mark_ready(self.state_dir, int(receipt["generation"]), proofs)
        self.launch_broker(self.active_port)

    def loop(self) -> None:
        while self.running:
            notify("WATCHDOG=1")
            if self.browser is None or self.browser.poll() is not None:
                if self.enrolling:
                    raise RuntimeError("enrollment browser exited before durable pairing")
                self.recover()
            if self.broker is not None and self.broker.poll() is not None:
                self.launch_broker(self.active_port)
            if self.revoke_requested:
                terminate(self.broker)
                terminate(self.browser)
                appliance.revoke(self.state_dir, self.key, "REVOKE")
                self.running = False
                continue
            if self.enrolling:
                self.commit_enrollment()
            elif self.checkpoint_requested or time.monotonic() - self.last_checkpoint >= self.interval:
                self.checkpoint_requested = False
                self.checkpoint()
            time.sleep(2)

    def stop(self) -> None:
        self.running = False
        with contextlib.suppress(Exception):
            self.checkpoint()
        terminate(self.broker)
        terminate(self.browser)
        with contextlib.suppress(FileNotFoundError):
            (self.runtime_dir / "supervisor.pid").unlink()
        notify("STOPPING=1\nSTATUS=connector stopped with encrypted checkpoint")


def main() -> None:
    supervisor = Supervisor()

    def stop(_signum: int, _frame: object) -> None:
        supervisor.running = False

    def checkpoint(_signum: int, _frame: object) -> None:
        supervisor.checkpoint_requested = True

    def revoke(_signum: int, _frame: object) -> None:
        supervisor.revoke_requested = True

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    signal.signal(signal.SIGUSR1, checkpoint)
    signal.signal(signal.SIGUSR2, revoke)
    try:
        supervisor.start()
        supervisor.loop()
    finally:
        supervisor.stop()


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"focusa sms supervisor failed: {type(error).__name__}", file=sys.stderr)
        raise SystemExit(1)
