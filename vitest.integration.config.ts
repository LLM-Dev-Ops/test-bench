import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
  test: {
    // Test environment
    environment: 'node',

    // Global test setup
    globals: true,
    setupFiles: ['./tests/setup.integration.ts'],

    // Only run integration tests
    include: ['tests/integration/**/*.{test,spec}.{js,ts}'],
    exclude: ['node_modules', 'dist'],

    // Longer timeout for integration tests
    testTimeout: 30000,
    hookTimeout: 30000,

    // Run sequentially for integration tests
    threads: false,
    isolate: true,

    // Reporters
    reporters: ['verbose'],

    // No coverage for integration tests
    coverage: {
      enabled: false,
    },
  },

  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@tests': path.resolve(__dirname, './tests'),
    },
  },
});
