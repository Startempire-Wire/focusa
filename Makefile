.PHONY: help install install-daemon build build-release daemon-restart daemon-status test clean

# Default target
help:
	@echo "Focusa Makefile — targets:"
	@echo "  make build              - Debug build (fast, for dev)"
	@echo "  make build-release      - Release build (optimized)"
	@echo "  make install            - Install daemon to /usr/local (binary + service restart)"
	@echo "  make install-daemon     - Same as install (alias)"
	@echo "  make daemon-restart     - Restart systemd service"
	@echo "  make daemon-status      - Show daemon status"
	@echo "  make test               - Run tests"
	@echo "  make clean              - Remove build artifacts"

# Debug build (faster iteration)
build:
	cargo build -p focusa-api --bin focusa-daemon

# Release build (production)
build-release:
	cargo build --release -p focusa-api --bin focusa-daemon

# Install daemon properly (fixes manual-cp portability issue)
install: install-daemon
install-daemon: build
	./scripts/install-daemon.sh

# Restart systemd daemon
daemon-restart:
	systemctl restart focusa-daemon
	sleep 3
	curl -s http://127.0.0.1:8787/v1/health | jq .

# Status
daemon-status:
	systemctl status focusa-daemon --no-pager
	curl -s http://127.0.0.1:8787/v1/health | jq .

# Tests
test:
	cargo test --workspace

# Clean
clean:
	cargo clean