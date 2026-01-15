# Beefcake Development Makefile
#
# Common development tasks for building docs, running tests, etc.
# For Windows, install Make via chocolatey: choco install make

.PHONY: help docs docs-rust docs-ts docs-open test build dev clean

# Default target - show help
help:
	@echo "Beefcake Development Commands"
	@echo "=============================="
	@echo ""
	@echo "Documentation:"
	@echo "  make docs          - Generate all documentation"
	@echo "  make docs-rust     - Generate Rust docs (cargo doc)"
	@echo "  make docs-ts       - Generate TypeScript docs (typedoc)"
	@echo "  make docs-open     - Generate and open Rust docs in browser"
	@echo ""
	@echo "Development:"
	@echo "  make dev           - Run Tauri dev mode"
	@echo "  make build         - Build release version"
	@echo "  make test          - Run all tests"
	@echo "  make clean         - Clean build artifacts"
	@echo ""
	@echo "Code Quality:"
	@echo "  make clippy        - Run Clippy lints"
	@echo "  make fmt           - Format code"
	@echo "  make check         - Check compilation without building"

# Generate all documentation
docs: docs-rust docs-ts
	@echo "✓ Documentation generated!"
	@echo "  - Rust docs: target/doc/beefcake/index.html"
	@echo "  - TypeScript docs: docs/typescript/index.html"

# Generate Rust documentation
docs-rust:
	@echo "Generating Rust documentation..."
	cargo doc --document-private-items --no-deps

# Generate TypeScript documentation
docs-ts:
	@echo "Generating TypeScript documentation..."
	npm run docs:ts

# Generate Rust docs and open in browser
docs-open:
	@echo "Generating and opening Rust documentation..."
	cargo doc --document-private-items --no-deps --open

# Run Tauri development mode
dev:
	npm run tauri dev

# Build release version
build:
	npm run build
	npm run tauri build

# Run tests
test:
	@echo "Running Rust tests..."
	cargo test
	@echo "✓ All tests passed!"

# Run Clippy linter
clippy:
	@echo "Running Clippy..."
	cargo clippy --all-targets --all-features

# Format code
fmt:
	@echo "Formatting Rust code..."
	cargo fmt
	@echo "✓ Code formatted!"

# Check compilation
check:
	@echo "Checking Rust compilation..."
	cargo check --all-targets --all-features

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf dist/
	rm -rf docs/typescript/
	rm -rf target/doc/
	@echo "✓ Cleaned!"

# Install dependencies
install:
	@echo "Installing dependencies..."
	cargo build
	npm install
	@echo "✓ Dependencies installed!"

# Check documentation coverage (experimental)
doc-coverage:
	@echo "Checking documentation coverage..."
	@cargo doc --document-private-items 2>&1 | grep -i "warning: missing documentation"
	@echo "Run 'cargo doc' to see full documentation warnings"
