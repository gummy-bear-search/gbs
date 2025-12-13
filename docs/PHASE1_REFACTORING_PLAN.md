# Phase 1 Refactoring - Detailed Plan

## Current Status
- `src/storage.rs`: 2,182 lines
- Target: Reduce to ~1,000 lines by extracting search and index operations

## Extraction Strategy

### Step 1: Create Module Structure ✅
- [x] `src/storage/index.rs` - Index struct
- [x] `src/storage/search/mod.rs` - Search module entry
- [x] `src/storage/search/utils.rs` - Utility functions (get_field_value, filter_source, compare_documents)

### Step 2: Extract Search Functions (Next)
Create `src/storage/search/query.rs` with:
- `score_document()` - Main query scoring function
- `score_bool_query()` - Bool query scoring

Create `src/storage/search/matchers.rs` with:
- `match_field()` - Match query
- `match_all_fields()` - Match all fields
- `match_phrase_field()` - Match phrase query
- `match_phrase_all_fields()` - Match phrase all fields
- `multi_match_fields()` - Multi-match query
- `term_match()` - Term query
- `terms_match()` - Terms query
- `range_match()` - Range query
- `wildcard_match()` - Wildcard query
- `prefix_match()` - Prefix query
- Helper functions: `search_value()`, `search_phrase_value()`, `words_in_order()`

### Step 3: Extract Highlighting
Create `src/storage/search/highlighting.rs` with:
- `highlight_document()` - Main highlighting function
- `highlight_text()` - Text highlighting
- `extract_query_terms()` - Extract terms from query
- `tokenize_query()` - Tokenize query string

### Step 4: Extract Index Operations
Create `src/storage/index_ops.rs` with:
- `create_index()`
- `delete_index()`
- `get_index()`
- `update_mapping()`
- `update_settings()`
- `match_indices()`
- `get_indices_stats()`
- `get_aliases()`

### Step 5: Update storage.rs
- Keep Storage struct and initialization
- Keep document operations (for now)
- Keep bulk operations (for now)
- Keep persistence logic
- Delegate search to search module
- Delegate index ops to index_ops module

## Implementation Order

1. ✅ Create module structure
2. ✅ Extract Index struct
3. ✅ Extract utility functions
4. ⏳ Extract search query functions
5. ⏳ Extract search matchers
6. ⏳ Extract highlighting
7. ⏳ Extract index operations
8. ⏳ Update storage.rs to use new modules
9. ⏳ Test compilation
10. ⏳ Run tests

## Estimated Lines Reduction

- Search functions: ~800 lines → `storage/search/`
- Index operations: ~400 lines → `storage/index_ops.rs`
- Utilities: ~100 lines → `storage/search/utils.rs`
- **Total reduction: ~1,300 lines**
- **Remaining in storage.rs: ~880 lines**
