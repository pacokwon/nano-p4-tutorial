.PHONY: fmt build serve

fmt:
	dprint fmt

build:
	cargo build --manifest-path mdbook-p4-highlight/Cargo.toml
	cargo build --manifest-path mdbook-spectec-highlight/Cargo.toml

serve: build
	mdbook serve --open
