# Work Loop Status Liveness Stress — 2026-07-19

Branch daemon used the real `focusa-workloop` BD graph under an isolated data directory and port.

After cloning the shared Focusa projection before secondary lock/provider/process awaits:

- 12 concurrent `/v1/health` requests: 12/12 HTTP 200, max 0.339 s
- 12 concurrent scoped `/v1/work-loop/status` requests: 12/12 HTTP 200, max 4.831 s
- 12 concurrent scoped `/v1/work-loop/heartbeat` mutations: 12/12 HTTP 200, max 5.564 s
- Combined requests: 36/36 passed
- Static lock-order regression: `tests/work_loop_status_lock_liveness_test.py`
- `cargo check -p focusa-api`: passed

The status path performs a complete real provider snapshot while heartbeats force daemon projection writes. Health remained responsive and no request exceeded its 15-second bound.
