# Multi-stage build for minimal production image
# Stage 1: Builder
FROM rust:1.75-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY cli/Cargo.toml ./cli/
COPY core/Cargo.toml ./core/
COPY datasets/Cargo.toml ./datasets/

# Copy source code
COPY cli ./cli
COPY core ./core
COPY datasets ./datasets

# Build for release
RUN cargo build --release --locked

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash llmuser

# Copy binary from builder
COPY --from=builder /app/target/release/llm-test-bench /usr/local/bin/

# Create directories for data
RUN mkdir -p /data/datasets /data/results /data/cache && \
    chown -R llmuser:llmuser /data

# Switch to non-root user
USER llmuser
WORKDIR /data

# Set environment variables
ENV RUST_LOG=info
ENV LLM_TEST_BENCH_CONFIG=/data/config.toml

# Entrypoint
ENTRYPOINT ["llm-test-bench"]
CMD ["--help"]

# Labels
LABEL org.opencontainers.image.title="LLM Test Bench"
LABEL org.opencontainers.image.description="Enterprise-grade LLM testing and benchmarking framework"
LABEL org.opencontainers.image.version="0.4.0"
LABEL org.opencontainers.image.vendor="LLM Test Bench Project"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
