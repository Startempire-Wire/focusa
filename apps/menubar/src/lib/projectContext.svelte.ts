// projectContext — extract project_root + continuity_id + session_id + work_item_id
// from the runtime snapshot so the action layer can scope requests without
// re-deriving the values per tab.

export interface ProjectContext {
  projectRoot: string;
  continuityId: string;
  sessionId?: string;
  workItemId?: string;
}

export function getProjectContext(s: any): ProjectContext {
  const project = s?.projectIdentity ?? {};
  const workpoint = s?.workpointResume ?? s?.workpoint ?? {};
  const packet = workpoint?.resume_packet ?? workpoint?.packet ?? workpoint;
  return {
    projectRoot: String(
      project.project_root ||
      project.root ||
      project.workspace_root ||
      project.project?.root ||
      packet?.scope?.project_root ||
      '',
    ),
    continuityId: String(
      project.continuity_id ||
      packet?.scope?.continuity_id ||
      workpoint.continuity_id ||
      '',
    ),
    sessionId: String(packet?.scope?.session_id || workpoint.session_id || '') || undefined,
    workItemId: String(packet?.work_item_id || workpoint.work_item_id || '') || undefined,
  };
}
