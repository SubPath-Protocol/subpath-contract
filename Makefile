all: build

build:
	cargo build --target wasm32-unknown-unknown --release

test:
	cargo test
