.PHONY: help build build-release run test lint fmt fmt-check check clean clippy all docker-build docker-test docker-run docker-shell

# Default target
help:
	@echo "Available targets:"
	@echo "  build          - Build the project (debug)"
	@echo "  build-release  - Build the project (release)"
	@echo "  run            - Run the Gummy Bear Search server"
	@echo "  test           - Run all tests"
	@echo "  lint           - Run clippy linter"
	@echo "  fmt            - Format code with rustfmt"
	@echo "  fmt-check      - Check code formatting"
	@echo "  check          - Check code compiles without building"
	@echo "  clean          - Clean build artifacts"
	@echo "  all            - Run fmt-check, lint, and test"
	@echo ""
	@echo "Docker targets:"
	@echo "  docker-build   - Build Docker image"
	@echo "  docker-test    - Run tests in Docker"
	@echo "  docker-run     - Run the application in Docker"
	@echo "  docker-shell   - Open interactive shell in Docker builder"

# Build targets
build:
	cargo build

build-release:
	cargo build --release

# Run targets
run:
	cargo run

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

# Docker targets
docker-build:
	docker build -t gbs .

docker-build-builder:
	docker build --target builder -t gbs:builder .

docker-test: docker-build-builder
	docker run --rm -v $$(pwd):/app -w /app gbs:builder \
		cargo test

docker-run:
	docker run --rm gbs

docker-shell: docker-build-builder
	docker run --rm -it -v $$(pwd):/app -w /app gbs:builder \
		/bin/sh
