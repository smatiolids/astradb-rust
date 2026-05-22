.PHONY: help build run release test check clean fmt lint install-deps

help:
	@echo "Available commands:"
	@echo "  make build         - Build the project (debug)"
	@echo "  make run           - Run the application"
	@echo "  make release       - Build optimized release binary"
	@echo "  make run-release   - Run optimized release binary"
	@echo "  make test          - Run tests"
	@echo "  make check         - Check code without building"
	@echo "  make fmt           - Format code"
	@echo "  make lint          - Run clippy linter"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make install-deps  - Install system dependencies"

build:
	cargo build

run: build
	cargo run

release:
	cargo build --release

run-release: release
	./target/release/astra-db-rust

test:
	cargo test

check:
	cargo check

fmt:
	cargo fmt --all

lint:
	cargo clippy -- -D warnings

clean:
	cargo clean

install-deps:
	@echo "Installing system dependencies..."
	@uname -s | grep -q Darwin && brew install openssl pkg-config || true
	@uname -s | grep -q Linux && (command -v apt-get > /dev/null && sudo apt-get install -y libssl-dev pkg-config || true) || true
	@uname -s | grep -q Linux && (command -v dnf > /dev/null && sudo dnf install -y openssl-devel pkg-config || true) || true
	@echo "Done!"
