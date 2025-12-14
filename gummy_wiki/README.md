# Gummy Wiki - Laravel Scout Integration Example

This is a Laravel application demonstrating how to use Gummy Search with Laravel Scout. It provides a wiki-like interface for searching and managing content.

## Features Demonstrated

- **Content Model**: Blog posts with title and body
- **Entity Model**: Generic entities
- **File Model**: File metadata
- **Dictionary Model**: Dictionary entries
- **Laravel Scout Integration**: Full search functionality
- **Artisan Commands**: Index creation, bulk import, search

## Setup

### Prerequisites

- PHP 8.1+ (or Docker)
- Composer (or Docker)
- Laravel 10+
- Gummy Search server running on `http://localhost:9200`

### Docker Setup (Recommended)

See [DOCKER.md](DOCKER.md) for complete Docker setup instructions.

Quick start:
```bash
docker-compose up -d --build
docker-compose exec app composer install
docker-compose exec app php artisan key:generate
docker-compose exec app php artisan migrate
docker-compose exec app php artisan seed:wikipedia
docker-compose exec app php artisan elastic:create-index
docker-compose exec app php artisan scout:import
```

Access the app at http://localhost:8000

### Installation

```bash
# Install dependencies
composer install

# Copy environment file
cp .env.example .env

# Generate application key
php artisan key:generate

# Run migrations
php artisan migrate

# Seed database with sample data
php artisan db:seed

# Or seed with real Wikipedia data
php artisan seed:wikipedia

# Configure Scout
# Edit .env and set:
SCOUT_DRIVER=elasticsearch
ELASTICSEARCH_HOST=http://localhost:9200

# Register Scout service provider
# Add to config/app.php providers array:
# App\Providers\ScoutServiceProvider::class,
```

### Running Gummy Search

Make sure Gummy Search is running:

```bash
# In the gummy-search project root
cargo run
```

## Usage

### Create Indices

```bash
php artisan elastic:create-index
```

This creates indices for all models:
- `content_index`
- `entity_index`
- `file_index`
- `dictionary_index`

### Import Data

```bash
# Import all models
php artisan scout:import

# Import specific model
php artisan scout:import "App\Models\Content"
```

### Search

```bash
# Search content
php artisan search:content "your search query"

# Or use in code
$results = Content::search('search query')->get();
```

### Flush Data

```bash
# Flush all indices
php artisan scout:flush

# Flush specific model
php artisan scout:flush "App\Models\Content"
```

## Models

### Content

Blog posts with title and body fields.

### Entity

Generic entities with name and description.

### File

File metadata with filename, path, and mime type.

### Dictionary

Dictionary entries with term and definition.

## API Endpoints Used

This example demonstrates the following Gummy Search endpoints:

- `PUT /{index}` - Create index
- `HEAD /{index}` - Check index existence
- `GET /{index}` - Get index information
- `PUT /{index}/_doc/{id}` - Index document
- `POST /{index}/_doc` - Create document with auto ID
- `GET /{index}/_doc/{id}` - Get document
- `DELETE /{index}/_doc/{id}` - Delete document
- `POST /{index}/_bulk` - Bulk operations
- `POST /{index}/_search` - Search documents
