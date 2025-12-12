# Gummy Search

Elasticsearch-compatible search engine written in Rust.

## Overview

Gummy Search is a limited port of Elasticsearch API written in Rust, designed to be compatible with Elasticsearch 6.4.0 API. It provides a RESTful API compatible with Elasticsearch endpoints, making it a drop-in replacement for basic Elasticsearch use cases.

## Features

### ✅ Implemented
- **Index Management**: Create, get, delete, check existence, update mappings/settings
- **Document Operations**: Full CRUD (create, read, update, delete)
- **Bulk Operations**: Mass indexing with NDJSON format support
- **Search Functionality**:
  - Match query (text search)
  - Match phrase query (exact phrase matching)
  - Multi-match query (search across multiple fields)
  - Term query (exact match)
  - Bool query (must, should, must_not, filter)
  - Range query (numeric/date ranges)
  - Match all query
  - Pagination (from, size)
  - Sorting
  - Multi-index search
- **Cluster Health**: Health check endpoint
- **HTTP Server**: Built with Axum, async/await support
- **Persistent Storage**: Sled-based persistent storage (data survives restarts)
- **Logging**: Comprehensive logging throughout codebase
- **Testing**: Unit and integration tests

### 🚧 In Progress
- Additional query types (wildcard, prefix, terms)
- Search highlighting
- _source filtering

### 📋 Planned
- Advanced query types (wildcard, prefix, terms)
- Search highlighting
- Aggregations
- Inverted index for better search performance
- Tokenization and text analysis

## Quick Start

### Running the Server

```bash
# Build and run
cargo run

# Or using Makefile
make build-release
./target/release/gummy-search
```

The server will start on `http://localhost:9200` (Elasticsearch default port).

### API Usage Examples

#### Create an Index

```bash
curl -X PUT "http://localhost:9200/my_index" -H 'Content-Type: application/json' -d'
{
  "settings": {
    "number_of_shards": 1,
    "number_of_replicas": 0
  },
  "mappings": {
    "properties": {
      "title": { "type": "text" },
      "body": { "type": "text" }
    }
  }
}'
```

#### Index a Document

```bash
curl -X PUT "http://localhost:9200/my_index/_doc/1" -H 'Content-Type: application/json' -d'
{
  "title": "Example Document",
  "body": "This is the content of the document"
}'
```

#### Get a Document

```bash
curl -X GET "http://localhost:9200/my_index/_doc/1"
```

#### Bulk Operations

```bash
curl -X POST "http://localhost:9200/my_index/_bulk" -H 'Content-Type: application/x-ndjson' -d'
{"index":{"_id":"1"}}
{"title":"Document 1","body":"Content 1"}
{"index":{"_id":"2"}}
{"title":"Document 2","body":"Content 2"}
{"delete":{"_id":"1"}}
'
```

#### Check Cluster Health

```bash
curl -X GET "http://localhost:9200/_cluster/health"
```

#### Check Index Existence

```bash
curl -X HEAD "http://localhost:9200/my_index"
```

## Running the Server

### Local Development

```bash
# Run in debug mode
cargo run

# Run in release mode
cargo run --release

# Or using Makefile
make build-release
./target/release/gummy-search
```

The server will start on `http://localhost:9200` by default.

### Environment Variables

- `RUST_LOG` - Set logging level (e.g., `RUST_LOG=info`, `RUST_LOG=debug`)
- `PORT` - Server port (default: 9200) - *Not yet configurable, hardcoded to 9200*

## Development

### Using Makefile

```bash
# Show all available targets
make help

# Build the project (debug)
make build

# Build the project (release)
make build-release

# Run the server
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

# Run the server
cargo run

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
docker run --rm -p 9200:9200 gummy-search

# Run in background
docker run -d --name gummy-search -p 9200:9200 gummy-search

# View logs
docker logs gummy-search
```

## API Compatibility

Gummy Search implements a subset of Elasticsearch 6.4.0 API endpoints. See [API Requirements](docs/elasticsearch-api-requirements.md) for detailed API endpoint specifications.

### Implemented Endpoints

- `PUT /{index}` - Create index
- `HEAD /{index}` - Check index existence
- `GET /{index}` - Get index information
- `DELETE /{index}` - Delete index
- `PUT /{index}/_doc/{id}` - Index document
- `POST /{index}/_doc` - Create document with auto-generated ID
- `GET /{index}/_doc/{id}` - Get document
- `DELETE /{index}/_doc/{id}` - Delete document
- `POST /_bulk` - Bulk operations
- `POST /{index}/_bulk` - Bulk operations for specific index
- `GET /_cluster/health` - Cluster health

### Status

See [TODO](docs/TODO.md) for detailed progress and upcoming features.

## Architecture

- **HTTP Framework**: [Axum](https://github.com/tokio-rs/axum) - Modern async web framework
- **Async Runtime**: [Tokio](https://tokio.rs/) - Async runtime for Rust
- **Storage**: Sled persistent storage (production-ready), configurable data directory
- **Error Handling**: Custom error types with proper HTTP status codes
- **Logging**: [Tracing](https://github.com/tokio-rs/tracing) for structured logging

## Project Status

**Current Version**: 0.1.0 (MVP)

- ✅ Core infrastructure complete
- ✅ Index and document operations working
- ✅ Bulk operations implemented
- 🚧 Search functionality in progress
- 📋 Advanced features planned

See [TODO.md](docs/TODO.md) for detailed progress tracking.

## Contributing

Contributions are welcome! Please ensure:
- Code follows Rust conventions
- All tests pass (`make test`)
- Code is formatted (`make fmt`)
- Linter passes (`make lint`)

## License

MIT
