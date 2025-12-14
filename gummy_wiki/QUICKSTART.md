# Quick Start Guide

This guide will help you quickly set up and test Gummy Wiki with Gummy Search.

## Step 1: Start Gummy Search Server

In the gummy-search project root:

```bash
cd /path/to/gummy-search
cargo run
```

The server should start on `http://localhost:9200`.

Verify it's running:

```bash
curl http://localhost:9200/_cluster/health
```

## Step 2: Set Up Laravel Application

```bash
cd gummy_wiki

# Install dependencies
composer install

# Set up environment
cp .env.example .env
php artisan key:generate

# Run migrations
php artisan migrate

# Seed database (with fake data)
php artisan db:seed

# Or seed with real Wikipedia data (recommended for testing)
php artisan seed:wikipedia
```

## Step 3: Register Scout Service Provider

Add to `config/app.php` in the `providers` array:

```php
App\Providers\ScoutServiceProvider::class,
```

Or if using Laravel 11+, add to `bootstrap/providers.php`:

```php
<?php

return [
    // ... other providers
    App\Providers\ScoutServiceProvider::class,
];
```

## Step 4: Create Elasticsearch Indices

```bash
php artisan elastic:create-index
```

This creates indices for:
- `content_index`
- `entity_index`
- `file_index`
- `dictionary_index`

## Step 5: Import Data to Search Index

```bash
# Import all models
php artisan scout:import

# Or import specific model
php artisan scout:import "App\Models\Content"
```

## Step 6: Test Search

```bash
# Search via command
php artisan search:content "Laravel"

# Or test in tinker
php artisan tinker
>>> Content::search('test')->get();
```

## Example Code Usage

### Creating and Indexing Content

```php
use App\Models\Content;

// Create content
$content = Content::create([
    'title' => 'My First Post',
    'body' => 'This is the content of my first blog post.',
]);

// Index it
$content->searchable();

// Or it will auto-index if you enable Scout events
```

### Searching Content

```php
use App\Models\Content;

// Simple search
$results = Content::search('Laravel')->get();

// Search with pagination
$results = Content::search('test')
    ->paginate(10);

// Search with constraints
$results = Content::search('test')
    ->where('published_at', '!=', null)
    ->get();
```

### Bulk Operations

Laravel Scout automatically uses bulk operations when:
- Importing: `php artisan scout:import`
- Mass updating: `Model::where(...)->searchable()`
- Mass deleting: `Model::where(...)->unsearchable()`

## Testing Endpoints Directly

You can also test Gummy Search endpoints directly:

```bash
# Create an index
curl -X PUT "http://localhost:9200/test_index" \
  -H 'Content-Type: application/json' \
  -d '{"settings":{"number_of_shards":1}}'

# Index a document
curl -X PUT "http://localhost:9200/test_index/_doc/1" \
  -H 'Content-Type: application/json' \
  -d '{"title":"Test","body":"Content"}'

# Search
curl -X POST "http://localhost:9200/test_index/_search" \
  -H 'Content-Type: application/json' \
  -d '{"query":{"match":{"title":"Test"}}}'
```

## Troubleshooting

### Scout not finding documents

Make sure you've:
1. Created the indices (`php artisan elastic:create-index`)
2. Imported the data (`php artisan scout:import`)
3. Gummy Search server is running

### Connection errors

Check that:
- Gummy Search is running on `http://localhost:9200`
- `ELASTICSEARCH_HOST` in `.env` matches the server URL
- No firewall is blocking the connection

### Search not working

Note: Search functionality requires the search endpoint to be implemented in Gummy Search. Currently, the search route is defined but returns a "not implemented" error. Once search is implemented in Gummy Search, this example will work fully.
