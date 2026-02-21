import { defineConfig } from 'tsup';

/**
 * Build configuration for the Cloud Function entry point.
 *
 * Bundles all internal agent code into a single file.
 * All npm packages are externalized (resolved from node_modules at runtime).
 */
export default defineConfig({
  entry: ['functions/index.ts'],
  format: ['esm'],
  outDir: 'dist/functions',
  platform: 'node',
  target: 'node20',
  clean: false,
  sourcemap: true,
  bundle: true,
  splitting: false,
  // Externalize all bare imports (npm packages) - they'll be in node_modules at runtime
  external: [/^[^./]/],
});
