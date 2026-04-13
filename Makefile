.PHONY: ci-local watch-app

ci-local:
	cargo +stable fmt --all
	cargo +stable check --workspace --all-targets
	cargo +stable test --workspace
	cargo +stable clippy --workspace --all-targets -- -D warnings
	cargo +stable fmt --all -- --check

watch-app:
	cargo watch -x "run -p veripatch-app"
