# Elasticsearch 7.x Compatibility Roadmap

## Overview

This document outlines the roadmap for achieving full compatibility with **Elasticsearch 7.x** (specifically targeting **7.17.x**, the latest 7.x version). This builds upon the ES 6.x compatibility work and addresses the breaking changes and new features introduced in ES 7.x.

## Current Status

**Base Compatibility:** ~85% of ES 6.8.23 API (see [ES6_COMPATIBILITY_ROADMAP.md](ES6_COMPATIBILITY_ROADMAP.md))

**ES 7.x Compatibility:** ~75% (most ES 6.x features work, but ES 7.x specific changes needed)

## Key Differences: ES 6.x → ES 7.x

### Breaking Changes

1. **Mapping Types Removed** ⚠️ **CRITICAL**
   - ES 6.x: Multiple mapping types per index (deprecated)
   - ES 7.x: Only `_doc` type allowed, type parameter removed from APIs
   - **Impact**: All APIs must use `_doc` only, remove type from URLs
   - **Status**: ✅ Already using `_doc` (ES 6.x compatible)

2. **Default Shard Count Changed**
   - ES 6.x: Default 5 shards
   - ES 7.x: Default 1 shard
   - **Impact**: New indices default to 1 shard
   - **Status**: ✅ Already defaults to 1 shard

3. **String Field Type Removed**
   - ES 6.x: `string` type (deprecated)
   - ES 7.x: Must use `text` or `keyword`
   - **Impact**: Mapping validation needed
   - **Status**: ⚠️ Need to add validation

4. **Total Hits Format Changed**
   - ES 6.x: `"total": 100` (number)
   - ES 7.x: `"total": {"value": 100, "relation": "eq"}` (object)
   - **Impact**: Search response format
   - **Status**: ✅ Already using ES 7.x format

5. **Cluster State Format Changes**
   - ES 7.x: Simplified cluster state API
   - **Impact**: Cluster endpoints may need updates
   - **Status**: ⚠️ Need to verify

### New Features in ES 7.x

1. **REST API Compatibility Mode**
   - Allows ES 7.x to accept ES 6.x format requests
   - Header: `Accept: application/vnd.elasticsearch+json;compatible-with=6`
   - **Impact**: Can support both formats
   - **Status**: ❌ Not implemented

2. **Index Lifecycle Management (ILM)**
   - Automated index lifecycle policies
   - **Impact**: New endpoints for ILM
   - **Status**: ❌ Not implemented

3. **Data Streams**
   - Time-series data management
   - **Impact**: New data stream APIs
   - **Status**: ❌ Not implemented

4. **Frozen Indices**
   - Read-only indices for cold storage
   - **Impact**: New index states
   - **Status**: ❌ Not implemented

5. **Searchable Snapshots**
   - Search directly from snapshots
   - **Impact**: Advanced feature
   - **Status**: ❌ Not implemented (enterprise feature)

6. **CCR (Cross-Cluster Replication)**
   - Replicate indices across clusters
   - **Impact**: Advanced feature
   - **Status**: ❌ Not implemented (enterprise feature)

## Migration Path: ES 6.x → ES 7.x

### Phase 0: Prerequisites (Complete ES 6.x First)
**Status**: In Progress (~85% complete)

Before targeting ES 7.x, we should:
1. ✅ Complete ES 6.x core features
2. ⚠️ Implement inverted index (performance)
3. ⚠️ Implement text analysis pipeline
4. ⚠️ Add aggregations

**Estimated Time**: 4-6 weeks (from current state)

### Phase 1: Breaking Changes (1-2 weeks)
**Goal**: Make codebase ES 7.x compatible

#### 1.1. Remove Type Parameters (1 day)
- ✅ Already using `_doc` only
- ⚠️ Verify all endpoints don't accept type parameter
- ⚠️ Update error messages if type is provided
- **Files to update**: All handlers, routes

#### 1.2. String Type Validation (2-3 days)
- Add validation to reject `string` type in mappings
- Suggest `text` or `keyword` instead
- Update mapping update endpoints
- **Files to update**: `src/storage/index_ops.rs`, mapping validation

#### 1.3. Response Format Verification (1 day)
- Verify search responses use ES 7.x format
- ✅ Already using `{"value": N, "relation": "eq"}`
- Verify cluster responses match ES 7.x
- **Files to update**: `src/storage/search/`, `src/storage/stats.rs`

#### 1.4. Default Shard Count (Already Done)
- ✅ Already defaults to 1 shard
- No changes needed

### Phase 2: ES 7.x New Features (4-6 weeks)
**Goal**: Add ES 7.x specific features

#### 2.1. REST API Compatibility Mode (1 week)
**Priority**: Medium
- Implement compatibility header parsing
- Support ES 6.x format requests with compatibility header
- Return ES 6.x format responses when requested
- **Impact**: Allows clients to use ES 6.x format
- **Complexity**: Medium
- **Files**: New middleware, response formatting

