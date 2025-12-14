# Elasticsearch API Requirements for Rust Port

## Overview

This document describes the minimum set of Elasticsearch API endpoints that need to be implemented in the Rust port to ensure compatibility with Elasticsearch 6.8.23 API.

## Elasticsearch Version

The project uses **Elasticsearch 6.8.23**, so all APIs must be compatible with this version.

## Required API Endpoints

### 1. Index Management

#### 1.1. Create Index
**Endpoint:** `PUT /{index}`

**Description:** Creates a new index with specified settings and mappings.

**Usage in Project:**
- `elastic:create-index` command uses this endpoint
- Creates indices for: Content, Entity, File, Dictionary, EsedoDictionary

**Example Request:**
```json
PUT /content_index
{
  "settings": {
    "number_of_shards": 1,
    "number_of_replicas": 0,
    "analysis": {
      "analyzer": {
        "content_analyzer": {
          "type": "custom",
          "tokenizer": "standard",
          "filter": ["lowercase", "russian_stemmer"]
        }
      }
    }
  },
  "mappings": {
    "properties": {
      "title": {
        "type": "text",
        "analyzer": "content_analyzer"
      },
      "body": {
        "type": "text",
        "analyzer": "content_analyzer"
      },
      "id": {
        "type": "keyword"
      }
    }
  }
}
```

**Must Implement:**
- Support for `settings` (number_of_shards, number_of_replicas, analysis)
- Support for `mappings` (properties, field types, analyzers)
- Index name validation
- Error handling (index already exists)

#### 1.2. Check Index Existence
**Endpoint:** `HEAD /{index}`

**Description:** Checks if an index exists.

**Usage in Project:**
- Used before creating an index for verification
- Used in index update logic

**Must Implement:**
- Return status 200 (exists) or 404 (does not exist)

#### 1.3. Get Index Information
**Endpoint:** `GET /{index}`

**Description:** Returns index information (settings, mappings).

**Usage in Project:**
- Used to check current configuration
- Used before updating mappings

**Must Implement:**
- Return settings and mappings
- Handle non-existent index

#### 1.4. Update Index Mapping
**Endpoint:** `PUT /{index}/_mapping`

**Description:** Updates the mapping of an existing index.

**Usage in Project:**
- `elastic:update-index` command uses this endpoint
- Used for document-service and explorer-service

**Example Request:**
```json
PUT /entity_index/_mapping
{
  "properties": {
    "new_field": {
      "type": "text"
    }
  }
}
```

**Must Implement:**
- Update existing mappings
- Add new fields
- Validate compatibility of changes

#### 1.5. Update Index Settings
**Endpoint:** `PUT /{index}/_settings`

**Description:** Updates index settings.

**Usage in Project:**
- Used to change analyzers and other settings
- May be used in `elastic:update-index`

**Example Request:**
```json
PUT /entity_index/_settings
{
  "analysis": {
    "analyzer": {
      "custom_analyzer": {
        "type": "custom",
        "tokenizer": "standard"
      }
    }
  }
}
```

**Must Implement:**
- Update analysis settings
- Update other dynamic settings
- Validate changes

#### 1.6. Delete Index
**Endpoint:** `DELETE /{index}`

**Description:** Deletes an index and all its data.

**Usage in Project:**
- `make elastic-clean` command uses `DELETE /_all`
- Used during reindexing

**Must Implement:**
- Delete a single index
- Support `DELETE /_all` to delete all indices (⚠️ dangerous operation)
- Confirmation of deletion

### 2. Document Operations

#### 2.1. Index Document (Create/Update)
**Endpoint:** `PUT /{index}/_doc/{id}` or `POST /{index}/_doc`

**Description:** Creates or updates a document in an index.

**Usage in Project:**
- Laravel Scout uses this endpoint for automatic indexing
- Used when creating/updating models (Content, Entity, File, Dictionary)

**Example Request:**
```json
PUT /content_index/_doc/123
{
  "id": 123,
  "title": "Document Title",
  "body": "Document Content",
  "created_at": "2024-01-01T00:00:00Z"
}
```

**Must Implement:**
- Create document with specified ID
- Automatic ID generation when using POST
- Update existing document
- Document data validation

#### 2.2. Get Document
**Endpoint:** `GET /{index}/_doc/{id}`

**Description:** Retrieves a document by ID.

**Usage in Project:**
- Used by Laravel Scout to retrieve documents
- Used to check document existence

**Must Implement:**
- Return document by ID
- Handle non-existent document (404)
- Support `_source` filtering

#### 2.3. Delete Document
**Endpoint:** `DELETE /{index}/_doc/{id}`

**Description:** Deletes a document from an index.

