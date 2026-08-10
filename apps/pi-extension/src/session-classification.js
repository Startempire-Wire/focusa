// Pi 0.81 session/project classification for non-blocking resume.
// A Pi UUID is temporal identity only; project authority still requires root evidence.
function normalizedRoot(value) {
    const root = String(value || "")
        .trim()
        .replace(/\/+$/, "");
    return root === "" ? "" : root;
}
export function classifyPiSessionProject(input) {
    const currentRoot = normalizedRoot(input.currentProjectRoot);
    const persistedRoot = normalizedRoot(input.persistedProjectRoot);
    if (input.bindingAmbiguous)
        return "session_project_mismatch";
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
export function persistedProjectRootFromState(state) {
    return normalizedRoot(state?.lastProjectIdentity?.project_root ||
        state?.projectRootResolution?.project_root ||
        state?.activeWorkpointPacket?.scope?.project_root ||
        state?.activeWorkpointPacket?.project_root);
}
