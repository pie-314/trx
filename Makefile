.PHONY: setup run dev build test fix verify fmt lint clean

setup:
	chmod +x scripts/*.sh
	chmod +x .githooks/*
	git config core.hooksPath .githooks
	@echo "Setup complete"

run:
	cargo run -p trx-cli

dev:
	cargo run -p trx-cli

build:
	cargo build --workspace --all-targets --all-features

test:
	cargo test --workspace --all-features

fix:
	./scripts/fix.sh

verify:
	./scripts/verify.sh

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings -A dead_code -A clippy::type_complexity

clean:
	cargo clean
