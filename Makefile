.PHONY: ci-local

ci-local:
	cargo +stable fmt --all
	cargo +stable check --workspace --all-targets
	cargo +stable test --workspace
	cargo +stable clippy --workspace --all-targets -- -D warnings
	cargo +stable fmt --all -- --check
