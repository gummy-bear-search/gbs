# Gummy Search Usage Examples

This document provides practical usage examples for common scenarios with Gummy Search.

## Table of Contents

1. [Basic Setup](#basic-setup)
2. [Index Management](#index-management)
3. [Document Operations](#document-operations)
4. [Search Examples](#search-examples)
5. [Bulk Operations](#bulk-operations)
6. [Real-World Scenarios](#real-world-scenarios)

## Basic Setup

### Starting the Server

```bash
# Using cargo
cargo run

# Using release build
cargo build --release
./target/release/gummy-search

# With custom configuration
GUMMY_PORT=9300 GUMMY_DATA_DIR=/custom/path cargo run
```

### Verifying Server is Running

```bash
curl http://localhost:9200/_cluster/health
```

Expected response:
```json
{
  "status": "green",
  "number_of_nodes": 1
}
```

## Index Management

### Creating an Index for Blog Posts

```bash
curl -X PUT "http://localhost:9200/blog_posts" -H 'Content-Type: application/json' -d'
{
  "settings": {
    "number_of_shards": 1,
    "number_of_replicas": 0
  },
  "mappings": {
    "properties": {
      "title": { "type": "text" },
      "content": { "type": "text" },
      "author": { "type": "keyword" },
      "published_at": { "type": "date" },
      "tags": { "type": "keyword" },
      "views": { "type": "integer" }
    }
  }
}'
```

### Creating an Index for E-commerce Products

```bash
curl -X PUT "http://localhost:9200/products" -H 'Content-Type: application/json' -d'
{
  "mappings": {
    "properties": {
      "name": { "type": "text" },
      "description": { "type": "text" },
      "price": { "type": "float" },
      "category": { "type": "keyword" },
      "in_stock": { "type": "boolean" },
      "sku": { "type": "keyword" }
    }
  }
}'
```

### Listing All Indices

```bash
# Simple list
curl -X GET "http://localhost:9200/_cat/indices"

# Verbose (with headers)
curl -X GET "http://localhost:9200/_cat/indices?v"
```

## Document Operations

### Indexing a Blog Post

```bash
curl -X PUT "http://localhost:9200/blog_posts/_doc/1" -H 'Content-Type: application/json' -d'
{
  "title": "Getting Started with Rust",
  "content": "Rust is a systems programming language...",
  "author": "John Doe",
  "published_at": "2024-01-15",
  "tags": ["rust", "programming", "tutorial"],
  "views": 150
}'
```

### Creating a Document with Auto-Generated ID

```bash
curl -X POST "http://localhost:9200/blog_posts/_doc" -H 'Content-Type: application/json' -d'
{
  "title": "Advanced Rust Patterns",
  "content": "This article covers advanced patterns...",
  "author": "Jane Smith",
  "published_at": "2024-01-20",
  "tags": ["rust", "advanced"],
  "views": 75
}'
```

### Retrieving a Document

```bash
curl -X GET "http://localhost:9200/blog_posts/_doc/1"
```

### Updating a Document

```bash
curl -X PUT "http://localhost:9200/blog_posts/_doc/1" -H 'Content-Type: application/json' -d'
{
  "title": "Getting Started with Rust (Updated)",
  "content": "Rust is a systems programming language...",
  "author": "John Doe",
  "published_at": "2024-01-15",
  "tags": ["rust", "programming", "tutorial", "beginner"],
  "views": 200
}'
```

### Deleting a Document

```bash
curl -X DELETE "http://localhost:9200/blog_posts/_doc/1"
```

## Search Examples

### Simple Text Search

```bash
curl -X POST "http://localhost:9200/blog_posts/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "match": {
      "title": "Rust"
    }
  }
}'
```

### Search with Highlighting

```bash
curl -X POST "http://localhost:9200/blog_posts/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "match": {
      "content": "programming"
    }
  },
  "highlight": {
    "fields": {
      "content": {}
    }
  }
}'
```

### Search with Pagination

```bash
curl -X POST "http://localhost:9200/blog_posts/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "match_all": {}
  },
  "from": 0,
  "size": 10
}'
```

### Search with Sorting

```bash
curl -X POST "http://localhost:9200/blog_posts/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "match_all": {}
  },
  "sort": [
    { "views": { "order": "desc" } }
  ]
}'
```

### Filter by Exact Value

```bash
curl -X POST "http://localhost:9200/blog_posts/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "term": {
      "author": "John Doe"
    }
  }
}'
```

### Filter by Multiple Values

```bash
curl -X POST "http://localhost:9200/blog_posts/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "terms": {
      "tags": ["rust", "tutorial"]
    }
  }
}'
```

### Range Query

```bash
curl -X POST "http://localhost:9200/blog_posts/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "range": {
      "views": {
        "gte": 100,
        "lte": 500
      }
    }
  }
}'
```

### Complex Bool Query

```bash
curl -X POST "http://localhost:9200/blog_posts/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "bool": {
      "must": [
        { "match": { "title": "Rust" } }
      ],
      "should": [
        { "match": { "tags": "tutorial" } }
      ],
      "must_not": [
        { "term": { "author": "Jane Smith" } }
      ],
      "filter": [
        { "range": { "views": { "gte": 50 } } }
      ]
    }
  }
}'
```

### Wildcard Search

```bash
curl -X POST "http://localhost:9200/blog_posts/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "wildcard": {
      "title": "*Rust*"
    }
  }
}'
```

### Prefix Search

```bash
curl -X POST "http://localhost:9200/blog_posts/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "prefix": {
      "title": "Getting"
    }
  }
}'
```

### Search with _source Filtering

```bash
curl -X POST "http://localhost:9200/blog_posts/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "match_all": {}
  },
  "_source": ["title", "author", "views"]
}'
```

### Multi-Index Search

```bash
curl -X POST "http://localhost:9200/_search" -H 'Content-Type: application/json' -d'
{
  "index": "blog_posts,products",
  "query": {
    "match": {
      "title": "Rust"
    }
  }
}'
```

### Multi-Index Search with Wildcard

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

## Bulk Operations

### Bulk Index Documents

Create a file `bulk_data.ndjson`:
```
{"index":{"_index":"blog_posts","_id":"1"}}
{"title":"Post 1","content":"Content 1","author":"Author 1"}
{"index":{"_index":"blog_posts","_id":"2"}}
{"title":"Post 2","content":"Content 2","author":"Author 2"}
{"index":{"_index":"blog_posts","_id":"3"}}
{"title":"Post 3","content":"Content 3","author":"Author 1"}
```

Then execute:
```bash
curl -X POST "http://localhost:9200/_bulk" -H 'Content-Type: application/x-ndjson' --data-binary @bulk_data.ndjson
```

### Bulk Update Documents

```
{"update":{"_index":"blog_posts","_id":"1"}}
{"doc":{"views":200}}
{"update":{"_index":"blog_posts","_id":"2"}}
{"doc":{"views":150}}
```

### Bulk Delete Documents

```
{"delete":{"_index":"blog_posts","_id":"1"}}
{"delete":{"_index":"blog_posts","_id":"2"}}
```

### Mixed Bulk Operations

```
{"index":{"_index":"blog_posts","_id":"4"}}
{"title":"New Post","content":"New content"}
{"update":{"_index":"blog_posts","_id":"3"}}
{"doc":{"views":300}}
{"delete":{"_index":"blog_posts","_id":"1"}}
```

## Real-World Scenarios

### Scenario 1: Blog Search Engine

```bash
# 1. Create index
curl -X PUT "http://localhost:9200/blog" -H 'Content-Type: application/json' -d'
{
  "mappings": {
    "properties": {
      "title": { "type": "text" },
      "body": { "type": "text" },
      "author": { "type": "keyword" },
      "published": { "type": "boolean" },
      "published_at": { "type": "date" }
    }
  }
}'

# 2. Index posts
curl -X PUT "http://localhost:9200/blog/_doc/1" -H 'Content-Type: application/json' -d'
{
  "title": "Introduction to Rust",
  "body": "Rust is a modern systems programming language...",
  "author": "Alice",
  "published": true,
  "published_at": "2024-01-01"
}'

# 3. Search published posts
curl -X POST "http://localhost:9200/blog/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "bool": {
      "must": [
        { "match": { "body": "Rust" } },
        { "term": { "published": true } }
      ]
    }
  },
  "highlight": {
    "fields": {
      "body": {}
    }
  }
}'
```

### Scenario 2: Product Catalog

```bash
# 1. Create products index
curl -X PUT "http://localhost:9200/products" -H 'Content-Type: application/json' -d'
{
  "mappings": {
    "properties": {
      "name": { "type": "text" },
      "description": { "type": "text" },
      "price": { "type": "float" },
      "category": { "type": "keyword" },
      "in_stock": { "type": "boolean" }
    }
  }
}'

# 2. Search products in price range
curl -X POST "http://localhost:9200/products/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "bool": {
      "must": [
        { "match": { "name": "laptop" } },
        { "term": { "in_stock": true } }
      ],
      "filter": [
        { "range": { "price": { "gte": 500, "lte": 2000 } } }
      ]
    }
  },
  "sort": [
    { "price": { "order": "asc" } }
  ]
}'
```

### Scenario 3: Log Analysis

```bash
# 1. Create daily log indices
curl -X PUT "http://localhost:9200/logs-2024-01-15" -H 'Content-Type: application/json' -d'
{
  "mappings": {
    "properties": {
      "timestamp": { "type": "date" },
      "level": { "type": "keyword" },
      "message": { "type": "text" },
      "service": { "type": "keyword" }
    }
  }
}'

# 2. Search across all January logs
curl -X POST "http://localhost:9200/_search" -H 'Content-Type: application/json' -d'
{
  "index": "logs-2024-01-*",
  "query": {
    "bool": {
      "must": [
        { "match": { "message": "error" } }
      ],
      "filter": [
        { "term": { "level": "ERROR" } }
      ]
    }
  },
  "sort": [
    { "timestamp": { "order": "desc" } }
  ],
  "size": 100
}'
```

### Scenario 4: User Search

```bash
# 1. Create users index
curl -X PUT "http://localhost:9200/users" -H 'Content-Type: application/json' -d'
{
  "mappings": {
    "properties": {
      "name": { "type": "text" },
      "email": { "type": "keyword" },
      "role": { "type": "keyword" },
      "active": { "type": "boolean" }
    }
  }
}'

# 2. Search active users by name
curl -X POST "http://localhost:9200/users/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "bool": {
      "must": [
        { "match": { "name": "John" } },
        { "term": { "active": true } }
      ]
    }
  },
  "_source": ["name", "email", "role"]
}'
```

## Tips and Best Practices

1. **Use Bulk Operations**: When indexing many documents, use bulk operations instead of individual requests.

2. **Index Naming**: Use lowercase names with underscores (e.g., `blog_posts`, `user_profiles`).

3. **Field Types**: Choose appropriate field types:
   - `text`: For full-text search
   - `keyword`: For exact matches and filtering
   - `integer`/`float`: For numeric values
   - `date`: For date/time values
   - `boolean`: For true/false values

4. **Query Performance**: Use `filter` in bool queries for exact matches (faster than `must`).

5. **Pagination**: Always use `from` and `size` for pagination to avoid loading too many results.

6. **Highlighting**: Only highlight fields that are displayed to users to reduce response size.

7. **Source Filtering**: Use `_source` filtering to reduce response size when you only need specific fields.
