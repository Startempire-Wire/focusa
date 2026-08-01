import { defineConfig } from "vite";
import { resolve } from "node:path";

export default defineConfig({
  build: {
    emptyOutDir: false,
    lib: {
      entry: resolve(__dirname, "src/rich-host.ts"),
      formats: ["es"],
      fileName: () => "a2ui-runtime.js",
    },
    outDir: resolve(__dirname, "../../apps/pi-extension/rich-host/assets"),
    sourcemap: false,
    minify: "esbuild",
    target: "es2022",
    rollupOptions: {
      output: {
        assetFileNames: "a2ui-[name][extname]",
      },
    },
  },
});
