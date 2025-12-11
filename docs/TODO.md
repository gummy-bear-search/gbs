# TODO - Gummy Search Development

This document tracks the development progress and tasks for the Gummy Search project.

## Critical Endpoints (MVP)

### Index Management
- [ ] **PUT /{index}** - Create index with settings and mappings
  - [ ] Implement index creation endpoint
  - [ ] Support settings (number_of_shards, number_of_replicas, analysis)
  - [ ] Support mappings (properties, field types, analyzers)
  - [ ] Index name validation
  - [ ] Error handling (index already exists)

- [ ] **HEAD /{index}** - Check index existence
  - [ ] Return 200 if exists, 404 if not

- [ ] **GET /{index}** - Get index information
  - [ ] Return settings and mappings
  - [ ] Handle non-existent index

- [ ] **PUT /{index}/_mapping** - Update index mapping
  - [ ] Update existing mappings
  - [ ] Add new fields
  - [ ] Validate compatibility

- [ ] **PUT /{index}/_settings** - Update index settings
  - [ ] Update analysis settings
  - [ ] Update dynamic settings
  - [ ] Validate changes

- [ ] **DELETE /{index}** - Delete index
  - [ ] Delete single index
  - [ ] Support DELETE /_all (dangerous operation)
  - [ ] Confirmation mechanism

### Document Operations
- [ ] **PUT /{index}/_doc/{id}** - Index document (create/update)
  - [ ] Create document with specified ID
  - [ ] Update existing document
  - [ ] Document data validation

- [ ] **POST /{index}/_doc** - Index document with auto-generated ID
  - [ ] Generate unique ID
  - [ ] Create document

- [ ] **GET /{index}/_doc/{id}** - Get document
  - [ ] Return document by ID
  - [ ] Handle 404 for non-existent document
  - [ ] Support _source filtering

- [ ] **DELETE /{index}/_doc/{id}** - Delete document
  - [ ] Delete document by ID
  - [ ] Handle non-existent document
  - [ ] Return operation status

### Bulk Operations
- [ ] **POST /_bulk** - Bulk operations
  - [ ] Support index operation
  - [ ] Support create operation
  - [ ] Support update operation
  - [ ] Support delete operation
  - [ ] Handle NDJSON format
  - [ ] Return results for each operation
  - [ ] Handle partial errors
  - [ ] Support refresh parameter

- [ ] **POST /{index}/_bulk** - Bulk operations for specific index
  - [ ] Same as above but with default index

### Search
- [ ] **POST /{index}/_search** - Search documents
  - [ ] Implement match query
  - [ ] Implement match_phrase query
  - [ ] Implement multi_match query
  - [ ] Implement term query
  - [ ] Implement terms query
  - [ ] Implement range query
  - [ ] Implement bool query (must, should, must_not, filter)
  - [ ] Implement wildcard query
  - [ ] Implement prefix query
  - [ ] Support filters
  - [ ] Support sorting
  - [ ] Support pagination (from, size)
  - [ ] Support highlighting
  - [ ] Support aggregations (optional)
  - [ ] Return _score for relevance
  - [ ] Support _source filtering

- [ ] **GET /{index}/_search** - Search with GET method
  - [ ] Same as POST but with query parameters

- [ ] **POST /_search** - Multi-index search
  - [ ] Search across multiple indices
  - [ ] Support wildcards in index names

### Index Refresh
- [ ] **POST /{index}/_refresh** - Refresh index
  - [ ] Refresh single index
  - [ ] Make changes visible for search

- [ ] **POST /_refresh** - Refresh all indices
  - [ ] Refresh all indices

## Important Endpoints (Full Functionality)

### Cluster Information
- [ ] **GET /_cluster/health** - Cluster health
  - [ ] Return cluster status (green, yellow, red)
  - [ ] Return node count
  - [ ] Return index count

- [ ] **GET /_cluster/stats** - Cluster statistics
  - [ ] Statistics by indices
  - [ ] Statistics by nodes
  - [ ] Memory usage

### Index Listing
- [ ] **GET /_cat/indices?v** - List indices
  - [ ] List all indices
  - [ ] Return document count per index
  - [ ] Return size per index

- [ ] **GET /_aliases** - List aliases
  - [ ] Return index aliases

## Implementation Tasks

### Core Infrastructure
- [ ] Set up HTTP server (actix-web or axum)
- [ ] Implement request routing
- [ ] Implement JSON parsing (serde)
- [ ] Implement error handling
- [ ] Set up logging

### Storage Backend
- [ ] Choose storage solution (in-memory, RocksDB, etc.)
- [ ] Implement index storage
- [ ] Implement document storage
- [ ] Implement inverted index for search
- [ ] Implement tokenization and analysis

### Search Engine
- [ ] Implement tokenizer
- [ ] Implement analyzer pipeline
- [ ] Implement query parser
- [ ] Implement scoring algorithm
- [ ] Implement result ranking

### Testing
- [ ] Unit tests for each endpoint
- [ ] Integration tests
- [ ] Performance tests
- [ ] Compatibility tests with Elasticsearch 6.4.0

### Documentation
- [ ] API documentation
- [ ] Code documentation
- [ ] Usage examples
- [ ] Architecture documentation

## Performance Optimization
- [ ] Implement connection pooling
- [ ] Implement request caching
- [ ] Optimize bulk operations
- [ ] Optimize search queries
- [ ] Memory management

## Security
- [ ] Input validation
- [ ] Rate limiting
- [ ] Authentication (if needed)
- [ ] Authorization (if needed)

## Future Enhancements
- [ ] Support for more query types
- [ ] Support for aggregations
- [ ] Support for suggestions
- [ ] Support for percolate queries
- [ ] Support for scroll API
- [ ] Support for reindex API

## Notes

- Focus on MVP endpoints first (6 critical endpoints)
- Ensure compatibility with Elasticsearch 6.4.0 API
- Test against Laravel Scout usage patterns
- Maintain performance comparable to Elasticsearch
