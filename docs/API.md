# Gummy Search API Documentation

This document provides comprehensive API documentation for Gummy Search, an Elasticsearch-compatible search engine.

## Base URL

All API requests should be made to:
```
http://localhost:9200
```

## Authentication

Currently, Gummy Search does not require authentication. All endpoints are publicly accessible.

## Content Types

- **Request**: `application/json` (for JSON requests) or `application/x-ndjson` (for bulk operations)
- **Response**: `application/json`

## Common Response Fields

### Success Response
- Status code: `200`, `201`, or `204`
- Body: JSON object with operation results

### Error Response
- Status code: `400` (Bad Request), `404` (Not Found), `409` (Conflict), or `500` (Internal Server Error)
- Body: JSON object with error details

## Endpoints

### Index Management

#### Create Index
**Endpoint:** `PUT /{index}`

**Description:** Creates a new index with optional settings and mappings.

**Request Body:**
```json
{
  "settings": {
    "number_of_shards": 1,
    "number_of_replicas": 0
  },
  "mappings": {
    "properties": {
      "title": { "type": "text" },
      "count": { "type": "integer" }
    }
  }
}
```

**Response:**
- Status: `200 OK`
- Body: Empty

**Example:**
```bash
curl -X PUT "http://localhost:9200/my_index" -H 'Content-Type: application/json' -d'
{
  "settings": {
    "number_of_shards": 1
  },
  "mappings": {
    "properties": {
      "title": { "type": "text" }
    }
  }
}'
```

#### Check Index Existence
**Endpoint:** `HEAD /{index}`

**Description:** Checks if an index exists.

**Response:**
- Status: `200 OK` if exists, `404 Not Found` if not

**Example:**
```bash
curl -X HEAD "http://localhost:9200/my_index"
```

#### Get Index
**Endpoint:** `GET /{index}`

**Description:** Retrieves index information including settings and mappings.

**Response:**
```json
{
  "my_index": {
    "settings": { ... },
    "mappings": { ... },
    "aliases": {}
  }
}
```

**Example:**
```bash
curl -X GET "http://localhost:9200/my_index"
```

#### Delete Index
**Endpoint:** `DELETE /{index}`

**Description:** Deletes an index and all its documents.

**Response:**
- Status: `200 OK`

**Example:**
```bash
curl -X DELETE "http://localhost:9200/my_index"
```

#### Update Mapping
**Endpoint:** `PUT /{index}/_mapping`

**Description:** Updates or adds new field mappings to an existing index.

**Request Body:**
```json
{
  "properties": {
    "new_field": { "type": "text" }
  }
}
```

**Response:**
- Status: `200 OK`

**Example:**
```bash
curl -X PUT "http://localhost:9200/my_index/_mapping" -H 'Content-Type: application/json' -d'
{
  "properties": {
    "author": { "type": "keyword" }
  }
}'
```

#### Update Settings
**Endpoint:** `PUT /{index}/_settings`

**Description:** Updates index settings.

**Request Body:**
```json
{
  "number_of_shards": 2
}
```

**Response:**
- Status: `200 OK`

**Example:**
```bash
curl -X PUT "http://localhost:9200/my_index/_settings" -H 'Content-Type: application/json' -d'
{
  "number_of_shards": 2
}'
```

### Document Operations

#### Index Document (Create/Update)
**Endpoint:** `PUT /{index}/_doc/{id}`

**Description:** Creates or updates a document with a specific ID.

**Request Body:**
```json
{
  "title": "Example Document",
  "body": "Content here"
}
```

**Response:**
- Status: `200 OK` (updated) or `201 Created` (new)

**Example:**
```bash
curl -X PUT "http://localhost:9200/my_index/_doc/1" -H 'Content-Type: application/json' -d'
{
  "title": "My Document",
  "body": "Document content"
}'
```

#### Create Document (Auto ID)
**Endpoint:** `POST /{index}/_doc`

**Description:** Creates a document with an auto-generated ID.

**Request Body:**
```json
{
  "title": "New Document"
}
```

**Response:**
```json
{
  "_id": "generated-uuid-here"
}
```

**Example:**
```bash
curl -X POST "http://localhost:9200/my_index/_doc" -H 'Content-Type: application/json' -d'
{
  "title": "New Document"
}'
```

