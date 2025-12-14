# Code Coverage Report & Improvement Plan

**Last Updated:** 2025-01-12
**Tool:** cargo-tarpaulin v0.34.1
**Status:** Active improvement plan in progress

## Overall Coverage

**42.40%** coverage
**708 / 1,670** lines covered

## Test Statistics

- **Total Tests:** 28
  - Storage unit tests: 13
  - Config tests: 2
  - Integration tests: 11
  - Persistence tests: 2
- **Test Files:** 4
- **Source Files:** 40

## Coverage by Module

### ✅ Storage Module (Well Tested)

The storage module has the best test coverage, with core functionality well-tested.

| File | Coverage | Lines Covered | Total Lines |
|------|----------|---------------|-------------|
| `storage/index.rs` | **100.0%** | 3/3 | ✅ Complete |
| `storage/persistence.rs` | **97.6%** | 41/42 | ✅ Excellent |
| `storage/search_impl.rs` | **87.0%** | 60/69 | ✅ Excellent |
| `storage/search/utils.rs` | **82.1%** | 55/67 | ✅ Good |
| `storage/mod.rs` | **80.8%** | 42/52 | ✅ Good |
| `storage/search/query.rs` | **77.6%** | 83/107 | ✅ Good |
| `storage/search/highlighting.rs` | **72.0%** | 72/100 | ✅ Good |
| `storage_backend.rs` | **56.3%** | 58/103 | ⚠️ Moderate |
| `storage/index_ops.rs` | **53.3%** | 97/182 | ⚠️ Moderate |
| `storage/search/matchers.rs` | **49.5%** | 108/218 | ⚠️ Moderate |
| `storage/document_ops.rs` | **39.4%** | 39/99 | ⚠️ Needs Improvement |

**Storage Module Average:** ~70% coverage

### ❌ Server Module (Not Tested)

The HTTP server layer has **0% coverage**. All handlers and routes are untested.

| File | Coverage | Lines Covered | Total Lines |
|------|----------|---------------|-------------|
| `server/handlers/search.rs` | **0%** | 0/85 | ❌ No coverage |
| `server/handlers/websocket.rs` | **0%** | 0/69 | ❌ No coverage |
| `server/handlers/bulk.rs` | **0%** | 0/50 | ❌ No coverage |
| `server/handlers/index.rs` | **0%** | 0/44 | ❌ No coverage |
| `server/handlers/cluster.rs` | **0%** | 0/36 | ❌ No coverage |
| `server/handlers/document.rs` | **0%** | 0/21 | ❌ No coverage |
| `server/routes/mod.rs` | **0%** | 0/14 | ❌ No coverage |
| `server/routes/index.rs` | **0%** | 0/8 | ❌ No coverage |
| `server/routes/cluster.rs` | **0%** | 0/6 | ❌ No coverage |
| `server/routes/document.rs` | **0%** | 0/6 | ❌ No coverage |
| `server/routes/search.rs` | **0%** | 0/5 | ❌ No coverage |
| `server/routes/web.rs` | **0%** | 0/5 | ❌ No coverage |
| `server/routes/bulk.rs` | **0%** | 0/4 | ❌ No coverage |
| `server/routes/refresh.rs` | **0%** | 0/4 | ❌ No coverage |
| `server/routes/websocket.rs` | **0%** | 0/3 | ❌ No coverage |
| `server/mod.rs` | **0%** | 0/6 | ❌ No coverage |

**Server Module Total:** 0/361 lines (0% coverage)

### ⚠️ Other Modules

| File | Coverage | Lines Covered | Total Lines |
|------|----------|---------------|-------------|
| `config.rs` | **43.5%** | 27/62 | ⚠️ Moderate |
| `bulk_ops.rs` | **35.9%** | 23/64 | ⚠️ Needs Improvement |
| `storage/stats.rs` | **0%** | 0/101 | ❌ No coverage |
| `error.rs` | **0%** | 0/14 | ❌ No coverage |
| `client.rs` | **0%** | 0/2 | ❌ No coverage |
| `main.rs` | **0%** | 0/19 | ❌ No coverage (expected) |

## Key Findings

### ✅ Strengths

1. **Core Storage Logic:** ~70% coverage
   - Search functionality is well-tested
   - Persistence operations have excellent coverage (97.6%)
   - Index operations are moderately tested

