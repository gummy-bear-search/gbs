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

## API Compatibility

See [API Requirements](../../../internal_docs/elasticsearch-api-requirements.md) for detailed API endpoint specifications.

## License

MIT
