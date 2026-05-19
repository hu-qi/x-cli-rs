.PHONY: help lock locked-check fmt clippy test check build run-image run-google run-baidu run-nanobanana verify install-check release-build

help:
	@echo "Available targets:"
	@echo "  lock           Generate or update Cargo.lock"
	@echo "  locked-check   Verify dependency resolution uses committed Cargo.lock"
	@echo "  fmt            Run cargo fmt --check"
	@echo "  clippy         Run cargo clippy with -D warnings"
	@echo "  test           Run all workspace tests"
	@echo "  check          Run fmt, clippy, and test"
	@echo "  build          Build release binaries"
	@echo "  run-image      Run x chatgpt-image generate with a sample prompt"
	@echo "  run-google     Run x google search with a sample query"
	@echo "  run-baidu      Run x baidu search with a sample query"
	@echo "  run-nanobanana Run x nanobanana gen with a sample prompt"
	@echo "  verify         Run lock, locked-check, check, and release build"
	@echo "  install-check  Syntax-check install scripts where supported"

lock:
	cargo generate-lockfile

locked-check:
	cargo check --workspace --locked

fmt:
	cargo fmt --check

clippy:
	cargo clippy --workspace --all-targets --locked -- -D warnings

test:
	cargo test --workspace --locked

check: fmt clippy test

build:
	cargo build --release --locked -p xcli -p chatgpt-image-cli -p google-cli -p baidu-cli -p nanobanana-cli -p xiaohongshu-cli

run-image:
	cargo run -p xcli -- --verbose chatgpt-image generate "a cute panda riding a bicycle" -o ./images

run-google:
	cargo run -p xcli -- --verbose google search "rust cli" --limit 5 --hl en

run-baidu:
	cargo run -p xcli -- --verbose baidu search "rust cli" --limit 5

run-nanobanana:
	cargo run -p xcli -- --verbose nanobanana gen "画一朵粉色月季花，微距特写" -o ./out --thumb-width 256 --timeout 300

run-xiaohongshu:
	cargo run -p xcli -- --verbose xiaohongshu search "穿搭" --limit 5

verify: lock locked-check check build

install-check:
	sh -n install.sh
	@if command -v pwsh >/dev/null 2>&1; then \
		pwsh -NoProfile -Command "\$$null = Get-Content ./install.ps1"; \
	else \
		echo "pwsh not found; skipping PowerShell syntax smoke check"; \
	fi

release-build: build
