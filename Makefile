.PHONY: help fmt clippy test check build run-image run-google verify install-check release-build

help:
	@echo "Available targets:"
	@echo "  fmt           Run cargo fmt --check"
	@echo "  clippy        Run cargo clippy with -D warnings"
	@echo "  test          Run all workspace tests"
	@echo "  check         Run fmt, clippy, and test"
	@echo "  build         Build release binaries"
	@echo "  run-image     Run x chatgpt-image generate with a sample prompt"
	@echo "  run-google    Run x google search with a sample query"
	@echo "  verify        Run check and release build"
	@echo "  install-check Syntax-check install scripts where supported"

fmt:
	cargo fmt --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

check: fmt clippy test

build:
	cargo build --release -p xcli -p chatgpt-image-cli -p google-cli

run-image:
	cargo run -p xcli -- --verbose chatgpt-image generate "a cute panda riding a bicycle" -o ./images

run-google:
	cargo run -p xcli -- --verbose google search "rust cli" --limit 5 --hl en

verify: check build

install-check:
	sh -n install.sh
	@if command -v pwsh >/dev/null 2>&1; then \
		pwsh -NoProfile -Command "\$$null = Get-Content ./install.ps1"; \
	else \
		echo "pwsh not found; skipping PowerShell syntax smoke check"; \
	fi

release-build: build
