.PHONY: test clean gen

test:
	cargo test

clean:
	cargo clean

gen:
	cargo run --bin tch-bindgen --manifest-path tch-bindgen/Cargo.toml --release
	rustfmt src/wrappers/tensor_fallible_generated.rs
	rustfmt src/wrappers/tensor_generated.rs
	rustfmt torch-sys/src/c_generated.rs
