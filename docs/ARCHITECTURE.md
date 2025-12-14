# Gummy Search Architecture

This document describes the architecture and design of Gummy Search.

## Overview

Gummy Search is an Elasticsearch-compatible search engine written in Rust. It provides a RESTful API that mimics Elasticsearch 6.8.23 endpoints, making it a drop-in replacement for basic Elasticsearch use cases.

## System Architecture

```
┌─────────────────┐
│   HTTP Client   │
└────────┬────────┘
         │ HTTP/REST
         ▼
┌─────────────────┐
│  Axum Server    │  ← HTTP routing, request handling
│  (src/server.rs)│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   AppState      │  ← Shared application state
│  (Storage Arc)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Storage      │  ← Core storage and search logic
│ (src/storage.rs)│
└────────┬────────┘
         │
         ├─────────────────┐
         ▼                 ▼
┌─────────────────┐  ┌─────────────────┐
│  In-Memory      │  │  Sled Backend   │  ← Persistent storage
│  HashMap        │  │(storage_backend)│
└─────────────────┘  └─────────────────┘
```

## Components

### 1. HTTP Server Layer (`src/server.rs`)

**Responsibility:** Handle HTTP requests and responses

**Key Features:**
- Route definitions using Axum
- Request parsing and validation
- Response formatting
- Error handling

**Main Handlers:**
- Index management (create, get, delete, update)
- Document operations (CRUD)
- Search operations
- Bulk operations
- Cluster operations

### 2. Storage Layer (`src/storage.rs`)

**Responsibility:** Core data storage and search functionality

**Key Features:**
- Index management
- Document storage and retrieval
- Search query processing
- Scoring and ranking
- Highlighting
- Source filtering

**Data Structures:**
- `Storage`: Main storage struct with `Arc<RwLock<HashMap<String, Index>>>`
- `Index`: Contains documents, mappings, and settings
- Documents stored as `HashMap<String, serde_json::Value>`

### 3. Persistent Storage Backend (`src/storage_backend.rs`)

**Responsibility:** Persist data to disk using Sled

**Key Features:**
- Index metadata persistence
- Document persistence
- Data loading on startup
- Flush operations

**Storage Format:**
- Sled key-value database
- Keys: `index:{index_name}`, `doc:{index_name}:{doc_id}`
- Values: JSON-serialized metadata and documents

### 4. Configuration (`src/config.rs`)

**Responsibility:** Application configuration management

**Key Features:**
- YAML config file support
- Environment variable overrides
- Default values
- Server, storage, and logging configuration

**Config Sources (priority order):**
1. Environment variables (highest)
2. Config file (`gummy-search.yaml`)
3. Default values (lowest)

### 5. Error Handling (`src/error.rs`)

**Responsibility:** Custom error types and conversions

**Error Types:**
- `IndexNotFound`
- `DocumentNotFound`
- `InvalidRequest`
- `Storage`
- `TaskJoin`

## Data Flow

### Document Indexing Flow

```
1. HTTP Request → server.rs::index_document
2. Parse request body
3. Storage::index_document
   - Validate index exists
   - Store in memory (HashMap)
   - Persist to Sled (if backend available)
4. Return response
```

### Search Flow

```
1. HTTP Request → server.rs::search_post
2. Parse query, pagination, sorting, highlighting
3. Storage::search
   - Load index from memory
   - Score each document against query
   - Sort by score/custom sort
   - Apply pagination
   - Apply source filtering
   - Apply highlighting
4. Format response
5. Return JSON
```

### Bulk Operations Flow

```
1. HTTP Request → server.rs::bulk_operations
2. Parse NDJSON format
3. For each operation:
   - Parse action (index, create, update, delete)
   - Execute via Storage::execute_bulk_action
   - Collect results
4. Aggregate results
5. Return bulk response
```

## Storage Model

### In-Memory Structure

```rust
Storage {
    indices: Arc<RwLock<HashMap<String, Index>>>,
    backend: Option<Arc<SledBackend>>
}

Index {
    name: String,
    documents: HashMap<String, serde_json::Value>,
    mappings: Option<serde_json::Value>,
    settings: Option<serde_json::Value>
}
```

### Persistent Storage (Sled)

**Database Structure:**
- Tree: `indices` - stores index metadata
- Tree: `documents` - stores document data

**Key Format:**
- Index: `index:{index_name}`
- Document: `doc:{index_name}:{doc_id}`

