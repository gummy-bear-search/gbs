# Multi-stage Dockerfile for Laravel application
# Alpine-based for smaller image size

# Stage 1: Build stage
FROM php:8.2-fpm-alpine AS builder

# Install system dependencies and PHP extensions
RUN apk add --no-cache \
    git \
    curl \
    libpng-dev \
    libzip-dev \
    zip \
    unzip \
    oniguruma-dev \
    sqlite \
    && docker-php-ext-install \
    pdo \
    pdo_sqlite \
    mbstring \
    exif \
    pcntl \
    bcmath \
    gd \
    zip

# Install Composer
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

# Set working directory
WORKDIR /var/www/html

# Copy composer files
COPY composer.json composer.lock ./

# Install PHP dependencies
RUN composer install --no-dev --optimize-autoloader --no-interaction --prefer-dist

# Copy application code
COPY . .

# Set permissions
RUN chown -R www-data:www-data /var/www/html \
    && chmod -R 755 /var/www/html/storage \
    && chmod -R 755 /var/www/html/bootstrap/cache

# Stage 2: Production stage
FROM php:8.2-fpm-alpine

# Install system dependencies and PHP extensions
RUN apk add --no-cache \
    libpng \
    libzip \
    oniguruma \
    sqlite \
    curl \
    wget \
    && docker-php-ext-install \
    pdo \
    pdo_sqlite \
    mbstring \
    exif \
    pcntl \
    bcmath \
    gd \
    zip

# Install Composer (for artisan commands)
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

# Create www-data user if it doesn't exist
RUN addgroup -g 1000 www-data || true \
    && adduser -u 1000 -G www-data -s /bin/sh -D www-data || true

# Set working directory
WORKDIR /var/www/html

# Copy application files from builder
COPY --from=builder --chown=www-data:www-data /var/www/html /var/www/html

# Create necessary directories
RUN mkdir -p storage/framework/cache \
    storage/framework/sessions \
    storage/framework/views \
    storage/logs \
    bootstrap/cache \
    database \
    && chown -R www-data:www-data storage bootstrap/cache database

# Copy entrypoint script (before switching user)
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Expose port 9000 for PHP-FPM
EXPOSE 9000

# Switch to www-data user
USER www-data

# Use entrypoint script
ENTRYPOINT ["docker-entrypoint.sh"]

# Start PHP-FPM
CMD ["php-fpm"]
