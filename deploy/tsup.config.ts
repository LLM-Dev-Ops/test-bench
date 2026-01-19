import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['deploy/server.ts'],
  format: ['esm'],
  target: 'node20',
  outDir: 'dist/deploy',
  sourcemap: true,
  clean: true,
  external: ['openai'],
  noExternal: [/.*/],
  platform: 'node',
  bundle: true,
});
