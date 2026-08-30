#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
if [[ "$(id -u)" == 0 ]]; then
  exec /usr/sbin/runuser -u wirebot -- "$ROOT/tests/run_sms_rust_tests.sh" "$@"
fi
cd "$ROOT"
case "${1:-all}" in
  core) exec /root/.cargo/bin/cargo test --locked -p focusa-core sms::tests -- --test-threads=1 ;;
  api) exec /root/.cargo/bin/cargo test --locked -p focusa-api routes::sms::tests -- --test-threads=1 ;;
  all)
    /root/.cargo/bin/cargo test --locked -p focusa-core sms::tests -- --test-threads=1
    exec /root/.cargo/bin/cargo test --locked -p focusa-api routes::sms::tests -- --test-threads=1
    ;;
  *) echo "usage: $0 [core|api|all]" >&2; exit 2 ;;
esac