#### Get Document
**Endpoint:** `GET /{index}/_doc/{id}`

**Description:** Retrieves a document by ID.

**Response:**
```json
{
  "_index": "my_index",
  "_type": "_doc",
  "_id": "1",
  "_version": 1,
  "_source": {
    "title": "My Document",
    "body": "Content"
  }
}
```

**Example:**
```bash
curl -X GET "http://localhost:9200/my_index/_doc/1"
```

#### Delete Document
**Endpoint:** `DELETE /{index}/_doc/{id}`

**Description:** Deletes a document by ID.

**Response:**
- Status: `200 OK`

**Example:**
```bash
curl -X DELETE "http://localhost:9200/my_index/_doc/1"
```

### Bulk Operations

#### Bulk Operations
**Endpoint:** `POST /_bulk` or `POST /{index}/_bulk`

**Description:** Performs multiple document operations in a single request.

**Request Body:** NDJSON format (newline-delimited JSON)
```
{"index":{"_index":"my_index","_id":"1"}}
{"title":"Document 1"}
{"create":{"_index":"my_index","_id":"2"}}
{"title":"Document 2"}
{"update":{"_index":"my_index","_id":"1"}}
{"doc":{"title":"Updated Document 1"}}
{"delete":{"_index":"my_index","_id":"2"}}
```

**Response:**
```json
{
  "took": 10,
  "errors": false,
  "items": [
    {
      "index": {
        "_index": "my_index",
        "_id": "1",
        "_version": 1,
        "result": "created",
        "status": 201
      }
    }
  ]
}
```

**Query Parameters:**
- `refresh`: Control when changes are made visible
  - `false` (default): No refresh
  - `true`: Refresh after bulk operations
  - `wait_for`: Wait for refresh to complete

**Example:**
```bash
# Bulk operations without refresh
curl -X POST "http://localhost:9200/_bulk" -H 'Content-Type: application/x-ndjson' --data-binary @bulk_data.ndjson

# Bulk operations with refresh
curl -X POST "http://localhost:9200/_bulk?refresh=true" -H 'Content-Type: application/x-ndjson' --data-binary @bulk_data.ndjson
```

### Search

#### Search (POST)
**Endpoint:** `POST /{index}/_search`

**Description:** Searches documents in an index.

**Request Body:**
```json
{
  "query": {
    "match": {
      "title": "search term"
    }
  },
  "from": 0,
  "size": 10,
  "sort": [
    { "_score": { "order": "desc" } }
  ],
  "_source": ["title", "body"],
  "highlight": {
    "fields": {
      "title": {}
    }
  }
}
```