#### 2.2. Index Lifecycle Management (2-3 weeks)
**Priority**: Medium
- POST /_ilm/policy/{name} - Create ILM policy
- GET /_ilm/policy/{name} - Get ILM policy
- DELETE /_ilm/policy/{name} - Delete ILM policy
- POST /{index}/_ilm/retry - Retry failed step
- **Impact**: Useful for automated index management
- **Complexity**: Medium-High
- **Files**: New `src/server/handlers/ilm.rs`, `src/storage/ilm.rs`

#### 2.3. Data Streams (2-3 weeks)
**Priority**: Low (time-series specific)
- POST /_data_stream/{name} - Create data stream
- GET /_data_stream/{name} - Get data stream
- DELETE /_data_stream/{name} - Delete data stream
- **Impact**: Important for time-series use cases
- **Complexity**: High
- **Files**: New `src/storage/data_streams.rs`

#### 2.4. Frozen Indices (1 week)
**Priority**: Low
- POST /{index}/_freeze - Freeze index
- POST /{index}/_unfreeze - Unfreeze index
- **Impact**: Useful for cold storage
- **Complexity**: Low-Medium
- **Files**: Update `src/storage/index_ops.rs`

### Phase 3: Enhanced Features (3-4 weeks)
**Goal**: Improve existing features to ES 7.x standards

#### 3.1. Enhanced Aggregations (if not done in ES 6.x)
**Priority**: High
- Complete aggregation implementation
- Support all ES 7.x aggregation types
- **Impact**: Critical for analytics
- **Complexity**: High
- **Estimated**: 2-3 weeks

#### 3.2. Improved Search Features
**Priority**: Medium
- Enhanced query types from ES 7.x
- Better scoring (BM25 improvements in ES 7.x)
- **Impact**: Better search quality
- **Complexity**: Medium
- **Estimated**: 1-2 weeks

#### 3.3. Security Features (Optional)
**Priority**: Low (unless needed)
- Basic authentication
- API key management
- **Impact**: Production security
- **Complexity**: High
- **Estimated**: 2-3 weeks

### Phase 4: Advanced Features (Optional, 4-6 weeks)
**Goal**: Enterprise-level features

#### 4.1. Searchable Snapshots
**Priority**: Very Low
- Enterprise feature
- Complex implementation
- **Recommendation**: Skip unless specifically needed

#### 4.2. Cross-Cluster Replication (CCR)
**Priority**: Very Low
- Enterprise feature
- Requires multi-cluster support
- **Recommendation**: Skip unless specifically needed

## Detailed Feature Breakdown

### ✅ Already ES 7.x Compatible

1. **Document Type**: Using `_doc` only ✅
2. **Default Shards**: Defaulting to 1 ✅
3. **Total Hits Format**: Using ES 7.x format ✅
4. **Core APIs**: Most endpoints compatible ✅

### ⚠️ Needs Updates for ES 7.x

1. **String Type Validation**
   - Reject `string` type in mappings
   - Suggest `text` or `keyword`
   - **Effort**: 2-3 days

2. **Type Parameter Handling**
   - Reject type parameter in URLs (except `_doc`)
   - Return appropriate errors
   - **Effort**: 1 day

3. **Response Format Verification**
   - Ensure all responses match ES 7.x exactly
   - Verify cluster state format
   - **Effort**: 1-2 days

### ❌ New Features to Add

1. **REST API Compatibility Mode** (1 week)
   - Parse compatibility headers
   - Format responses based on compatibility mode
   - Support both ES 6.x and ES 7.x formats

2. **Index Lifecycle Management** (2-3 weeks)
   - Policy management
   - Index lifecycle states
   - Automated transitions

3. **Data Streams** (2-3 weeks)
   - Time-series data management
   - Automatic index creation
   - Backing indices management

4. **Frozen Indices** (1 week)
   - Index state management
   - Read-only frozen state

## Implementation Priority

### Critical (Must Have for ES 7.x)
1. **String Type Validation** - Reject deprecated types
2. **Type Parameter Validation** - Ensure `_doc` only
3. **Response Format Verification** - Match ES 7.x exactly

### Important (Should Have)
4. **REST API Compatibility Mode** - Support ES 6.x clients
5. **Index Lifecycle Management** - Useful feature
6. **Enhanced Aggregations** - If not done in ES 6.x phase

### Optional (Nice to Have)
7. **Data Streams** - Time-series specific
8. **Frozen Indices** - Cold storage
9. **Security Features** - If needed for production

### Skip (Enterprise Features)
10. **Searchable Snapshots** - Too complex, enterprise-only
11. **Cross-Cluster Replication** - Too complex, enterprise-only

## Recommended Roadmap

### Short-term (1-2 weeks)
**Goal**: ES 7.x Breaking Changes Compliance

1. String type validation
2. Type parameter validation
3. Response format verification
4. Testing against ES 7.17.x

**Result**: 100% ES 7.x breaking changes compliance

### Medium-term (4-6 weeks)
**Goal**: ES 7.x Feature Parity

1. REST API compatibility mode
2. Index Lifecycle Management
3. Enhanced aggregations (if needed)
4. Frozen indices support

**Result**: ~90% ES 7.x feature compatibility