2. **Search Functionality:** Well-tested
   - Query parsing: 77.6%
   - Highlighting: 72.0%
   - Search implementation: 87.0%

3. **Configuration:** Basic tests present (43.5%)

### ❌ Critical Gaps

1. **HTTP Handlers:** 0% coverage (305 lines)
   - All 21 handler functions are untested
   - No integration tests for HTTP endpoints
   - **Impact:** High - this is the public API

2. **Statistics Module:** 0% coverage (101 lines)
   - `get_indices_stats()` - untested
   - `get_aliases()` - untested
   - `get_cluster_stats()` - untested
   - **Impact:** Medium - monitoring functionality

3. **Error Handling:** 0% coverage (14 lines)
   - Error type conversions untested
   - Error response formatting untested
   - **Impact:** Medium - important for reliability

4. **Storage Edge Cases:** Some gaps
   - Document operations: 39.4% (needs improvement)
   - Matchers: 49.5% (many edge cases untested)
   - Index operations: 53.3% (some edge cases missing)

## Action Plan to Increase Coverage

### Current Status
- **Coverage:** 42.40% (708/1,670 lines)
- **Target:** 75%+ overall coverage
- **Gap:** ~545 lines need testing

### Priority 1: HTTP Handler Tests (Highest Impact) 🔴

**Expected Coverage Gain:** +18-20%
**Target Files:** 15 handler/route files (361 lines total)
**Estimated Time:** 8-12 hours

#### Implementation Plan

1. **Set up HTTP testing infrastructure**
   - [ ] Add `axum-test` dependency (or similar)
   - [ ] Create test helper utilities for setting up test server
   - [ ] Create test fixtures for common test data

2. **Index Management Handlers** (`server/handlers/index.rs` - 44 lines)
   - [ ] Test `create_index()` - success and error cases
   - [ ] Test `get_index()` - existing and non-existing indices
   - [ ] Test `delete_index()` - single index and `_all`
   - [ ] Test `check_index()` - HEAD endpoint
   - [ ] Test `update_mapping()` - valid and invalid mappings
   - [ ] Test `update_settings()` - valid and invalid settings

3. **Document Handlers** (`server/handlers/document.rs` - 21 lines)
   - [ ] Test `index_document()` - PUT with ID
   - [ ] Test `create_document()` - POST without ID (auto-generate)
   - [ ] Test `get_document()` - existing and non-existing documents
   - [ ] Test `delete_document()` - success and 404 cases

4. **Search Handlers** (`server/handlers/search.rs` - 85 lines)
   - [ ] Test `search_post()` - all query types
   - [ ] Test `search_get()` - query parameter parsing
   - [ ] Test `search_multi_index()` - multiple indices and wildcards
   - [ ] Test pagination, sorting, highlighting, source filtering

5. **Bulk Operations Handler** (`server/handlers/bulk.rs` - 50 lines)
   - [ ] Test bulk index operations
   - [ ] Test bulk create operations
   - [ ] Test bulk update operations
   - [ ] Test bulk delete operations
   - [ ] Test mixed operations
   - [ ] Test error handling (partial failures)
   - [ ] Test refresh parameter

6. **Cluster Handlers** (`server/handlers/cluster.rs` - 36 lines)
   - [ ] Test `cluster_health()` - return format
   - [ ] Test `cluster_stats()` - statistics calculation
   - [ ] Test `cat_indices()` - verbose and non-verbose modes
   - [ ] Test `get_aliases()` - alias retrieval

7. **Refresh Handlers** (`server/routes/refresh.rs` - 4 lines)
   - [ ] Test single index refresh
   - [ ] Test all indices refresh

8. **WebSocket Handler** (`server/handlers/websocket.rs` - 69 lines)
   - [ ] Test WebSocket connection establishment
   - [ ] Test initial message sending (health, stats, indices)
   - [ ] Test periodic updates
   - [ ] Test connection closure

**Approach:** Use `axum-test` crate for HTTP endpoint testing. Create integration test suite in `tests/integration_http.rs`.

### Priority 2: Statistics Module Tests 🟡

**Expected Coverage Gain:** +6%
**Target File:** `storage/stats.rs` (101 lines)
**Estimated Time:** 2-3 hours