**Response:**
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
        "_index": "my_index",
        "_type": "_doc",
        "_id": "1",
        "_score": 1.0,
        "_source": {
          "title": "Document",
          "body": "Content"
        },
        "highlight": {
          "title": ["Document with <em>search</em> term"]
        }
      }
    ]
  }
}
```

**Query Types:**

1. **Match Query:**
```json
{
  "query": {
    "match": {
      "field": "search text"
    }
  }
}
```

2. **Match Phrase Query:**
```json
{
  "query": {
    "match_phrase": {
      "field": "exact phrase"
    }
  }
}
```

3. **Multi-Match Query:**
```json
{
  "query": {
    "multi_match": {
      "query": "search text",
      "fields": ["field1", "field2"]
    }
  }
}
```

4. **Term Query:**
```json
{
  "query": {
    "term": {
      "field": "exact value"
    }
  }
}
```

5. **Terms Query:**
```json
{
  "query": {
    "terms": {
      "field": ["value1", "value2"]
    }
  }
}
```

6. **Wildcard Query:**
```json
{
  "query": {
    "wildcard": {
      "field": "pat*ern"
    }
  }
}
```

7. **Prefix Query:**
```json
{
  "query": {
    "prefix": {
      "field": "prefix"
    }
  }
}
```

8. **Range Query:**
```json
{
  "query": {
    "range": {
      "field": {
        "gte": 10,
        "lte": 20
      }
    }
  }
}
```

9. **Bool Query:**
```json
{
  "query": {
    "bool": {
      "must": [
        { "match": { "title": "search" } }
      ],
      "should": [
        { "match": { "body": "content" } }
      ],
      "must_not": [
        { "term": { "status": "deleted" } }
      ],
      "filter": [
        { "range": { "date": { "gte": "2024-01-01" } } }
      ]
    }
  }
}
```

10. **Match All Query:**
```json
{
  "query": {
    "match_all": {}
  }
}
```

**Example:**
```bash
curl -X POST "http://localhost:9200/my_index/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "match": {
      "title": "example"
    }
  },
  "size": 10
}'
```

#### Search (GET)
**Endpoint:** `GET /{index}/_search`

**Description:** Same as POST but with query parameters.

**Query Parameters:**
- `q`: Query string (simple search)
- `from`: Starting offset (default: 0)
- `size`: Number of results (default: 10)

**Example:**
```bash
curl -X GET "http://localhost:9200/my_index/_search?q=example&size=10"
```

#### Multi-Index Search
**Endpoint:** `POST /_search`

**Description:** Searches across multiple indices or all indices.

**Request Body:**
```json
{
  "index": "logs-*",
  "query": {
    "match_all": {}
  }
}
```

**Index Patterns:**
- `"index": "logs-*"` - Wildcard pattern
- `"index": "index1,index2"` - Comma-separated list
- `"index": ["index1", "index2"]` - Array of indices
- Omit `index` field to search all indices

**Example:**
```bash
curl -X POST "http://localhost:9200/_search" -H 'Content-Type: application/json' -d'
{
  "index": "logs-2024-*",
  "query": {
    "match": {
      "message": "error"
    }
  }
}'
```

### Refresh

#### Refresh Index
**Endpoint:** `POST /{index}/_refresh`

**Description:** Refreshes an index (makes changes visible for search).

**Response:**
- Status: `200 OK`

**Example:**
```bash
curl -X POST "http://localhost:9200/my_index/_refresh"
```

#### Refresh All Indices
**Endpoint:** `POST /_refresh`

**Description:** Refreshes all indices.

**Response:**
- Status: `200 OK`

**Example:**
```bash
curl -X POST "http://localhost:9200/_refresh"
```

### Cluster Operations

#### Cluster Health
**Endpoint:** `GET /_cluster/health`

**Description:** Returns cluster health information.

**Response:**
```json
{
  "status": "green",
  "number_of_nodes": 1,
  "number_of_data_nodes": 1,
  "active_primary_shards": 0,
  "active_shards": 0,
  "relocating_shards": 0,
  "initializing_shards": 0,
  "unassigned_shards": 0
}
```

**Example:**
```bash
curl -X GET "http://localhost:9200/_cluster/health"
```

#### Cluster Stats
**Endpoint:** `GET /_cluster/stats`

**Description:** Returns cluster statistics.

**Response:**
```json
{
  "cluster_name": "gummy-search",
  "indices": {
    "count": 5,
    "docs": {
      "count": 1000
    }
  },
  "nodes": {
    "count": {
      "total": 1
    }
  }
}
```

**Example:**
```bash
curl -X GET "http://localhost:9200/_cluster/stats"
```

#### List Indices (Cat API)
**Endpoint:** `GET /_cat/indices?v`

**Description:** Lists all indices in a human-readable format.

**Query Parameters:**
- `v`: Verbose mode (shows headers)

**Response (text/plain):**
```
health status index     uuid pri rep docs.count store.size
green  open   my_index  -     1   0   100       0b
```

**Example:**
```bash
curl -X GET "http://localhost:9200/_cat/indices?v"
```

## Error Codes

- **200 OK**: Successful operation
- **201 Created**: Document created
- **204 No Content**: Successful operation with no content
- **400 Bad Request**: Invalid request (e.g., malformed JSON, invalid query)
- **404 Not Found**: Resource not found (index, document)
- **409 Conflict**: Conflict (e.g., document already exists)
- **500 Internal Server Error**: Server error

## Rate Limiting

Currently, Gummy Search does not implement rate limiting. All requests are processed immediately.

## Compatibility

Gummy Search is designed to be compatible with Elasticsearch 6.4.0 API. However, not all features are implemented. See the [README](../README.md) for a complete list of implemented features.

## Notes

- All timestamps are in milliseconds since Unix epoch
- Document IDs are case-sensitive
- Index names must be lowercase and cannot contain special characters
- Search is case-insensitive for text fields
- Highlighting uses `<em>` tags by default