**Value Format:**
- JSON-serialized metadata and documents

## Search Implementation

### Query Processing

1. **Query Parsing**: Extract query type and parameters
2. **Document Scoring**: Score each document against query
3. **Sorting**: Sort by score or custom sort fields
4. **Pagination**: Apply `from` and `size` parameters
5. **Post-processing**: Apply source filtering and highlighting

### Supported Query Types

- **Match**: Full-text search (case-insensitive substring match)
- **Match Phrase**: Exact phrase matching
- **Multi-Match**: Search across multiple fields
- **Term**: Exact value match
- **Terms**: Match any of multiple values
- **Wildcard**: Pattern matching with `*` and `?`
- **Prefix**: Prefix matching
- **Range**: Numeric/date range queries
- **Bool**: Boolean logic (must, should, must_not, filter)
- **Match All**: Return all documents

### Scoring Algorithm

Current implementation uses simple scoring:
- Exact match: 1.0
- Partial match: 0.5-0.9 (based on position)
- No match: 0.0

**Future Enhancement:** Implement TF-IDF or BM25 scoring

## Concurrency Model

### Thread Safety

- **Storage**: Uses `Arc<RwLock<>>` for shared access
- **Read Operations**: Multiple concurrent reads allowed
- **Write Operations**: Exclusive write access
- **Backend Operations**: Uses `spawn_blocking` for async I/O

### Async/Await

- Built on Tokio runtime
- All I/O operations are async
- Blocking operations (Sled) run in `spawn_blocking`

## Configuration

### Config File (`gummy-search.yaml`)

```yaml
server:
  host: "0.0.0.0"
  port: 9200

storage:
  data_dir: "./data"

logging:
  level: "info"
```

### Environment Variables

- `GUMMY_CONFIG`: Path to config file
- `GUMMY_HOST`: Server host
- `GUMMY_PORT`: Server port
- `GUMMY_DATA_DIR`: Data directory
- `GUMMY_LOG_LEVEL`: Log level
- `RUST_LOG`: Log level (takes precedence)

## Error Handling

### Error Types

All errors are converted to `GummySearchError`:
- HTTP status codes mapped to error types
- Detailed error messages
- Proper error propagation

### Error Responses

```json
{
  "error": {
    "type": "index_not_found_exception",
    "reason": "Index 'my_index' not found"
  }
}
```

## Performance Considerations

### Current Limitations

1. **Linear Search**: All documents are scanned for each query
2. **No Inverted Index**: Full-text search is O(n) where n is document count
3. **In-Memory Only**: All data must fit in memory

### Future Optimizations

1. **Inverted Index**: Build term-to-document mappings
2. **Tokenization**: Proper text analysis and tokenization
3. **Caching**: Cache frequent queries
4. **Indexing**: Pre-compute common query results

## Testing

### Test Structure

- **Unit Tests**: Test individual functions in isolation
- **Integration Tests**: Test full workflows end-to-end
- **Persistence Tests**: Verify data survives restarts

### Test Coverage

- Index management operations
- Document CRUD operations
- Search queries (all types)
- Bulk operations
- Highlighting and source filtering
- Wildcard index patterns

## Dependencies

### Core Dependencies

- **axum**: HTTP server framework
- **tokio**: Async runtime
- **serde/serde_json**: JSON serialization
- **sled**: Persistent key-value storage
- **regex**: Pattern matching for wildcards
- **tracing**: Structured logging

### Development Dependencies

- **tokio-test**: Async testing utilities
- **axum-test**: HTTP testing utilities
- **tempfile**: Temporary file handling for tests

## Deployment

### Build

```bash
cargo build --release
```

### Run

```bash
./target/release/gummy-search
```

### Docker

```bash
docker build -t gummy-search .
docker run -p 9200:9200 gummy-search
```

## Limitations

1. **No Distributed Mode**: Single-node only
2. **No Sharding**: All data in one index
3. **No Replication**: No replica support
4. **Simple Scoring**: Basic relevance scoring
5. **No Aggregations**: Aggregation queries not supported
6. **No Tokenization**: Simple text matching, no advanced analysis

## Future Enhancements

1. **Inverted Index**: For faster full-text search
2. **Tokenization**: Proper text analysis
3. **Advanced Scoring**: TF-IDF, BM25
4. **Aggregations**: Support aggregation queries
5. **Distributed Mode**: Multi-node support
6. **Sharding**: Split indices across shards
7. **Replication**: Replica support
