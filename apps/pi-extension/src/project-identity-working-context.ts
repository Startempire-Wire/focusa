function normalizeRoot(value: unknown): string {
  const raw = String(value || "").trim();
  if (!raw) return "";
  return raw.length > 1 ? raw.replace(/\/+$/, "") : raw;
}

function identityBody(identity: any): any {
  return identity?.project_identity && typeof identity.project_identity === "object"
    ? identity.project_identity
    : identity || {};
}

export function resolveProjectIdentityLookupCwd(input: {
  projectRoot: string;
  ambientCwd: string;
  persistedIdentity?: any;
}): string {
  const projectRoot = normalizeRoot(input.projectRoot);
  const ambientCwd = normalizeRoot(input.ambientCwd);
  if (ambientCwd === projectRoot || ambientCwd.startsWith(`${projectRoot}/`)) return ambientCwd;

  const persisted = identityBody(input.persistedIdentity);
  const persistedParent = normalizeRoot(
    persisted.canonical_parent_root || persisted.project_root
  );
  const persistedWorktree = normalizeRoot(
    persisted.active_worktree_root || persisted.working_context?.active_worktree_root
  );
  const persistedSubpathId = String(
    persisted.working_context?.working_subpath?.working_subpath_id || ""
  ).trim();
  const verified = persisted.status === "verified" || persisted.verified === true;

  if (
    verified &&
    persistedParent === projectRoot &&
    persistedWorktree &&
    persistedWorktree !== "/" &&
    persistedWorktree !== "/root" &&
    persistedWorktree !== "/tmp" &&
    persistedSubpathId &&
    persistedSubpathId !== "primary"
  ) {
    return persistedWorktree;
  }
  return projectRoot;
}
