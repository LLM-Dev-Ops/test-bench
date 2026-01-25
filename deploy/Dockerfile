# =============================================================================
# LLM-Test-Bench - Production Dockerfile
# =============================================================================
# Multi-stage build for minimal image size and security
# Target: Google Cloud Run
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Build
# -----------------------------------------------------------------------------
FROM node:20-alpine AS builder

WORKDIR /app

# Install build tools for native modules
RUN apk add --no-cache python3 make g++ git

# Install dependencies first (better caching)
COPY package.json package-lock.json ./
RUN npm install --production=false --ignore-scripts || npm install --production=false

# Copy source files
COPY tsconfig.json ./
COPY deploy/main.ts ./deploy/

# Build the server
RUN npx tsup deploy/main.ts \
  --format esm \
  --target node20 \
  --outDir dist \
  --minify

# -----------------------------------------------------------------------------
# Stage 2: Production
# -----------------------------------------------------------------------------
FROM node:20-alpine AS production

# Security: Run as non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001

WORKDIR /app

# Copy built files only
COPY --from=builder /app/dist ./dist

# Set ownership
RUN chown -R nodejs:nodejs /app

USER nodejs

# Environment
ENV NODE_ENV=production
ENV PORT=8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Expose port
EXPOSE 8080

# Start server
CMD ["node", "dist/main.js"]
