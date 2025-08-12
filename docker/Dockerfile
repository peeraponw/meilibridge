# Multi-stage build for optimal image size
# Build stage
FROM rust:1.75-bookworm as builder

# Add labels for metadata
ARG BUILD_DATE
ARG VERSION
ARG VCS_REF
LABEL org.opencontainers.image.created="${BUILD_DATE}" \
      org.opencontainers.image.title="MeiliBridge" \
      org.opencontainers.image.description="Real-time PostgreSQL to Meilisearch synchronization" \
      org.opencontainers.image.authors="MeiliBridge Contributors" \
      org.opencontainers.image.vendor="MeiliBridge" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.source="https://github.com/your-org/meilibridge" \
      org.opencontainers.image.documentation="https://github.com/your-org/meilibridge/blob/main/README.md" \
      org.opencontainers.image.licenses="MIT"

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/meilibridge

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies - this is cached as long as manifests don't change
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash meilibridge

# Create necessary directories
RUN mkdir -p /etc/meilibridge /var/lib/meilibridge /var/log/meilibridge && \
    chown -R meilibridge:meilibridge /etc/meilibridge /var/lib/meilibridge /var/log/meilibridge

# Copy binary from builder
COPY --from=builder /usr/src/meilibridge/target/release/meilibridge /usr/local/bin/meilibridge

# Copy entrypoint script
COPY ./docker/docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Copy default configuration (optional)
# COPY config.example.yaml /etc/meilibridge/config.yaml

# Set user
USER meilibridge

# Default port environment variables
ENV MEILIBRIDGE_API_PORT=7701
ENV MEILIBRIDGE_METRICS_PORT=7702

# Expose ports (these are just documentation, actual binding is controlled by the app)
# API port
EXPOSE ${MEILIBRIDGE_API_PORT}
# Metrics port (future implementation)
EXPOSE ${MEILIBRIDGE_METRICS_PORT}
# Reserved for future features (7703-7710)

# Set working directory
WORKDIR /var/lib/meilibridge

# Health check (uses the configured port)
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${MEILIBRIDGE_API_PORT:-7701}/health || exit 1

# Use the entrypoint script
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["--config", "/etc/meilibridge/config.yaml"]