**Usage in Project:**
- Laravel Scout uses when deleting models
- `scout:flush` command uses to clear all documents of a model

**Must Implement:**
- Delete document by ID
- Handle non-existent document
- Return operation status

#### 2.4. Bulk Operations
**Endpoint:** `POST /_bulk` or `POST /{index}/_bulk`

**Description:** Performs multiple operations (indexing, updating, deleting) in a single request.

**Usage in Project:**
- `scout:import` command uses bulk API for mass indexing
- Critically important for performance when importing large volumes of data

**Example Request:**
```
POST /content_index/_bulk
{"index":{"_id":"1"}}
{"title":"Document 1","body":"Content 1"}
{"index":{"_id":"2"}}
{"title":"Document 2","body":"Content 2"}
{"delete":{"_id":"3"}}
{"update":{"_id":"4"}}
{"doc":{"title":"Updated Title"}}
```

**Must Implement:**
- Support operations: `index`, `create`, `update`, `delete`
- Handle NDJSON format
- Return results for each operation
- Handle partial errors (some operations may fail)
- Support `refresh` parameter

**Criticality:** ⭐⭐⭐⭐⭐ (high) - used for mass data import

### 3. Search

#### 3.1. Search Documents
**Endpoint:** `GET /{index}/_search` or `POST /{index}/_search`

**Description:** Performs document search by query.

**Usage in Project:**
- Laravel Scout uses for search: `Model::search('query')->get()`
- Main search functionality in the application

**Example Request:**
```json
POST /content_index/_search
{
  "query": {
    "match": {
      "title": "contract"
    }
  },
  "from": 0,
  "size": 10,
  "sort": [
    {
      "_score": {
        "order": "desc"
      }
    }
  ]
}
```

**Must Implement:**
- Support various query types:
  - `match` - full-text search
  - `match_phrase` - phrase search
  - `multi_match` - search across multiple fields
  - `term` - exact match
  - `terms` - match one of values
  - `range` - value range
  - `bool` - boolean queries (must, should, must_not, filter)
  - `wildcard` - wildcard search
  - `prefix` - prefix search
- Support filters (`filter`)
- Support sorting (`sort`)
- Support pagination (`from`, `size`)
- Support result highlighting (`highlight`)
- Support aggregations (`aggs`) - optional
- Return `_score` for relevance
- Support `_source` filtering

**Criticality:** ⭐⭐⭐⭐⭐ (high) - main search functionality

#### 3.2. Multi-Index Search
**Endpoint:** `GET /_search` or `POST /_search`

**Description:** Search across all indices or multiple specified indices.

**Usage in Project:**
- May be used for global search
- Search across multiple content types simultaneously

**Must Implement:**
- Support search across multiple indices: `/_search` or `/index1,index2/_search`
- Support wildcards in index names: `/content*/_search`

### 4. Utilities and Information

#### 4.1. Cluster Health Check
**Endpoint:** `GET /_cluster/health`

**Description:** Returns cluster health information.

**Usage in Project:**
- Used for monitoring
- Check Elasticsearch availability

**Must Implement:**
- Return cluster status (green, yellow, red)
- Information about number of nodes
- Information about number of indices

#### 4.2. Cluster Information
**Endpoint:** `GET /_cluster/stats`

**Description:** Returns cluster statistics.

**Usage in Project:**
- Used for monitoring and debugging

**Must Implement:**
- Statistics by indices
- Statistics by nodes
- Memory usage

#### 4.3. List Indices
**Endpoint:** `GET /_cat/indices?v` or `GET /_aliases`

**Description:** Returns a list of all indices.

**Usage in Project:**
- Used for debugging and monitoring
- Check index existence

**Must Implement:**
- List all indices
- Information about each index (document count, size)

#### 4.4. Refresh Index
**Endpoint:** `POST /{index}/_refresh` or `POST /_refresh`

**Description:** Forces index refresh, making all changes visible for search.

**Usage in Project:**
- Used after bulk operations for immediate refresh
- May be used in `scout:import` after mass indexing

**Must Implement:**
- Refresh a single index
- Refresh all indices
- Support `refresh=true` in bulk operations

**Criticality:** ⭐⭐⭐⭐ (high) - important for data consistency

## Implementation Priorities

### Critically Important (MVP)
1. ✅ **PUT /{index}** - create index
2. ✅ **POST /{index}/_bulk** - bulk operations (for import)
3. ✅ **POST /{index}/_search** - search documents
4. ✅ **PUT /{index}/_doc/{id}** - index document
5. ✅ **DELETE /{index}/_doc/{id}** - delete document
6. ✅ **POST /{index}/_refresh** - refresh index

