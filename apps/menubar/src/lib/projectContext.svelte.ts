// projectContext — extract project_root + continuity_id + session_id + work_item_id
// from the runtime snapshot so the action layer can scope requests without
// re-deriving the values per tab.

/** Spec104 MEN-01: typed ScopeContext (advisory, not canonical). */
export interface ScopeContext {
  /** Canonical parent project root (legacy field name retained for compatibility). */
  project_root: string;
  active_worktree_root?: string;
  working_subpath_id?: string;
  continuity_id: string;
  session_id?: string;
  work_item_id?: string;
  /** Daemon's reported scope source: header, query, or default. */
  scope_source?: "header" | "query" | "default";
  /** Daemon's reported scope status: ok | blocked | advisory. */
  scope_status?: "ok" | "blocked" | "advisory" | "unknown";
  /** Whether this ScopeContext is canonical authority or just advisory. */
  canonical_scope?: boolean;
}

export interface ProjectContext extends ScopeContext {
  /** Alias of canonical parent project_root for legacy callers. */
  projectRoot: string;
  activeWorktreeRoot?: string;
  workingSubpathId?: string;
  /** Alias of continuity_id for legacy callers. */
  continuityId: string;
  sessionId?: string;
  workItemId?: string;
}

export function getProjectContext(s: any): ProjectContext {
  const project = s?.projectIdentity ?? {};
  const workpoint = s?.workpointResume ?? s?.workpoint ?? {};
  const packet = workpoint?.resume_packet ?? workpoint?.packet ?? workpoint;
  const projectRoot = String(
    project.project_root ||
    project.root ||
    project.workspace_root ||
    project.project?.root ||
    packet?.scope?.project_root ||
    '',
  );
  const activeWorktreeRoot = String(
    project.active_worktree_root ||
    project.working_context?.active_worktree_root ||
    packet?.scope?.active_worktree_root ||
    projectRoot
  );
  const workingSubpathId = String(
    project.working_context?.working_subpath?.working_subpath_id ||
    packet?.scope?.working_subpath_id ||
    workpoint.working_subpath_id ||
    "primary"
  );
  const continuityId = String(
    project.continuity_id ||
    packet?.scope?.continuity_id ||
    workpoint.continuity_id ||
    '',
  );
  const sessionId = String(packet?.scope?.session_id || workpoint.session_id || '') || undefined;
  const workItemId = String(packet?.work_item_id || workpoint.work_item_id || '') || undefined;
  return {
    // Snake_case fields satisfy ScopeContext (Spec104 MEN-01 typed interface).
    project_root: projectRoot,
    active_worktree_root: activeWorktreeRoot,
    working_subpath_id: workingSubpathId,
    continuity_id: continuityId,
    session_id: sessionId,
    work_item_id: workItemId,
    // CamelCase aliases satisfy legacy ProjectContext callers.
    projectRoot,
    activeWorktreeRoot,
    workingSubpathId,
    continuityId,
    sessionId,
    workItemId,
  };
}

/**
 * Spec104 MEN-04..08: derive typed scope status for Peek components.
 * Returns an object with text-friendly display strings + raw values.
 */
export function deriveTypedScopeStatus(scope?: Partial<ScopeContext> | null): {
  status: 'ok' | 'blocked' | 'advisory' | 'unknown';
  statusText: string;
  projectRoot: string;
  continuityId: string;
  isAdvisory: boolean;
} {
  const status = (scope?.scope_status as 'ok' | 'blocked' | 'advisory' | 'unknown') ?? 'unknown';
  return {
    status,
    statusText: status,
    projectRoot: scope?.project_root ?? '',
    continuityId: scope?.continuity_id ?? '',
    isAdvisory: scope?.canonical_scope === false || status === 'advisory',
  };
}

/** Spec104 WL-02: format scope + advisory status for TUI/menubar display. */
export function formatScopeForDisplay(scope?: Partial<ScopeContext> | null): string {
  const status = deriveTypedScopeStatus(scope);
  const workingRoot = scope?.active_worktree_root || status.projectRoot;
  const workingId = scope?.working_subpath_id || 'primary';
  const authority = `parent=${status.projectRoot} worktree=${workingRoot} subpath=${workingId}`;
  if (status.status === 'blocked') return `[BLOCKED] ${authority}`;
  if (status.isAdvisory) return `[advisory] ${authority} (${status.continuityId})`;
  return `${authority} (${status.continuityId})`;
}
