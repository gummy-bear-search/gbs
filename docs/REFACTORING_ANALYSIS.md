# Code Refactoring Analysis

## File Size Overview

| File | Lines | Status | Priority |
|------|-------|--------|----------|
| `src/storage.rs` | **2,182** | 🔴 Too Large | **HIGH** |
| `src/server.rs` | **702** | 🟡 Large | **MEDIUM** |
| `src/config.rs` | 229 | ✅ OK | - |
| `src/bulk_ops.rs` | 204 | ✅ OK | - |
| `src/storage_backend.rs` | 201 | ✅ OK | - |
| `src/models.rs` | 196 | ✅ OK | - |
| Others | < 60 | ✅ OK | - |

## 1. `src/storage.rs` (2,182 lines) - CRITICAL

### Current Structure
This file contains:
- `Index` struct and implementation
- `Storage` struct and implementation
- All search query logic (match, term, bool, range, wildcard, etc.)
- Highlighting logic
- Source filtering logic
- Query term extraction
- Text tokenization
- Statistics and monitoring functions
- Index management (CRUD)
- Document management (CRUD)
- Bulk operations execution
- Tests (at the end)

### Recommended Decomposition

#### Option A: Feature-based Modules (Recommended)

```
src/
├── storage/
│   ├── mod.rs              # Re-exports and main Storage struct
│   ├── index.rs            # Index struct and basic operations (~200 lines)
│   ├── storage.rs          # Storage struct, initialization, persistence (~300 lines)
│   ├── index_ops.rs        # Index CRUD operations (~300 lines)
│   ├── document_ops.rs     # Document CRUD operations (~200 lines)
│   ├── search/
│   │   ├── mod.rs          # Search entry point
│   │   ├── query.rs        # Query parsing and execution (~400 lines)
│   │   ├── matchers.rs     # Match, term, range, wildcard queries (~500 lines)
│   │   ├── bool_query.rs   # Bool query logic (~200 lines)
│   │   ├── highlighting.rs # Highlighting logic (~200 lines)
│   │   └── scoring.rs      # Scoring and ranking (~200 lines)
│   ├── stats.rs            # Statistics and monitoring (~200 lines)
│   └── tests.rs            # All tests (~300 lines)
```

**Benefits:**
- Clear separation of concerns
- Easier to test individual components
- Better code navigation
- Reduced compilation time for changes

#### Option B: Layer-based Modules

```
src/
├── storage/
│   ├── mod.rs
│   ├── storage.rs          # Main Storage struct
│   ├── index.rs            # Index struct
│   ├── operations/
│   │   ├── mod.rs
│   │   ├── index.rs        # Index operations
│   │   ├── document.rs     # Document operations
│   │   └── bulk.rs         # Bulk operations
│   ├── search/
│   │   ├── mod.rs
│   │   ├── engine.rs       # Main search engine
│   │   ├── queries.rs      # Query types
│   │   └── highlighting.rs  # Highlighting
│   └── utils/
│       ├── mod.rs
│       ├── filtering.rs    # Source filtering
│       └── stats.rs        # Statistics
```

### Functions to Extract from `storage.rs`

1. **Search-related functions** (~800 lines):
   - `search()` - Main search function
   - `match_field()`, `match_phrase_field()`, `multi_match_fields()`
   - `term_match()`, `terms_match()`, `range_match()`, `wildcard_match()`, `prefix_match()`
   - `bool_query()`, `match_all_query()`
   - `highlight_document()`, `highlight_text()`, `extract_query_terms()`, `tokenize_query()`
   - `filter_source()`

2. **Index operations** (~400 lines):
   - `create_index()`, `delete_index()`, `get_index()`, `update_mapping()`, `update_settings()`
   - `match_indices()`, `get_indices_stats()`, `get_aliases()`

3. **Document operations** (~200 lines):
   - `index_document()`, `create_document()`, `get_document()`, `delete_document()`

4. **Statistics** (~200 lines):
   - `get_cluster_stats()`, `get_indices_stats()`

5. **Tests** (~300 lines):
   - All test functions

### Implementation Steps

1. Create `src/storage/mod.rs` with module declarations
2. Extract `Index` struct to `src/storage/index.rs`
3. Create `src/storage/search/mod.rs` and extract search functions
4. Extract query matchers to separate files
5. Extract highlighting to `src/storage/search/highlighting.rs`
6. Extract index operations to `src/storage/index_ops.rs`
7. Extract document operations to `src/storage/document_ops.rs`
8. Extract statistics to `src/storage/stats.rs`
9. Move tests to `src/storage/tests.rs` or keep in respective modules
10. Update imports throughout codebase