### Important (for full functionality)
7. ✅ **PUT /{index}/_mapping** - update mapping
8. ✅ **PUT /{index}/_settings** - update settings
9. ✅ **GET /{index}/_doc/{id}** - get document
10. ✅ **HEAD /{index}** - check index existence
11. ✅ **GET /{index}** - index information
12. ✅ **DELETE /{index}** - delete index

### Useful (for monitoring and debugging)
13. ✅ **GET /_cluster/health** - cluster health
14. ✅ **GET /_cat/indices** - list indices
15. ✅ **GET /_cluster/stats** - cluster statistics

## Implementation Details

### Data Format

#### NDJSON for Bulk API
Bulk API uses NDJSON (Newline Delimited JSON) format, where each line is a separate JSON object:
```
{"action": {"_id": "1"}}
{"field": "value"}
{"action": {"_id": "2"}}
{"field": "value2"}
```

### API Versioning

Elasticsearch 6.8.23 uses:
- Document type: `_doc` (instead of deprecated document type)
- Document endpoint: `/{index}/_doc/{id}`

### Error Handling

Must handle the following HTTP statuses:
- `200` - successful operation
- `201` - document created
- `404` - index or document not found
- `400` - invalid request
- `409` - conflict (e.g., document already exists)
- `500` - internal server error

### Response Format

#### Successful Search Response:
```json
{
  "took": 5,
  "timed_out": false,
  "_shards": {
    "total": 1,
    "successful": 1,
    "skipped": 0,
    "failed": 0
  },
  "hits": {
    "total": {
      "value": 100,
      "relation": "eq"
    },
    "max_score": 1.0,
    "hits": [
      {
        "_index": "content_index",
        "_type": "_doc",
        "_id": "123",
        "_score": 1.0,
        "_source": {
          "title": "Document",
          "body": "Content"
        }
      }
    ]
  }
}
```

#### Successful Bulk Operation Response:
```json
{
  "took": 100,
  "errors": false,
  "items": [
    {
      "index": {
        "_index": "content_index",
        "_type": "_doc",
        "_id": "1",
        "_version": 1,
        "result": "created",
        "_shards": {
          "total": 1,
          "successful": 1,
          "failed": 0
        },
        "status": 201
      }
    }
  ]
}
```

## Testing

### Test Scenarios

1. **Index Creation:**
   - Create index with settings and mappings
   - Verify index was created
   - Attempt to create existing index (should return error)

2. **Document Indexing:**
   - Create document with ID
   - Update existing document
   - Create document without ID (auto-generation)

3. **Bulk Operations:**
   - Index 1000 documents via bulk
   - Mixed operations (index, update, delete)
   - Handle partial errors

4. **Search:**
   - Simple match query
   - Bool query with filters
   - Sorting and pagination
   - Search across multiple indices

5. **Mapping Update:**
   - Add new field to mapping
   - Update analyzer

## Implementation Recommendations

### Rust Libraries

Recommended libraries for HTTP and JSON:
- `reqwest` - HTTP client
- `serde` + `serde_json` - JSON serialization/deserialization
- `tokio` - async runtime (if async needed)

### Project Structure

```
gummy-search/
├── src/
│   ├── client.rs          # Main client
│   ├── index.rs           # Index operations
│   ├── document.rs        # Document operations
│   ├── search.rs          # Search queries
│   ├── bulk.rs            # Bulk operations
│   ├── models.rs          # Data models
│   └── error.rs           # Error handling
├── tests/
│   └── integration.rs     # Integration tests
└── gummy_wiki/   # Laravel Scout integration example
```

### Example API Client

```rust
pub struct GummySearchClient {
    base_url: String,
    client: reqwest::Client,
}

impl GummySearchClient {
    pub async fn create_index(&self, index: &str, settings: IndexSettings) -> Result<()> {
        // PUT /{index}
    }

    pub async fn index_document(&self, index: &str, id: &str, doc: &Document) -> Result<()> {
        // PUT /{index}/_doc/{id}
    }

    pub async fn bulk(&self, operations: Vec<BulkOperation>) -> Result<BulkResponse> {
        // POST /_bulk
    }

    pub async fn search(&self, index: &str, query: SearchQuery) -> Result<SearchResponse> {
        // POST /{index}/_search
    }
}
```

## Conclusion

To create a limited Rust port of Elasticsearch, it is necessary to implement at least **6 critically important endpoints** for basic functionality and **9 additional** for full Elasticsearch 6.8.23 API compatibility.

Main focus should be on:
1. Index creation and management
2. Bulk operations for mass indexing
3. Search queries with support for main query types
4. Basic CRUD operations with documents
