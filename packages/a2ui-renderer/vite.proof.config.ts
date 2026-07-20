import { defineConfig } from "vite";
import { resolve } from "node:path";

export default defineConfig({
  root: resolve(import.meta.dirname, "proof"),
  build: {
    outDir: resolve(import.meta.dirname, "proof-dist"),
    emptyOutDir: true,
  },
});