---

## 2. `src/server.rs` (702 lines) - MEDIUM PRIORITY

### Current Structure
This file contains:
- Route definitions
- All HTTP handlers (20+ handlers)
- WebSocket handler
- Request/response processing

### Recommended Decomposition

```
src/
├── server/
│   ├── mod.rs              # Router setup and AppState
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── cluster.rs      # Cluster health, stats (~100 lines)
│   │   ├── index.rs        # Index CRUD handlers (~150 lines)
│   │   ├── document.rs    # Document CRUD handlers (~150 lines)
│   │   ├── search.rs       # Search handlers (~150 lines)
│   │   ├── bulk.rs         # Bulk operations handler (~100 lines)
│   │   └── websocket.rs    # WebSocket handler (~100 lines)
│   └── routes.rs           # Route definitions (~50 lines)
```

### Functions to Extract

1. **Cluster handlers** (~50 lines):
   - `cluster_health()`, `cluster_stats()`, `cat_indices()`, `get_aliases()`

2. **Index handlers** (~150 lines):
   - `create_index()`, `get_index()`, `delete_index()`, `check_index()`
   - `update_mapping()`, `update_settings()`

3. **Document handlers** (~150 lines):
   - `index_document()`, `create_document()`, `get_document()`, `delete_document()`

4. **Search handlers** (~150 lines):
   - `search_get()`, `search_post()`, `search_multi_index()`

5. **Bulk handler** (~100 lines):
   - `bulk_operations()`

6. **WebSocket handler** (~100 lines:
   - `websocket_handler()`, `handle_socket()`

7. **Utility handlers** (~20 lines):
   - `root()`, `web_index()`

### Implementation Steps

1. Create `src/server/mod.rs` with module structure
2. Create `src/server/handlers/mod.rs`
3. Extract handlers by category to separate files
4. Create `src/server/routes.rs` for route definitions
5. Update `src/lib.rs` to use new module structure

---

## 3. Other Files - No Changes Needed

- `src/config.rs` (229 lines) - Well-structured, no changes needed
- `src/bulk_ops.rs` (204 lines) - Focused on bulk parsing, OK
- `src/storage_backend.rs` (201 lines) - Backend abstraction, OK
- `src/models.rs` (196 lines) - Data models, OK

---

## Refactoring Priority

### Phase 1: High Impact (Do First)
1. ✅ Extract search logic from `storage.rs` to `storage/search/`
   - **Impact**: Reduces `storage.rs` by ~800 lines
   - **Risk**: Low (search is self-contained)
   - **Time**: 2-3 hours

2. ✅ Extract index operations from `storage.rs` to `storage/index_ops.rs`
   - **Impact**: Reduces `storage.rs` by ~400 lines
   - **Risk**: Low
   - **Time**: 1-2 hours

### Phase 2: Medium Impact
3. ✅ Extract document operations from `storage.rs` to `storage/document_ops.rs`
   - **Impact**: Reduces `storage.rs` by ~200 lines
   - **Risk**: Low
   - **Time**: 1 hour

4. ✅ Extract handlers from `server.rs` to `server/handlers/`
   - **Impact**: Reduces `server.rs` by ~600 lines
   - **Risk**: Medium (many route dependencies)
   - **Time**: 2-3 hours

### Phase 3: Polish
5. ✅ Extract statistics to `storage/stats.rs`
6. ✅ Extract tests to separate test modules
7. ✅ Add module-level documentation

---

## Benefits of Refactoring

1. **Maintainability**: Easier to find and modify specific functionality
2. **Testability**: Smaller modules are easier to test in isolation
3. **Compilation**: Faster incremental compilation (only changed modules recompile)
4. **Code Navigation**: Better IDE support and code navigation
5. **Team Collaboration**: Multiple developers can work on different modules
6. **Code Review**: Smaller PRs are easier to review

---

## Risks and Mitigation

### Risks
1. **Breaking Changes**: Import paths will change
2. **Circular Dependencies**: Need careful module design
3. **Test Failures**: Tests may need updates

### Mitigation
1. Use `cargo check` and `cargo test` after each extraction
2. Keep public API stable (use `pub use` for re-exports)
3. Extract one module at a time and test immediately
4. Update imports incrementally

---

## Estimated Time

- **Phase 1**: 3-5 hours
- **Phase 2**: 3-4 hours
- **Phase 3**: 2-3 hours
- **Total**: 8-12 hours

---

## Next Steps

1. Review this analysis
2. Decide on decomposition approach (Option A or B for storage)
3. Start with Phase 1 (search extraction)
4. Test thoroughly after each extraction
5. Update documentation
