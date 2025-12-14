# Docker Setup for Laravel Example App

This guide explains how to run the Laravel example application with Docker and Docker Compose.

## Prerequisites

- Docker 20.10+
- Docker Compose 2.0+

## Quick Start

### 1. Build and Start Services

```bash
docker-compose up -d --build
```

This will start:
- **Laravel App** (PHP-FPM) - Port 9000
- **Nginx** - Port 8000 (web server)
- **Gummy Search** - Port 9200 (search engine)

### 2. Install Dependencies

```bash
# Install Composer dependencies
docker-compose exec app composer install

# Create .env file if it doesn't exist
docker-compose exec app cp .env.example .env

# Generate application key
docker-compose exec app php artisan key:generate
```

### 3. Setup Database and Search

```bash
# Run migrations
docker-compose exec app php artisan migrate

# Seed database (optional)
docker-compose exec app php artisan db:seed

# Or seed with Wikipedia data
docker-compose exec app php artisan seed:wikipedia

# Create search indices
docker-compose exec app php artisan elastic:create-index

# Import data to search
docker-compose exec app php artisan scout:import
```

### 4. Access the Application

- **Web Interface**: http://localhost:8000
- **Gummy Search API**: http://localhost:9200
- **Laravel App (PHP-FPM)**: Internal only (port 9000)

## Services

### Laravel App (`app`)

- **Image**: Custom PHP 8.2 FPM Alpine
- **Port**: 9000 (internal)
- **Volumes**:
  - Application code: `./:/var/www/html`
  - Storage: `./storage:/var/www/html/storage`
  - Database: `./database:/var/www/html/database`

### Nginx (`nginx`)

- **Image**: nginx:alpine
- **Port**: 8000 (external)
- **Configuration**: `docker/nginx/default.conf`

### Gummy Search (`gummy-search`)

- **Image**: Built from parent directory
- **Port**: 9200 (external)
- **Volume**: `gummy-data:/app/data` (persistent storage)

## Environment Variables

Create a `.env` file or set environment variables:

```env
APP_ENV=local
APP_DEBUG=true
DB_CONNECTION=sqlite
DB_DATABASE=/var/www/html/database/database.sqlite
SCOUT_DRIVER=elasticsearch
ELASTICSEARCH_HOST=http://gummy-search:9200
```

### Docker Compose Environment Variables

- `RUN_SEEDERS=true` - Automatically run seeders on startup
- `IMPORT_TO_SEARCH=true` - Automatically import data to search on startup

Example:
```bash
RUN_SEEDERS=true IMPORT_TO_SEARCH=true docker-compose up -d
```

## Common Commands

### View Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f app
docker-compose logs -f gummy-search
docker-compose logs -f nginx
```

### Execute Artisan Commands

```bash
# Run any artisan command
docker-compose exec app php artisan <command>

# Examples
docker-compose exec app php artisan migrate
docker-compose exec app php artisan tinker
docker-compose exec app php artisan search:content "query"
```

### Execute Composer Commands

```bash
docker-compose exec app composer install
docker-compose exec app composer update
```

### Access Container Shell

```bash
# Laravel app container
docker-compose exec app sh

# Gummy Search container
docker-compose exec gummy-search sh
```

### Stop Services

```bash
# Stop all services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

### Rebuild Services

```bash
# Rebuild all services
docker-compose build --no-cache

# Rebuild specific service
docker-compose build --no-cache app
```

## Development Workflow

### 1. Initial Setup

```bash
# Start services
docker-compose up -d

# Install dependencies
docker-compose exec app composer install

# Setup environment
docker-compose exec app cp .env.example .env
docker-compose exec app php artisan key:generate

# Setup database
docker-compose exec app php artisan migrate

# Seed with Wikipedia data
docker-compose exec app php artisan seed:wikipedia

# Setup search
docker-compose exec app php artisan elastic:create-index
docker-compose exec app php artisan scout:import
```

### 2. Daily Development

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f app

# Run tests
docker-compose exec app php artisan test

# Run migrations
docker-compose exec app php artisan migrate
```

## Troubleshooting

### Permission Issues

If you encounter permission issues with storage or cache:

```bash
docker-compose exec app chmod -R 775 storage bootstrap/cache
docker-compose exec app chown -R www-data:www-data storage bootstrap/cache
```

### Gummy Search Not Ready

If Laravel can't connect to Gummy Search:

```bash
# Check if Gummy Search is running
docker-compose ps gummy-search

# Check Gummy Search logs
docker-compose logs gummy-search

# Test connection
curl http://localhost:9200/_cluster/health
```

### Database Issues

If SQLite database has issues:

```bash
# Remove database and recreate
docker-compose exec app rm database/database.sqlite
docker-compose exec app php artisan migrate
```

### Clear Cache

```bash
docker-compose exec app php artisan cache:clear
docker-compose exec app php artisan config:clear
docker-compose exec app php artisan route:clear
docker-compose exec app php artisan view:clear
```

## Production Considerations

For production deployment:

1. **Update `.env`**:
   ```env
   APP_ENV=production
   APP_DEBUG=false
   ```

2. **Optimize Laravel**:
   ```bash
   docker-compose exec app php artisan config:cache
   docker-compose exec app php artisan route:cache
   docker-compose exec app php artisan view:cache
   ```

3. **Use production Dockerfile** (if separate):
   - Remove development dependencies
   - Optimize Composer autoloader
   - Use production PHP settings

4. **Security**:
   - Use strong application keys
   - Restrict network access
   - Use HTTPS (add SSL/TLS termination)

## Network

All services are connected via the `gummy-network` bridge network:

- `app` can reach `gummy-search` at `http://gummy-search:9200`
- `nginx` can reach `app` at `http://app:9000`
- External access: `localhost:8000` (nginx) and `localhost:9200` (gummy-search)

## Volumes

- **Application code**: Mounted from host (for development)
- **Storage**: Persistent Laravel storage
- **Database**: SQLite database file
- **Gummy Search data**: Docker volume `gummy-data` (persistent)

## Health Checks

Gummy Search includes a health check that verifies the service is ready before Laravel starts. The health check:

- Tests `/_cluster/health` endpoint
- Runs every 10 seconds
- Times out after 5 seconds
- Retries 5 times
- Waits 10 seconds before first check