### Long-term (8-12 weeks)
**Goal**: Full ES 7.x Compatibility

1. Data streams
2. Advanced features
3. Performance optimizations
4. Comprehensive testing

**Result**: ~95% ES 7.x compatibility

## Testing Strategy

### Compatibility Tests
1. **API Format Tests**
   - Test all endpoints against ES 7.17.x
   - Verify request/response formats match
   - Test error responses

2. **Breaking Change Tests**
   - Test type parameter rejection
   - Test string type rejection
   - Test default shard count

3. **Feature Tests**
   - Test ILM policies
   - Test data streams (if implemented)
   - Test frozen indices

4. **Client Compatibility Tests**
   - Test with ES 7.x clients
   - Test with ES 6.x clients (compatibility mode)
   - Test with Laravel Scout (ES 7.x)

## Migration Checklist

### Pre-Migration
- [ ] Complete ES 6.x compatibility (Phase 0)
- [ ] Review ES 7.x breaking changes
- [ ] Plan migration strategy
- [ ] Set up ES 7.17.x test environment

### Migration Steps
- [ ] Update string type validation
- [ ] Update type parameter handling
- [ ] Verify response formats
- [ ] Add REST API compatibility mode
- [ ] Implement ILM (optional)
- [ ] Test with ES 7.x clients

### Post-Migration
- [ ] Update documentation
- [ ] Update version number in code
- [ ] Update configuration defaults
- [ ] Test with real-world clients
- [ ] Performance testing

## Version Configuration

### Current Configuration
```yaml
# gummy-search.yaml
es_version: "6.8.23"  # Current default
```

### ES 7.x Configuration
```yaml
# gummy-search.yaml
es_version: "7.17.15"  # ES 7.x default
```

### Dual Compatibility
Consider supporting both versions:
- Default to ES 7.x
- Support ES 6.x via compatibility mode
- Allow version selection via config

## Estimated Timeline

### Minimum ES 7.x Compliance
**Time**: 1-2 weeks
**Compatibility**: 100% breaking changes compliance
**Status**: Production-ready for ES 7.x clients

### Full ES 7.x Features
**Time**: 4-6 weeks
**Compatibility**: ~90% ES 7.x features
**Status**: Most ES 7.x use cases supported

### Complete ES 7.x Compatibility
**Time**: 8-12 weeks
**Compatibility**: ~95% ES 7.x compatibility
**Status**: Near-complete ES 7.x compatibility

## Comparison: ES 6.x vs ES 7.x

| Feature | ES 6.x | ES 7.x | Status |
|---------|--------|--------|--------|
| Document Type | `_doc` (deprecated others) | `_doc` only | ✅ Compatible |
| Default Shards | 5 | 1 | ✅ Compatible |
| String Type | Deprecated | Removed | ⚠️ Need validation |
| Total Hits | Number | Object | ✅ Compatible |
| ILM | ❌ | ✅ | ❌ Not implemented |
| Data Streams | ❌ | ✅ | ❌ Not implemented |
| Frozen Indices | ❌ | ✅ | ❌ Not implemented |
| Compatibility Mode | ❌ | ✅ | ❌ Not implemented |

## Recommendations

### For Immediate ES 7.x Support
Focus on **Phase 1** (Breaking Changes):
- String type validation
- Type parameter validation
- Response format verification
- **Time**: 1-2 weeks
- **Result**: 100% breaking changes compliance

### For Full ES 7.x Compatibility
Complete **Phases 1-2**:
- Breaking changes compliance
- REST API compatibility mode
- Index Lifecycle Management
- **Time**: 4-6 weeks
- **Result**: ~90% ES 7.x compatibility

### For Complete ES 7.x Support
Complete all phases:
- All breaking changes
- All new features
- Advanced features
- **Time**: 8-12 weeks
- **Result**: ~95% ES 7.x compatibility

## Next Steps

1. **Immediate**: Complete ES 6.x compatibility first (recommended)
2. **Short-term**: Implement ES 7.x breaking changes (1-2 weeks)
3. **Medium-term**: Add ES 7.x new features (4-6 weeks)
4. **Long-term**: Advanced features and optimizations

## Notes

- ES 7.x is largely backward compatible with ES 6.x APIs
- Most ES 6.x code will work with ES 7.x
- Main work is adding new features and ensuring format compliance
- Consider supporting both ES 6.x and ES 7.x via compatibility mode
- ES 7.x is the last major version before ES 8.x (which has more breaking changes)

## References

- [Elasticsearch 7.0 Release Notes](https://www.elastic.co/guide/en/elasticsearch/reference/7.0/release-notes-7.0.0.html)
- [Elasticsearch 7.17 Release Notes](https://www.elastic.co/guide/en/elasticsearch/reference/7.17/release-notes-7.17.0.html)
- [Breaking Changes in 7.0](https://www.elastic.co/guide/en/elasticsearch/reference/7.0/breaking-changes-7.0.html)
- [Migration Guide: 6.x to 7.x](https://www.elastic.co/guide/en/elasticsearch/reference/7.17/migrating-7.0.html)
