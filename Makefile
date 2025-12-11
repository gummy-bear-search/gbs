.PHONY: help build build-release run test lint fmt fmt-check check clean clippy all

# Default target
help:
	@echo "Available targets:"
	@echo "  build          - Build the project (debug)"
	@echo "  build-release  - Build the project (release)"
	@echo "  run            - Run the basic_usage example"
	@echo "  test           - Run all tests"
	@echo "  lint           - Run clippy linter"
	@echo "  fmt            - Format code with rustfmt"
	@echo "  fmt-check      - Check code formatting"
	@echo "  check          - Check code compiles without building"
	@echo "  clean          - Clean build artifacts"
	@echo "  all            - Run fmt-check, lint, and test"

# Build targets
build:
	cargo build

build-release:
	cargo build --release

# Run targets
run:
	cargo run --example basic_usage

# Test targets
test:
	cargo test

test-release:
	cargo test --release

# Linting targets
lint: clippy

clippy:
	cargo clippy -- -D warnings

clippy-fix:
	cargo clippy --fix --allow-dirty --allow-staged

# Formatting targets
fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

# Check target (faster than build)
check:
	cargo check

# Clean target
clean:
	cargo clean

# Run all checks
all: fmt-check lint test
