# works on macOS
default:
	cargo build --release --locked

run:
	cargo run

lint:
	cargo clippy -- -D warnings

format-check:
	cargo fmt --all -- --check

format:
	cargo fmt --all