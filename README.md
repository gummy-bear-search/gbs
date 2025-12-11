# Gummy Search

Elasticsearch-compatible search engine written in Rust.

## Overview

Gummy Search is a limited port of Elasticsearch API written in Rust, designed to be compatible with Elasticsearch 6.4.0 API.

## Features

- Index management (create, update, delete)
- Document operations (index, get, delete)
- Bulk operations for mass indexing
- Search queries with various query types
- Compatible with Laravel Scout

## Usage

```rust
use gummy_search::GummySearchClient;

let client = GummySearchClient::new("http://localhost:9200");

// Create an index
client.create_index("content_index", settings).await?;

// Index a document
client.index_document("content_index", "123", &document).await?;

// Search
let results = client.search("content_index", query).await?;
```

## Development

### Using Makefile

```bash
# Show all available targets
make help

# Build the project (debug)
make build

# Build the project (release)
make build-release

# Run the basic_usage example
make run

# Run all tests
make test

# Run linters (clippy)
make lint

# Format code
make fmt

# Check code formatting
make fmt-check

# Run all checks (fmt-check, lint, test)
make all

# Clean build artifacts
make clean
```

### Using Cargo directly

```bash
# Build
cargo build

# Run tests
cargo test

# Run examples
cargo run --example basic_usage

# Run clippy linter
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

## Docker

The project includes a multi-stage Dockerfile based on the official Rust 1.91.1 Alpine image.

### Using Makefile (Recommended)

```bash
# Build Docker image
make docker-build

# Run tests in Docker
make docker-test

# Run the application in Docker
make docker-run

# Open interactive shell in Docker (for development)
make docker-shell
```

### Building the Docker image

```bash
# Build the final image
docker build -t gummy-search .

# Build the builder stage (for development)
docker build --target builder -t gummy-search:builder .
```

### Running Tests in Docker

There are several ways to run tests:

**Option 1: Using Makefile (easiest)**
```bash
make docker-test
```

**Option 2: Using Docker directly**
```bash
# Build the builder stage first
docker build --target builder -t gummy-search:builder .

# Run tests with volume mount (recommended for development)
docker run --rm -v $(pwd):/app -w /app gummy-search:builder \
    cargo test

# Run tests without volume mount (uses code from image)
docker run --rm gummy-search:builder \
    cargo test
```

**Option 3: Interactive shell for debugging**
```bash
# Open interactive shell
make docker-shell

# Then inside the container:
cargo test
cargo test -- --nocapture  # Show output
```

### Running the Application

```bash
# Run the container
docker run --rm gummy-search

# Run with port mapping (if server is implemented)
docker run --rm -p 9200:9200 gummy-search
```

## API Compatibility

See [API Requirements](../../../internal_docs/elasticsearch-api-requirements.md) for detailed API endpoint specifications.

## License

MIT
