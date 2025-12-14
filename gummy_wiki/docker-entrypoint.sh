#!/bin/sh
set -e

# Wait for Gummy Search to be ready (if curl is available)
if command -v curl > /dev/null 2>&1 || command -v wget > /dev/null 2>&1; then
    echo "Waiting for Gummy Search to be ready..."
    if command -v curl > /dev/null 2>&1; then
        until curl -f http://gummy-search:9200/_cluster/health > /dev/null 2>&1; do
            echo "Gummy Search is unavailable - sleeping"
            sleep 2
        done
    elif command -v wget > /dev/null 2>&1; then
        until wget --quiet --spider http://gummy-search:9200/_cluster/health > /dev/null 2>&1; do
            echo "Gummy Search is unavailable - sleeping"
            sleep 2
        done
    fi
    echo "Gummy Search is ready!"
fi

# Run migrations (only if database file doesn't exist or force flag is set)
if [ ! -f /var/www/html/database/database.sqlite ] || [ "$FORCE_MIGRATE" = "true" ]; then
    echo "Running migrations..."
    php artisan migrate --force || true
fi

# Optionally run seeders
if [ "$RUN_SEEDERS" = "true" ]; then
    echo "Running seeders..."
    php artisan db:seed --force || true
fi

# Create search indices
if [ "$CREATE_INDICES" = "true" ]; then
    echo "Creating search indices..."
    php artisan elastic:create-index || true
fi

# Import data to search
if [ "$IMPORT_TO_SEARCH" = "true" ]; then
    echo "Importing data to search index..."
    php artisan scout:import || true
fi

exec "$@"
