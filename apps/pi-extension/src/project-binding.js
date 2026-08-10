import { createHash } from "node:crypto";
function boundedText(value, max = 512) {
    return String(value || "")
        .trim()
        .slice(0, max);
}
function normalizedRoot(value) {
    const root = boundedText(value, 4096).replace(/\/+$/, "");
    return root === "/" ? "/" : root;
}
function sortedUnique(values) {
    return [...new Set(values.map((value) => boundedText(value, 256)).filter(Boolean))].sort();
}
function normalizedCandidates(values) {
    return values
        .map((candidate) => ({
        project_root: normalizedRoot(candidate.project_root),
        active_worktree_root: normalizedRoot(candidate.active_worktree_root) || undefined,
        canonical_parent_root: normalizedRoot(candidate.canonical_parent_root) || undefined,
        score: Number.isFinite(Number(candidate.score)) ? Math.max(0, Number(candidate.score)) : 0,
        sources: sortedUnique(Array.isArray(candidate.sources) ? candidate.sources : []),
        markers: sortedUnique(Array.isArray(candidate.markers) ? candidate.markers : []),
        relationship: boundedText(candidate.relationship, 80) || undefined,
        repo_fingerprint: boundedText(candidate.repo_fingerprint, 256) || undefined,
        project_fingerprint: boundedText(candidate.project_fingerprint, 256) || undefined,
    }))
        .filter((candidate) => candidate.project_root)
        .sort((left, right) => right.score - left.score || left.project_root.localeCompare(right.project_root))
        .slice(0, 16);
}
function stableHash(value) {
    return createHash("sha256").update(JSON.stringify(value)).digest("hex");
}
function confidenceFor(candidates, state) {
    if (state === "QUARANTINED" || !candidates.length)
        return "none";
    const score = candidates[0]?.score || 0;
    return score >= 900 ? "high" : score >= 700 ? "medium" : "low";
}
export function reconcileProjectBindingDecision(input) {
    const candidates = normalizedCandidates(input.candidates || []);
    const selectedProjectRoot = normalizedRoot(input.selectedProjectRoot) || undefined;
    const selectedWorktreeRoot = normalizedRoot(input.selectedWorktreeRoot) || undefined;
    const canonicalParentRoot = normalizedRoot(input.canonicalParentRoot) || undefined;
    const rejectionReasons = sortedUnique(input.rejectionReasons || []);
    const selectedRootSafe = input.selectedRootSafe === true;
    const verificationCanonical = input.verificationCanonical === true;
    const daemonAvailable = input.daemonAvailable !== false;
    let state;
    if (input.ambiguous || (selectedProjectRoot && !selectedRootSafe)) {
        state = "QUARANTINED";
        rejectionReasons.push(input.ambiguous ? "conflicting_strong_candidates" : "unsafe_selected_root");
    }
    else if (!selectedProjectRoot) {
        state = daemonAvailable ? "RECOVERING" : "RECOVERING";
        rejectionReasons.push("no_safe_selected_project_root");
    }
    else if (verificationCanonical) {
        state = "BOUND";
    }
    else if (!daemonAvailable || input.evidenceFreshness === "stale") {
        state = "RECOVERING";
        rejectionReasons.push(!daemonAvailable ? "daemon_unavailable" : "stale_binding_evidence");
    }
    else {
        state = "VERIFY";
        rejectionReasons.push("canonical_project_verification_required");
    }
    const evidenceSources = sortedUnique(candidates.flatMap((candidate) => candidate.sources));
    const evidenceRevision = stableHash({
        state,
        selectedProjectRoot,
        selectedWorktreeRoot,
        canonicalParentRoot,
        continuityId: boundedText(input.continuityId, 256),
        candidates,
        evidenceFreshness: input.evidenceFreshness || "unknown",
        repoFingerprint: boundedText(input.repoFingerprint, 256),
        projectFingerprint: boundedText(input.projectFingerprint, 256),
        verificationStatus: boundedText(input.verificationStatus, 120),
        rejectionReasons: sortedUnique(rejectionReasons),
    });
    const decisionId = `project-binding:${evidenceRevision.slice(0, 24)}`;
    const capability = state === "BOUND" ? "scoped" : state === "QUARANTINED" ? "unbound_read_only" : "recovery_read_plan";
    return {
        schema: "focusa.project_binding_decision.v1",
        decision_id: decisionId,
        state,
        selected_project_root: selectedProjectRoot,
        selected_worktree_root: selectedWorktreeRoot,
        canonical_parent_root: canonicalParentRoot,
        continuity_id: boundedText(input.continuityId, 256) || "unbound",
        candidates,
        evidence_sources: evidenceSources,
        evidence_freshness: input.evidenceFreshness || "unknown",
        evidence_revision: evidenceRevision,
        rejection_reasons: sortedUnique(rejectionReasons),
        scope_safety_policy_version: "focusa.scope_safety.v1",
        repo_fingerprint: boundedText(input.repoFingerprint, 256) || undefined,
        project_fingerprint: boundedText(input.projectFingerprint, 256) || undefined,
        confidence: confidenceFor(candidates, state),
        verification_status: boundedText(input.verificationStatus, 120) || "unknown",
        permitted_capability_tier: capability,
        recovery_packet_ref: boundedText(input.recoveryPacketRef, 256) || undefined,
        operator_decision_ref: boundedText(input.operatorDecisionRef, 256) || undefined,
        binding_receipt_id: state === "BOUND" ? `binding-receipt:${evidenceRevision.slice(0, 24)}` : undefined,
        effective_at: input.effectiveAt || new Date().toISOString(),
        supersedes_decision_id: input.previousDecision && input.previousDecision.decision_id !== decisionId
            ? input.previousDecision.decision_id
            : undefined,
    };
}
export function canReuseFreshVerifiedBindingOffline(decision, input) {
    if (!decision || decision.state !== "BOUND")
        return false;
    const selectedRoot = normalizedRoot(input.selectedProjectRoot);
    if (!selectedRoot || selectedRoot !== normalizedRoot(decision.selected_project_root))
        return false;
    const observedAt = Date.parse(decision.effective_at);
    const now = input.nowMs ?? Date.now();
    const maxAge = input.maxAgeMs ?? 15 * 60 * 1000;
    if (!Number.isFinite(observedAt) || now - observedAt < 0 || now - observedAt > maxAge)
        return false;
    const repoMatches = !!decision.repo_fingerprint &&
        !!input.repoFingerprint &&
        decision.repo_fingerprint === input.repoFingerprint;
    const projectMatches = !!decision.project_fingerprint &&
        !!input.projectFingerprint &&
        decision.project_fingerprint === input.projectFingerprint;
    return repoMatches || projectMatches;
}
export function projectBindingAllowsDurableWrites(decision) {
    return !decision || decision.state === "BOUND";
}
export function shouldEmitProjectScopeRecoveryPacket(previous, next) {
    if (next.state !== "RECOVERING" && next.state !== "QUARANTINED" && next.state !== "VERIFY")
        return false;
    return !previous || previous.evidence_revision !== next.evidence_revision;
}