#### Implementation Plan

1. **Set up test fixtures**
   - [ ] Create helper to build test indices with documents
   - [ ] Create helper to build test storage state

2. **Test `get_indices_stats()`**
   - [ ] Test with empty storage
   - [ ] Test with single index
   - [ ] Test with multiple indices
   - [ ] Test document count accuracy
   - [ ] Test with indices of varying sizes

3. **Test `get_aliases()`**
   - [ ] Test with no aliases (current implementation)
   - [ ] Test return format matches Elasticsearch structure
   - [ ] Test with multiple indices

4. **Test `get_cluster_stats()`**
   - [ ] Test cluster name and UUID
   - [ ] Test timestamp format
   - [ ] Test status calculation (green/yellow/red)
   - [ ] Test indices statistics aggregation
   - [ ] Test nodes statistics
   - [ ] Test with different ES versions (6.8.23)
   - [ ] Test with empty cluster
   - [ ] Test with populated cluster

**Approach:** Add unit tests in `tests/stats_test.rs` or add to existing test modules.

### Priority 3: Error Handling Tests 🟡

**Expected Coverage Gain:** +1%
**Target File:** `error.rs` (14 lines)
**Estimated Time:** 1-2 hours

#### Implementation Plan

1. **Test Error Type Conversions**
   - [ ] Test `From<serde_json::Error>` conversion
   - [ ] Test `From<std::io::Error>` conversion
   - [ ] Test `From<anyhow::Error>` conversion
   - [ ] Test `From<sled::Error>` conversion

2. **Test Error Response Formatting**
   - [ ] Test error serialization to JSON
   - [ ] Test HTTP status code mapping
   - [ ] Test error message formatting

3. **Test Error Propagation**
   - [ ] Test error chain preservation
   - [ ] Test error context addition

**Approach:** Add unit tests in `tests/error_test.rs`.

### Priority 4: Storage Edge Cases 🟢

**Expected Coverage Gain:** +5%
**Target Files:** Multiple storage modules
**Estimated Time:** 4-6 hours

#### Implementation Plan

1. **Document Operations** (`storage/document_ops.rs` - currently 39.4%)
   - [ ] Test document update conflicts
   - [ ] Test missing document scenarios
   - [ ] Test document with special characters
   - [ ] Test very large documents
   - [ ] Test concurrent document operations
   - [ ] Test document with nested structures

2. **Matchers** (`storage/search/matchers.rs` - currently 49.5%)
   - [ ] Test all query types with edge cases:
     - [ ] Match query: empty strings, special characters, unicode
     - [ ] Term query: case sensitivity, type mismatches
     - [ ] Range query: boundary conditions, invalid ranges
     - [ ] Wildcard query: complex patterns, escaping
     - [ ] Prefix query: empty prefix, long prefixes
     - [ ] Terms query: empty arrays, large arrays
   - [ ] Test matchers with missing fields
   - [ ] Test matchers with null values
   - [ ] Test matchers with array fields

3. **Index Operations** (`storage/index_ops.rs` - currently 53.3%)
   - [ ] Test concurrent index creation
   - [ ] Test invalid index names
   - [ ] Test index operations on non-existent indices
   - [ ] Test mapping updates with incompatible changes
   - [ ] Test settings updates with invalid values
   - [ ] Test index deletion with active operations

4. **Storage Backend** (`storage_backend.rs` - currently 56.3%)
   - [ ] Test persistence edge cases
   - [ ] Test data corruption recovery
   - [ ] Test concurrent access scenarios

**Approach:** Add focused unit tests for edge cases in existing test files or create new test modules.

## Coverage Goals & Milestones

### Milestone 1: 60% Coverage (Short-term - 2-3 weeks)
- **Current:** 42.40%
- **Target:** 60%
- **Gap:** +17.6% (~294 lines)
- **Focus:**
  - ✅ Priority 1: HTTP Handler Tests (complete)
  - ✅ Priority 2: Statistics Module Tests (complete)
- **Estimated Time:** 10-15 hours

### Milestone 2: 70% Coverage (Medium-term - 1-2 months)
- **Target:** 70%
- **Gap:** +10% (~167 lines)
- **Focus:**
  - ✅ Priority 3: Error Handling Tests (complete)
  - ✅ Priority 4: Storage Edge Cases (partial)
