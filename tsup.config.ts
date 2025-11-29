import { defineConfig } from 'tsup';

export default defineConfig([
  // Main SDK build
  {
    entry: ['src/index.ts'],
    format: ['esm'],
    dts: true,
    clean: true,
    sourcemap: true,
    minify: false,
    splitting: false,
    treeshake: true,
    external: ['openai'],
    platform: 'node',
    target: 'node18',
  },
  // CLI build (with shebang preserved from source)
  {
    entry: ['src/cli.ts'],
    format: ['esm'],
    dts: true,
    clean: false, // Don't clean since index.ts already cleaned
    sourcemap: true,
    minify: false,
    splitting: false,
    treeshake: true,
    platform: 'node',
    target: 'node18',
    // Shebang is already in source file, don't add it again
  },
]);
