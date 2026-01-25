/**
 * In-Memory Cache - Read-Only Operations
 *
 * PHASE 1 - FOUNDATIONAL TOOLING (Layer 1)
 *
 * Provides TTL-based caching for read-only operations:
 * - Telemetry reads
 * - Registry lookups
 * - Schema checks
 *
 * Default TTL: 30-60 seconds
 */

// =============================================================================
// CACHE CONFIGURATION
// =============================================================================

const DEFAULT_TTL_MS = 30_000; // 30 seconds
const MAX_TTL_MS = 60_000; // 60 seconds
const MAX_CACHE_SIZE = 100; // Maximum entries

// =============================================================================
// CACHE ENTRY
// =============================================================================

interface CacheEntry<T> {
  value: T;
  expiresAt: number;
  createdAt: number;
}

// =============================================================================
// READ-ONLY CACHE
// =============================================================================

export class ReadOnlyCache<T> {
  private cache: Map<string, CacheEntry<T>> = new Map();
  private ttlMs: number;

  constructor(ttlMs: number = DEFAULT_TTL_MS) {
    this.ttlMs = Math.min(ttlMs, MAX_TTL_MS);
  }

  /**
   * Get a cached value if it exists and is not expired
   */
  get(key: string): T | undefined {
    const entry = this.cache.get(key);
    if (!entry) return undefined;

    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      return undefined;
    }

    return entry.value;
  }

  /**
   * Set a cached value
   */
  set(key: string, value: T, ttlMs?: number): void {
    // Enforce max cache size with LRU eviction
    if (this.cache.size >= MAX_CACHE_SIZE) {
      this.evictOldest();
    }

    const effectiveTtl = ttlMs ? Math.min(ttlMs, MAX_TTL_MS) : this.ttlMs;
    const now = Date.now();

    this.cache.set(key, {
      value,
      expiresAt: now + effectiveTtl,
      createdAt: now,
    });
  }

  /**
   * Check if a key exists and is not expired
   */
  has(key: string): boolean {
    return this.get(key) !== undefined;
  }

  /**
   * Delete a cached entry
   */
  delete(key: string): boolean {
    return this.cache.delete(key);
  }

  /**
   * Clear all cached entries
   */
  clear(): void {
    this.cache.clear();
  }

  /**
   * Get cache statistics
   */
  stats(): CacheStats {
    let validCount = 0;
    let expiredCount = 0;
    const now = Date.now();

    for (const entry of this.cache.values()) {
      if (now > entry.expiresAt) {
        expiredCount++;
      } else {
        validCount++;
      }
    }

    return {
      total_entries: this.cache.size,
      valid_entries: validCount,
      expired_entries: expiredCount,
      max_size: MAX_CACHE_SIZE,
      ttl_ms: this.ttlMs,
    };
  }

  /**
   * Remove expired entries
   */
  prune(): number {
    const now = Date.now();
    let removed = 0;

    for (const [key, entry] of this.cache.entries()) {
      if (now > entry.expiresAt) {
        this.cache.delete(key);
        removed++;
      }
    }

    return removed;
  }

  /**
   * Evict the oldest entry (LRU)
   */
  private evictOldest(): void {
    let oldestKey: string | null = null;
    let oldestTime = Infinity;

    for (const [key, entry] of this.cache.entries()) {
      if (entry.createdAt < oldestTime) {
        oldestTime = entry.createdAt;
        oldestKey = key;
      }
    }

    if (oldestKey) {
      this.cache.delete(oldestKey);
    }
  }
}

// =============================================================================
// TYPES
// =============================================================================

export interface CacheStats {
  total_entries: number;
  valid_entries: number;
  expired_entries: number;
  max_size: number;
  ttl_ms: number;
}

// =============================================================================
// SINGLETON CACHES FOR COMMON USE CASES
// =============================================================================

// Telemetry reads cache (30s TTL)
const telemetryCache = new ReadOnlyCache<unknown>(30_000);

// Registry lookups cache (60s TTL)
const registryCache = new ReadOnlyCache<unknown>(60_000);

// Schema checks cache (60s TTL)
const schemaCache = new ReadOnlyCache<unknown>(60_000);

/**
 * Get the telemetry cache instance
 */
export function getTelemetryCache(): ReadOnlyCache<unknown> {
  return telemetryCache;
}

/**
 * Get the registry cache instance
 */
export function getRegistryCache(): ReadOnlyCache<unknown> {
  return registryCache;
}

/**
 * Get the schema cache instance
 */
export function getSchemaCache(): ReadOnlyCache<unknown> {
  return schemaCache;
}

// =============================================================================
// CACHED FETCH WRAPPER
// =============================================================================

/**
 * Fetch with caching for read-only GET requests
 */
export async function cachedFetch<T>(
  url: string,
  cache: ReadOnlyCache<T>,
  options?: { ttlMs?: number; key?: string }
): Promise<T> {
  const cacheKey = options?.key ?? url;

  // Check cache first
  const cached = cache.get(cacheKey);
  if (cached !== undefined) {
    return cached;
  }

  // Fetch from source
  const response = await fetch(url, {
    method: 'GET',
    headers: { 'Content-Type': 'application/json' },
  });

  if (!response.ok) {
    throw new Error(`Fetch failed: ${response.status}`);
  }

  const data = await response.json() as T;

  // Cache the result
  cache.set(cacheKey, data, options?.ttlMs);

  return data;
}
