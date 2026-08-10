import { existsSync, readFileSync } from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";
function normalizeRoot(value) {
    const raw = String(value || "").trim();
    if (!raw)
        return "";
    return raw.length > 1 ? raw.replace(/\/+$/, "") : raw;
}
export function resolveCanonicalMarkerProjectRoot(start, maxDepth = 12) {
    let current = resolve(String(start || ""));
    for (let depth = 0; depth <= maxDepth; depth += 1) {
        const markerPath = join(current, ".focusa-project.json");
        if (existsSync(markerPath)) {
            try {
                const marker = JSON.parse(readFileSync(markerPath, "utf8"));
                const declared = normalizeRoot(marker?.project_root);
                if (declared && isAbsolute(declared) && declared !== "/" && declared !== "/root" && declared !== "/tmp") {
                    return declared;
                }
            }
            catch {
                return "";
            }
        }
        const parent = dirname(current);
        if (parent === current)
            break;
        current = parent;
    }
    return "";
}
function identityBody(identity) {
    return identity?.project_identity && typeof identity.project_identity === "object"
        ? identity.project_identity
        : identity || {};
}
export function resolveProjectIdentityLookupCwd(input) {
    const projectRoot = normalizeRoot(input.projectRoot);
    const ambientCwd = normalizeRoot(input.ambientCwd);
    if (ambientCwd === projectRoot || ambientCwd.startsWith(`${projectRoot}/`))
        return ambientCwd;
    const persisted = identityBody(input.persistedIdentity);
    const persistedParent = normalizeRoot(persisted.canonical_parent_root || persisted.project_root);
    const persistedWorktree = normalizeRoot(persisted.active_worktree_root || persisted.working_context?.active_worktree_root);
    const persistedSubpathId = String(persisted.working_context?.working_subpath?.working_subpath_id || "").trim();
    const verified = persisted.status === "verified" || persisted.verified === true;
    if (verified &&
        persistedParent === projectRoot &&
        persistedWorktree &&
        persistedWorktree !== "/" &&
        persistedWorktree !== "/root" &&
        persistedWorktree !== "/tmp" &&
        persistedSubpathId &&
        persistedSubpathId !== "primary") {
        return persistedWorktree;
    }
    return projectRoot;
}
