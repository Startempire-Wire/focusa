// Pi 0.81 session/project classification for non-blocking resume.
// A Pi UUID is temporal identity only; project authority still requires root evidence.

export type PiSessionProjectClassification =
  | "new_session_new_project"
  | "new_session_existing_project"
  | "resumed_session_resumed_project"
  | "resumed_session_recoverable_project"
  | "resumed_session_worktree_rebound"
  | "session_project_mismatch"
  | "forked_compacted_continuation";

export interface PiSessionProjectClassificationInput {
  reason: "startup" | "reload" | "new" | "resume" | "fork";
  currentProjectRoot: string;
  markerExists: boolean;
  persistedStateFound: boolean;
  persistedProjectRoot?: string;
  bindingAmbiguous?: boolean;
  sameCanonicalProject?: boolean;
  bindingCandidateRoots?: string[];
  explicitContinuationMetadata?: boolean;
}

function normalizedRoot(value: string | undefined): string {
  const root = String(value || "")
    .trim()
    .replace(/\/+$/, "");
  return root === "" ? "" : root;
}

export function classifyPiSessionProject(
  input: PiSessionProjectClassificationInput
): PiSessionProjectClassification {
  const currentRoot = normalizedRoot(input.currentProjectRoot);
  const persistedRoot = normalizedRoot(input.persistedProjectRoot);

  if (input.bindingAmbiguous) return "session_project_mismatch";
  if (persistedRoot && currentRoot && persistedRoot !== currentRoot) {
    const candidateMatch = (input.bindingCandidateRoots || []).map(normalizedRoot).includes(persistedRoot);
    if (input.sameCanonicalProject || candidateMatch) {
      return "resumed_session_worktree_rebound";
    }
    return "session_project_mismatch";
  }
  if (input.reason === "fork" || input.explicitContinuationMetadata) {
    return "forked_compacted_continuation";
  }
  if (input.persistedStateFound) {
    return input.markerExists ? "resumed_session_resumed_project" : "resumed_session_recoverable_project";
  }
  return input.markerExists ? "new_session_existing_project" : "new_session_new_project";
}

export function persistedProjectRootFromState(state: any): string {
  return normalizedRoot(
    state?.lastProjectIdentity?.project_root ||
      state?.projectRootResolution?.project_root ||
      state?.activeWorkpointPacket?.scope?.project_root ||
      state?.activeWorkpointPacket?.project_root
  );
}