- **Estimated Time:** 5-8 hours

### Milestone 3: 75%+ Coverage (Long-term - 2-3 months)
- **Target:** 75%+
- **Gap:** +5% (~84 lines)
- **Focus:**
  - ✅ Complete Priority 4: All Storage Edge Cases
  - ✅ Additional edge cases and error paths
  - ✅ Integration test improvements
- **Estimated Time:** 4-6 hours

### Long-term Goal: 85%+ Coverage
- **Target:** 85%+
- **Focus:** Comprehensive edge case coverage, all error paths
- **Note:** Some code paths (like `main.rs`) are intentionally not tested

## Implementation Timeline

### Week 1-2: HTTP Handler Tests
- Set up testing infrastructure
- Implement index management handler tests
- Implement document handler tests
- **Expected Coverage:** 42.40% → ~50%

### Week 3: Search & Bulk Handler Tests
- Implement search handler tests
- Implement bulk operations handler tests
- **Expected Coverage:** 50% → ~58%

### Week 4: Cluster & Statistics Tests
- Implement cluster handler tests
- Implement statistics module tests
- **Expected Coverage:** 58% → ~64%

### Week 5-6: Error Handling & Edge Cases
- Implement error handling tests
- Start storage edge case tests
- **Expected Coverage:** 64% → ~70%

### Week 7-8: Complete Edge Cases
- Complete storage edge case tests
- Additional integration tests
- **Expected Coverage:** 70% → ~75%+

## Running Coverage Reports

### Install cargo-tarpaulin

```bash
cargo install cargo-tarpaulin
```

### Generate Coverage Report

```bash
# Full coverage report
cargo tarpaulin --out stdout --timeout 120

# Exclude test files and examples
cargo tarpaulin --out stdout --timeout 120 --exclude-files 'tests/*' 'example_laravel_app/*'

# Generate HTML report
cargo tarpaulin --out Html --output-dir coverage

# Generate XML report (for CI/CD)
cargo tarpaulin --out Xml --output-dir coverage
```

### View HTML Report

After generating HTML report:

```bash
open coverage/tarpaulin-report.html
```

## Notes

- Coverage is measured at the line level
- `main.rs` is expected to have 0% coverage (entry point, not unit tested)
- Integration tests provide some indirect coverage but don't show up in line coverage
- Some modules like `client.rs` are placeholders and may not need full coverage

## Coverage History

| Date | Coverage | Notes |
|------|----------|-------|
| 2025-01-12 | 42.40% | Initial coverage report after test reorganization |
| 2025-01-XX | TBD | After HTTP handler tests (Target: ~60%) |
| 2025-XX-XX | TBD | After statistics module tests (Target: ~64%) |
| 2025-XX-XX | TBD | After error handling tests (Target: ~65%) |
| 2025-XX-XX | TBD | After storage edge cases (Target: ~70%) |
| 2025-XX-XX | TBD | Final milestone (Target: 75%+) |

## Testing Tools & Setup

### Required Dependencies

Add to `Cargo.toml`:

```toml
[dev-dependencies]
axum-test = "9.1"  # For HTTP endpoint testing
tokio-test = "0.4"  # For async test utilities
```

### Test File Structure

```
tests/
├── integration.rs          # Existing integration tests
├── persistence_test.rs     # Existing persistence tests
├── storage.rs              # Existing storage tests
├── config.rs              # Existing config tests
├── integration_http.rs     # NEW: HTTP handler tests
├── stats_test.rs          # NEW: Statistics module tests
├── error_test.rs          # NEW: Error handling tests
└── edge_cases_test.rs     # NEW: Storage edge case tests
```

### Running Coverage Reports

```bash
# Generate coverage report
cargo tarpaulin --out stdout --timeout 120

# Generate HTML report for detailed analysis
cargo tarpaulin --out Html --output-dir coverage

# View HTML report
open coverage/tarpaulin-report.html
```

### CI/CD Integration

Consider adding coverage checks to CI/CD:

```yaml
# Example GitHub Actions
- name: Run coverage
  run: cargo tarpaulin --out Xml --output-dir coverage

- name: Upload coverage
  uses: codecov/codecov-action@v3
  with:
    files: ./coverage/cobertura.xml
```
