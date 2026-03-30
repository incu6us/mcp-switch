.PHONY: build test lint fmt clean install

build:
	cargo build --release

test:
	cargo test --all

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --all

clean:
	cargo clean

install:
	cargo install --path .
