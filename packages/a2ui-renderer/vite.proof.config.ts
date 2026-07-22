import { defineConfig } from "vite";
import { resolve } from "node:path";

export default defineConfig({
  resolve: { preserveSymlinks: true },
  root: resolve(import.meta.dirname, "proof"),
  build: {
    outDir: resolve(import.meta.dirname, "proof-dist"),
    emptyOutDir: true,
    rollupOptions: {
      input: {
        alpha0: resolve(import.meta.dirname, "proof/index.html"),
        contextCommit: resolve(import.meta.dirname, "proof/context-commit.html"),
        contextIngest: resolve(import.meta.dirname, "proof/context-ingest.html"),
        contextRetrieve: resolve(import.meta.dirname, "proof/context-retrieve.html"),
        contextClaims: resolve(import.meta.dirname, "proof/context-claims.html"),
        workspaceArtifact: resolve(import.meta.dirname, "proof/workspace-artifact.html"),
        workspaceLiveRefresh: resolve(import.meta.dirname, "proof/workspace-live-refresh.html"),
        roleProfile: resolve(import.meta.dirname, "proof/role-profile.html"),
        interviewStrategy: resolve(import.meta.dirname, "proof/interview-strategy.html"),
        interviewResume: resolve(import.meta.dirname, "proof/interview-resume.html"),
        specWorkbench: resolve(import.meta.dirname, "proof/spec-workbench.html"),
      },
    },
  },
});
