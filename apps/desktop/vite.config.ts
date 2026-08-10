import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const repositoryRoot = fileURLToPath(new URL('../../', import.meta.url));

// The repository has no root package.json, so vite's workspace-root detection
// cannot discover it; the sveltekit plugin then narrows server.fs.allow to
// src/.svelte-kit/node_modules and the fixture JSONs under tests/ 403. Pin the
// allow list to the repository root so fixture imports (tests/fixtures/...) and
// workspace files resolve in dev and preview.
export default defineConfig({
  plugins: [sveltekit()],
  server: {
    host: '127.0.0.1',
    port: 1430,
    strictPort: true,
    fs: {
      allow: [repositoryRoot, path.resolve(repositoryRoot, 'apps/desktop')]
    }
  },
  build: {
    target: ['es2021', 'chrome100', 'safari15'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: Boolean(process.env.TAURI_DEBUG)
  }
});
