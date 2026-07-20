import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { resolve } from "node:path";

export default defineConfig({
  resolve: { conditions: ["browser"] },
  plugins: [svelte({ compilerOptions: { customElement: true } })],
  build: {
    lib: {
      entry: resolve(import.meta.dirname, "src/index.ts"),
      formats: ["es"],
      fileName: "focusa-elements",
    },
    rollupOptions: {
      external: [/^svelte(?:\/.*)?$/],
    },
  },
  test: {
    environment: "jsdom",
    include: ["tests/**/*.test.ts"],
  },
});
