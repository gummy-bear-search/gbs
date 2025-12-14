# Gummy Search

<div align="center">

![Gummy Search Logo](images/gummy-search-400x500.jpg)

</div>

Elasticsearch-compatible search engine written in Rust.

## Overview

Gummy Search is a limited port of Elasticsearch API written in Rust, designed to be compatible with Elasticsearch 6.8.23 API. It provides a RESTful API compatible with Elasticsearch endpoints, making it a drop-in replacement for basic Elasticsearch use cases.

## Features

### ✅ Implemented
- **Index Management**: Create, get, delete, check existence, update mappings/settings
- **Document Operations**: Full CRUD (create, read, update, delete)
- **Bulk Operations**: Mass indexing with NDJSON format support (with refresh parameter)
- **Search Functionality**:
  - Match query (text search)
  - Match phrase query (exact phrase matching)
  - Multi-match query (search across multiple fields)
  - Term query (exact match)
  - Terms query (match any of multiple values)
  - Wildcard query (pattern matching with * and ?)
  - Prefix query (prefix matching)
  - Bool query (must, should, must_not, filter)
  - Range query (numeric/date ranges)
  - Match all query
  - Pagination (from, size)
  - Sorting
  - Multi-index search (with wildcard patterns)
  - _source filtering (include/exclude fields)
  - Search highlighting (highlight matched terms)
- **Cluster Health**: Health check endpoint
- **Monitoring**: Cluster stats and index listing endpoints
- **HTTP Server**: Built with Axum, async/await support
- **Persistent Storage**: Sled-based persistent storage (data survives restarts)
- **Logging**: Comprehensive logging throughout codebase
- **Testing**: Unit and integration tests

### 🚧 In Progress
- Performance optimizations

### 📋 Planned
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

#### Search Documents

**Match Query:**
```bash
curl -X POST "http://localhost:9200/my_index/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "match": {
      "title": "example"
    }
  }
}'
```

**Wildcard Query:**
```bash
curl -X POST "http://localhost:9200/my_index/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "wildcard": {
      "title": "*exam*"
    }
  }
}'
```

**Prefix Query:**
```bash
curl -X POST "http://localhost:9200/my_index/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "prefix": {
      "title": "exam"
    }
  }
}'
```

**Terms Query:**
```bash
curl -X POST "http://localhost:9200/my_index/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "terms": {
      "status": ["published", "draft"]
    }
  }
}'
```

**Search with Highlighting:**
```bash
curl -X POST "http://localhost:9200/my_index/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "match": {
      "title": "search"
    }
  },
  "highlight": {
    "fields": {
      "title": {}
    }
  }
}'
```

**Multi-Index Search with Wildcards:**
```bash
curl -X POST "http://localhost:9200/_search" -H 'Content-Type: application/json' -d'
{
  "index": "logs-*",
  "query": {
    "match_all": {}
  }
}'
```

#### Check Cluster Health

```bash
curl -X GET "http://localhost:9200/_cluster/health"
```

#### Get Aliases

```bash
curl -X GET "http://localhost:9200/_aliases"
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

### Configuration

Gummy Search supports configuration via YAML file or environment variables.

#### Configuration File

Create a `gummy-search.yaml` file in the project root (or copy `gummy-search.yaml.example`):

```yaml
server:
  host: "0.0.0.0"
  port: 9200

storage:
  data_dir: "./data"

logging:
  level: "info"
```

**Config file search order:**
1. Path specified in `GUMMY_CONFIG` environment variable
2. `./gummy-search.yaml`
3. `./config/gummy-search.yaml`
4. `~/.config/gummy-search/gummy-search.yaml`

#### Environment Variables

Environment variables override config file values (highest priority):

- `GUMMY_CONFIG` - Path to config file
- `GUMMY_HOST` - Server host (default: "0.0.0.0")
- `GUMMY_PORT` - Server port (default: 9200)
- `GUMMY_DATA_DIR` - Data directory path (default: "./data")
- `GUMMY_LOG_LEVEL` - Log level (default: "info")
- `GUMMY_ES_VERSION` - Elasticsearch compatibility version (default: "6.8.23")
- `RUST_LOG` - Log level (takes precedence over `GUMMY_LOG_LEVEL` and config file)

**Example:**
```bash
# Use custom port via environment variable
GUMMY_PORT=9300 cargo run

# Use custom config file
GUMMY_CONFIG=/path/to/config.yaml cargo run
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

Gummy Search implements a subset of Elasticsearch 6.8.23 API endpoints.

## Documentation

- **[API Routes](docs/ROUTES.md)** - Quick reference of all API routes organized by category
- **[API Documentation](docs/API.md)** - Complete API reference with examples
- **[Code Coverage](docs/CODE_COVERAGE.md)** - Test coverage report and analysis
- **[Usage Examples](docs/USAGE_EXAMPLES.md)** - Practical usage examples and scenarios
- **[Architecture](docs/ARCHITECTURE.md)** - System architecture and design
- **[API Requirements](docs/elasticsearch-api-requirements.md)** - Detailed API endpoint specifications
- **[Storage Options](docs/storage-options.md)** - Storage backend documentation
- **[TODO](docs/TODO.md)** - Development progress and upcoming features

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
- `GET /_cluster/stats` - Cluster statistics
- `GET /_cat/indices` - List indices (cat API)
- `GET /_aliases` - Get index aliases

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
