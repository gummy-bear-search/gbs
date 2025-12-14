# Gummy Search API Routes

This document provides a comprehensive overview of all API routes in Gummy Search, organized by category.

## Base URL

All routes are relative to the base URL (default: `http://localhost:9200`).

## Route Categories

- [Web Interface](#web-interface)
- [Cluster Management](#cluster-management)
- [Index Management](#index-management)
- [Document Operations](#document-operations)
- [Search Operations](#search-operations)
- [Bulk Operations](#bulk-operations)
- [Index Refresh](#index-refresh)
- [WebSocket](#websocket)

---

## Web Interface

### Root
- **Method:** `GET`
- **Path:** `/`
- **Handler:** `root()`
- **Description:** Returns a simple text message identifying the service
- **Response:** `200 OK` with plain text: "Gummy Search - Elasticsearch-compatible search engine"

### Web Dashboard
- **Method:** `GET`
- **Path:** `/web` or `/web/`
- **Handler:** `web_index()`
- **Description:** Serves the web dashboard HTML interface
- **Response:** `200 OK` with HTML content from `static/index.html`

### Static Assets
- **Method:** `GET`
- **Path:** `/static/*`
- **Handler:** Static file server
- **Description:** Serves static assets (CSS, JS, images, favicons)
- **Response:** Static file content

---

## Cluster Management

### Cluster Health
- **Method:** `GET`
- **Path:** `/_cluster/health`
- **Handler:** `handlers::cluster_health()`
- **Description:** Returns cluster health status
- **Response:** JSON with cluster status, node counts, shard information
- **Example:**
  ```json
  {
    "status": "green",
    "number_of_nodes": 1,
    "number_of_data_nodes": 1,
    "active_primary_shards": 0,
    "active_shards": 0
  }
  ```

### Cluster Statistics
- **Method:** `GET`
- **Path:** `/_cluster/stats`
- **Handler:** `handlers::cluster_stats()`
- **Description:** Returns comprehensive cluster statistics
- **Response:** JSON with cluster, indices, nodes, and system statistics

### List Indices (Cat API)
- **Method:** `GET`
- **Path:** `/_cat/indices`
- **Handler:** `handlers::cat_indices()`
- **Query Parameters:**
  - `v` - Verbose mode (includes header row)
- **Description:** Returns a list of all indices in cat format
- **Response:** Plain text (simple list) or formatted table with headers (verbose mode)

### Get Aliases
- **Method:** `GET`
- **Path:** `/_aliases`
- **Handler:** `handlers::get_aliases()`
- **Description:** Returns all index aliases
- **Response:** JSON object mapping index names to their aliases

---

## Index Management

### Create Index
- **Method:** `PUT`
- **Path:** `/{index}`
- **Handler:** `handlers::create_index()`
- **Description:** Creates a new index with optional settings and mappings
- **Request Body:** (optional) JSON with `settings` and/or `mappings`
- **Response:** `200 OK` on success
- **Errors:**
  - `400 Bad Request` - Index already exists

### Check Index Existence
- **Method:** `HEAD`
- **Path:** `/{index}`
- **Handler:** `handlers::check_index()`
- **Description:** Checks if an index exists
- **Response:**
  - `200 OK` - Index exists
  - `404 Not Found` - Index does not exist

### Get Index Information
- **Method:** `GET`
- **Path:** `/{index}`
- **Handler:** `handlers::get_index()`
- **Description:** Returns index settings, mappings, and aliases
- **Response:** JSON with index configuration
- **Errors:**
  - `404 Not Found` - Index does not exist

### Delete Index
- **Method:** `DELETE`
- **Path:** `/{index}`
- **Handler:** `handlers::delete_index()`
- **Description:** Deletes an index
- **Special:** Supports `DELETE /_all` to delete all indices (dangerous operation)
- **Response:** `200 OK` on success
- **Errors:**
  - `404 Not Found` - Index does not exist

### Update Index Mapping
- **Method:** `PUT`
- **Path:** `/{index}/_mapping`
- **Handler:** `handlers::update_mapping()`
- **Description:** Updates or adds field mappings to an index
- **Request Body:** JSON with `properties` or `mappings.properties`
- **Response:** `200 OK` on success
- **Errors:**
  - `400 Bad Request` - Missing properties in request body
  - `404 Not Found` - Index does not exist

### Update Index Settings
- **Method:** `PUT`
- **Path:** `/{index}/_settings`
- **Handler:** `handlers::update_settings()`
- **Description:** Updates index settings (analysis, shards, replicas, etc.)
- **Request Body:** JSON with settings to update
- **Response:** `200 OK` on success
- **Errors:**
  - `404 Not Found` - Index does not exist

---

## Document Operations

### Index Document (Create or Update)
- **Method:** `PUT`
- **Path:** `/{index}/_doc/{id}`
- **Handler:** `handlers::index_document()`
- **Description:** Creates or updates a document with a specific ID
- **Request Body:** JSON document
- **Response:** `201 Created` on success
- **Errors:**
  - `404 Not Found` - Index does not exist

### Create Document (Auto-Generated ID)
- **Method:** `POST`
- **Path:** `/{index}/_doc`
- **Handler:** `handlers::create_document()`
- **Description:** Creates a document with an auto-generated ID
- **Request Body:** JSON document
- **Response:** `200 OK` with JSON containing `_id`, `_index`, `_type`, `_version`, `result`
- **Errors:**
  - `404 Not Found` - Index does not exist

### Get Document
- **Method:** `GET`
- **Path:** `/{index}/_doc/{id}`
- **Handler:** `handlers::get_document()`
- **Description:** Retrieves a document by ID
- **Response:** JSON with `_index`, `_type`, `_id`, `_version`, `_source`
- **Errors:**
  - `404 Not Found` - Index or document does not exist

### Delete Document
- **Method:** `DELETE`
- **Path:** `/{index}/_doc/{id}`
- **Handler:** `handlers::delete_document()`
- **Description:** Deletes a document by ID
- **Response:** `200 OK` on success
- **Errors:**
  - `404 Not Found` - Index or document does not exist

---

## Search Operations

### Search (GET)
- **Method:** `GET`
- **Path:** `/{index}/_search`
- **Handler:** `handlers::search_get()`
- **Description:** Performs a search using query parameters
- **Query Parameters:**
  - `q` - Query string (searches in all fields)
  - `from` - Pagination offset (default: 0)
  - `size` - Number of results (default: 10)
- **Response:** JSON with search results
- **Example:** `GET /my_index/_search?q=hello&from=0&size=10`

### Search (POST)
- **Method:** `POST`
- **Path:** `/{index}/_search`
- **Handler:** `handlers::search_post()`
- **Description:** Performs a search with a JSON query body
- **Request Body:** JSON with query DSL
- **Supported Query Types:**
  - `match` - Text search in a field
  - `match_all` - Return all documents
  - `match_phrase` - Phrase search
  - `multi_match` - Search across multiple fields
  - `term` - Exact term match
  - `terms` - Match any of the terms
  - `range` - Range queries (gt, gte, lt, lte)
  - `wildcard` - Wildcard pattern matching
  - `prefix` - Prefix matching
  - `bool` - Boolean query (must, should, must_not, filter)
- **Request Body Options:**
  - `query` - Query DSL object
  - `from` - Pagination offset
  - `size` - Number of results
  - `sort` - Sort specification
  - `_source` - Source filtering
  - `highlight` - Highlighting configuration
- **Response:** JSON with search results including hits, total, max_score

### Multi-Index Search
- **Method:** `POST`
- **Path:** `/_search`
- **Handler:** `handlers::search_multi_index()`
- **Description:** Searches across multiple indices
- **Request Body:** JSON with query DSL and optional `indices` array
- **Features:**
  - Supports wildcard index patterns (`*`, `?`)
  - Combines results from multiple indices
  - Sorts results by score across all indices
  - Applies pagination to combined results
- **Response:** JSON with combined search results

---

## Bulk Operations

### Bulk Operations (Index-Specific)
- **Method:** `POST`
- **Path:** `/{index}/_bulk`
- **Handler:** `handlers::bulk_operations()`
- **Description:** Performs bulk operations on documents in a specific index
- **Query Parameters:**
  - `refresh` - Refresh mode: `true`, `wait_for`, or `false` (default: `false`)
- **Request Body:** Newline-delimited JSON (NDJSON)
- **Supported Actions:**
  - `index` - Create or update document
  - `create` - Create document (fails if exists)
  - `update` - Update document (merges with existing)
  - `delete` - Delete document
- **Response:** JSON with results for each action
- **Format:** Each action requires two lines:
  1. Action metadata: `{"index": {"_index": "my_index", "_id": "1"}}`
  2. Document (for index/create/update): `{"field": "value"}`

### Bulk Operations (Multi-Index)
- **Method:** `POST`
- **Path:** `/_bulk`
- **Handler:** `handlers::bulk_operations()`
- **Description:** Performs bulk operations across any indices (index must be specified in action metadata)
- **Query Parameters:**
  - `refresh` - Refresh mode: `true`, `wait_for`, or `false` (default: `false`)
- **Request Body:** Newline-delimited JSON (NDJSON)
- **Response:** JSON with results for each action

---

## Index Refresh

### Refresh Index
- **Method:** `POST`
- **Path:** `/{index}/_refresh`
- **Handler:** `handlers::refresh_index()`
- **Description:** Refreshes an index (makes changes immediately visible)
- **Note:** For in-memory storage, this is a no-op as changes are immediately visible
- **Response:** `200 OK`

### Refresh All Indices
- **Method:** `POST`
- **Path:** `/_refresh`
- **Handler:** `handlers::refresh_all()`
- **Description:** Refreshes all indices
- **Note:** For in-memory storage, this is a no-op as changes are immediately visible
- **Response:** `200 OK`

---

## WebSocket

### WebSocket Connection
- **Method:** `GET`
- **Path:** `/_ws`
- **Handler:** `handlers::websocket_handler()`
- **Description:** Establishes a WebSocket connection for real-time updates
- **Protocol:** WebSocket (upgrades from HTTP)
- **Message Types:**
  - `cluster_health` - Cluster health status
  - `cluster_stats` - Cluster statistics
  - `indices` - List of indices with document counts
- **Update Frequency:** Every 30 seconds
- **Use Case:** Real-time dashboard updates

---

## Route Summary Table

| Method | Path | Handler | Category |
|--------|------|---------|----------|
| GET | `/` | `root()` | Web Interface |
| GET | `/web` | `web_index()` | Web Interface |
| GET | `/web/` | `web_index()` | Web Interface |
| GET | `/static/*` | Static server | Web Interface |
| GET | `/_cluster/health` | `cluster_health()` | Cluster |
| GET | `/_cluster/stats` | `cluster_stats()` | Cluster |
| GET | `/_cat/indices` | `cat_indices()` | Cluster |
| GET | `/_aliases` | `get_aliases()` | Cluster |
| PUT | `/{index}` | `create_index()` | Index |
| HEAD | `/{index}` | `check_index()` | Index |
| GET | `/{index}` | `get_index()` | Index |
| DELETE | `/{index}` | `delete_index()` | Index |
| PUT | `/{index}/_mapping` | `update_mapping()` | Index |
| PUT | `/{index}/_settings` | `update_settings()` | Index |
| PUT | `/{index}/_doc/{id}` | `index_document()` | Document |
| GET | `/{index}/_doc/{id}` | `get_document()` | Document |
| DELETE | `/{index}/_doc/{id}` | `delete_document()` | Document |
| POST | `/{index}/_doc` | `create_document()` | Document |
| POST | `/{index}/_bulk` | `bulk_operations()` | Bulk |
| POST | `/_bulk` | `bulk_operations()` | Bulk |
| GET | `/{index}/_search` | `search_get()` | Search |
| POST | `/{index}/_search` | `search_post()` | Search |
| POST | `/_search` | `search_multi_index()` | Search |
| POST | `/{index}/_refresh` | `refresh_index()` | Refresh |
| POST | `/_refresh` | `refresh_all()` | Refresh |
| GET | `/_ws` | `websocket_handler()` | WebSocket |

---

## Notes

- All routes support CORS (permissive mode)
- All routes have HTTP tracing enabled
- Path parameters:
  - `{index}` - Index name
  - `{id}` - Document ID
- Query parameters are case-sensitive
- JSON request/response bodies follow Elasticsearch 6.8.23 API format
- Error responses follow Elasticsearch error format for compatibility
