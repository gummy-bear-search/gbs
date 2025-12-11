# Build stage
FROM rust:1.91.1-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig

# Set up Rust environment
ENV RUST_BACKTRACE=1

# Create app directory
WORKDIR /app

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY examples ./examples
COPY tests ./tests

# Build the project
RUN cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc

# Create app user
RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser

# Copy binary from builder stage
COPY --from=builder /app/target/release/gummy-search /usr/local/bin/gummy-search

# Make binary executable
RUN chmod +x /usr/local/bin/gummy-search

# Switch to non-root user
USER appuser

# Expose port (if needed for future server implementation)
EXPOSE 9200

# Default command
CMD ["gummy-search"]
