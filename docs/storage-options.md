# Persistent Storage Options for Gummy Bear Search

## Current State
- **Current Implementation**: In-memory `HashMap<String, Index>` with `Arc<RwLock<>>`
- **Limitation**: Data is lost on server restart
- **Use Case**: MVP/Development

## Storage Options Comparison

### 1. RocksDB ⭐ (Recommended for Production)

**Description**: High-performance embedded key-value store from Facebook/Meta, written in C++ with Rust bindings.

**Pros**:
- ✅ Excellent performance (used by many production systems)
- ✅ ACID transactions
- ✅ Built-in compression
- ✅ Bloom filters for fast lookups
- ✅ Range queries support
- ✅ Very mature and battle-tested
- ✅ Good for large datasets (terabytes+)
- ✅ Configurable write-ahead logging

**Cons**:
- ❌ C++ dependency (larger binary)
- ❌ More complex configuration
- ❌ Requires tuning for optimal performance
- ❌ Slightly more complex API

**Rust Crate**: `rocksdb`

**Example Usage**:
```rust
use rocksdb::{DB, Options};

let mut opts = Options::default();
opts.create_if_missing(true);
let db = DB::open(&opts, "/path/to/db").unwrap();

// Store index metadata
db.put(b"index:my_index", b"metadata").unwrap();

// Store document
db.put(b"doc:my_index:1", b"document_json").unwrap();
```

**Best For**: Production deployments, high-throughput scenarios, large datasets

---

### 2. Sled ⭐ (Recommended for Development/Medium Scale)

**Description**: Pure Rust embedded database with a B-tree-based key-value store.

**Pros**:
- ✅ Pure Rust (no C dependencies)
- ✅ Simple, ergonomic API
- ✅ ACID transactions
- ✅ Concurrent reads
- ✅ Lightweight
- ✅ Good documentation
- ✅ Easy to integrate

**Cons**:
- ❌ Less mature than RocksDB
- ❌ Smaller community
- ❌ May have performance limits at very large scale
- ❌ Occasional stability issues reported

**Rust Crate**: `sled`

**Example Usage**:
```rust
use sled::Db;

let db = sled::open("/path/to/db").unwrap();

// Store index metadata
db.insert(b"index:my_index", b"metadata").unwrap();

// Store document
db.insert(b"doc:my_index:1", b"document_json").unwrap();
```

**Best For**: Development, medium-scale production, when you want pure Rust

---

### 3. SQLite

**Description**: Lightweight, serverless SQL database engine.

**Pros**:
- ✅ Battle-tested and reliable
- ✅ SQL queries (flexible)
- ✅ ACID transactions
- ✅ Small footprint
- ✅ Widely used and understood
- ✅ Good tooling

**Cons**:
- ❌ SQL overhead for simple key-value operations
- ❌ Single-writer concurrency model
- ❌ May be slower for pure key-value patterns
- ❌ Requires schema design

**Rust Crates**: `rusqlite` (synchronous) or `sqlx` (async)

**Example Usage**:
```rust
use rusqlite::{Connection, Result};

let conn = Connection::open("/path/to/db.db")?;
conn.execute(
    "CREATE TABLE IF NOT EXISTS documents (
        index_name TEXT,
        doc_id TEXT,
        data TEXT,
        PRIMARY KEY (index_name, doc_id)
    )",
    [],
)?;
```

**Best For**: When you need SQL queries, simpler deployments, relational data

---

### 4. Redb (Newer Option)

**Description**: Pure Rust embedded database with a focus on simplicity and performance.

**Pros**:
- ✅ Pure Rust
- ✅ Simple API
- ✅ Good performance
- ✅ ACID transactions
- ✅ Multi-version concurrency control

**Cons**:
- ❌ Newer project (less battle-tested)
- ❌ Smaller community
- ❌ Less documentation/examples

**Rust Crate**: `redb`

**Best For**: New projects wanting pure Rust, willing to try newer tech

---

### 5. File-Based Storage (JSON/TOML/Binary)

**Description**: Serialize data structures directly to files.

**Pros**:
- ✅ No external dependencies
- ✅ Full control
- ✅ Simple implementation
- ✅ Easy to backup/restore

**Cons**:
- ❌ No transactions
- ❌ Manual concurrency handling
- ❌ Slower at scale
- ❌ No built-in indexing
- ❌ Risk of data corruption

**Best For**: Prototyping, very small datasets, learning

---

## Recommendation Matrix

| Scenario | Recommended Option | Reason |
|----------|-------------------|---------|
| **Development/MVP** | Sled | Easy to use, pure Rust, good enough for dev |
| **Production (Small-Medium)** | Sled | Pure Rust, good performance, simpler than RocksDB |
| **Production (Large Scale)** | RocksDB | Best performance, proven at scale |
| **Need SQL Queries** | SQLite | When you need relational queries |
| **Prototyping** | File-based | Quick to implement, no dependencies |

## Implementation Strategy

### Phase 1: Abstract Storage Trait
Create a storage trait to allow switching backends:

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn create_index(&self, name: &str, settings: Option<Value>, mappings: Option<Value>) -> Result<()>;
    async fn index_document(&self, index: &str, id: &str, doc: Value) -> Result<()>;
    async fn get_document(&self, index: &str, id: &str) -> Result<Value>;
    async fn search(&self, index: &str, query: &Value, from: Option<u32>, size: Option<u32>, sort: Option<&Value>) -> Result<Value>;
    // ... other methods
}
```

### Phase 2: Implement Sled Backend
Start with Sled for easier development and pure Rust.

### Phase 3: Add RocksDB Backend (Optional)
Add RocksDB as an alternative for production deployments.

## Migration Path

1. **Current**: In-memory HashMap
2. **Next**: Add storage trait abstraction
3. **Implement**: Sled backend (easiest migration)
4. **Optional**: Add RocksDB backend for production
5. **Future**: Support multiple backends via configuration

## Performance Considerations

- **Write Performance**: RocksDB > Sled > SQLite > File-based
- **Read Performance**: RocksDB ≈ Sled > SQLite > File-based
- **Memory Usage**: File-based < SQLite < Sled < RocksDB
- **Setup Complexity**: File-based < Sled < SQLite < RocksDB

## Data Structure Design

For any persistent backend, consider:

```
Storage Layout:
- index:{index_name} -> Index metadata (settings, mappings)
- doc:{index_name}:{doc_id} -> Document JSON
- search:{index_name}:{term} -> Inverted index (future)
```

## Next Steps

1. Create `StorageBackend` trait
2. Implement `SledStorage` backend
3. Update `Storage` to use backend trait
4. Add configuration option to choose backend
5. Test persistence (restart server, verify data)
