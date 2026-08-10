// Bounded native-session persistence for Spec 130 §§38–54.
// This module is deliberately independent of mutable Pi state so hashing,
// sidecar integrity, retention, and replay can be tested in isolation.
import { closeSync, fsyncSync, mkdirSync, openSync, readFileSync, readdirSync, renameSync, unlinkSync, writeFileSync, } from "fs";
import { createHash } from "crypto";
import { homedir } from "os";
import { join } from "path";
export const COMPACTION_PERSISTENCE_ANCHOR_SCHEMA = "focusa.compaction_persistence_anchor.v1";
export const COMPACTION_PERSISTENCE_ANCHOR_REF_SCHEMA = "focusa.compaction_persistence_anchor_ref.v1";
export const COMPACTION_PERSISTENCE_SIDECAR_SCHEMA = "focusa.compaction_persistence_sidecar.v1";
export const NATIVE_ANCHOR_MAX_BYTES = 8 * 1024;
export const PROJECT_SWITCH_ANCHOR_MAX_BYTES = 2 * 1024;
export const PERSISTENCE_SIDECAR_MAX_BYTES = 256 * 1024;
export const PERSISTENCE_SIDECAR_GENERATIONS = 3;
const PERSISTENCE_SIDECAR_DIR = "pi-session-state";
function injectPersistenceFault(boundary, requested) {
    if (requested === boundary)
        throw new Error(`injected persistence fault: ${boundary}`);
}
export function stableSemanticValue(value, key = "") {
    if (value == null || typeof value !== "object")
        return value;
    if (Array.isArray(value) && key !== "projectSwitchLedger")
        return value.map((item) => stableSemanticValue(item));
    if (key === "projectSwitchLedger" && Array.isArray(value)) {
        return value
            .map((entry) => ({
            project_alias: entry.project_alias,
            project_root: entry.project_root,
            remote_host: entry.remote_host || null,
            confidence_class: entry.confidence >= 0.8 ? "high" : entry.confidence >= 0.55 ? "medium" : "low",
            source: entry.source,
            relationship_kind: entry.relationship_kind || null,
            primary_scope_ref: entry.primary_scope_ref || null,
            supporting_scope_ref: entry.supporting_scope_ref || null,
            operator_confirmed_scope_switch: entry.operator_confirmed_scope_switch === true,
            action_authority_transfers: entry.action_authority_transfers === true,
        }))
            .sort((a, b) => `${a.project_root}\0${a.project_alias}`.localeCompare(`${b.project_root}\0${b.project_alias}`));
    }
    if (key === "toolOutputPressure") {
        return {
            recapRequired: value.recapRequired === true,
            recapReason: String(value.recapReason || ""),
            lastToolName: String(value.lastToolName || ""),
        };
    }
    const volatile = new Set([
        "timestamp",
        "updatedAt",
        "updated_at",
        "capturedAt",
        "captured_at",
        "refreshed_at",
        "verifiedAt",
        "verified_at",
        "measured_at",
        "lastEventAt",
        "lastRecapAt",
        "lastCompactResumeAt",
        "turnCount",
        "totalCompactions",
        "vitalInfoPrompted",
        "first_seen_turn",
        "last_seen_turn",
    ]);
    const output = {};
    for (const childKey of Object.keys(value).sort()) {
        if (volatile.has(childKey))
            continue;
        output[childKey] = stableSemanticValue(value[childKey], childKey);
    }
    return output;
}
function sha256Text(value) {
    return `sha256:${createHash("sha256").update(value).digest("hex")}`;
}
export function semanticPersistenceDigest(value) {
    return sha256Text(JSON.stringify(stableSemanticValue(value)));
}
function persistenceRoot() {
    const dataRoot = String(process.env.FOCUSA_DATA_DIR || "").trim() || join(homedir(), ".focusa");
    return join(dataRoot, PERSISTENCE_SIDECAR_DIR);
}
function sidecarKey(projectRoot, sessionId) {
    return createHash("sha256").update(`${projectRoot}\0${sessionId}`).digest("hex");
}
function validateSidecarKey(key) {
    if (!/^[a-f0-9]{64}$/.test(key))
        throw new Error("invalid persistence sidecar key");
}
function sidecarPathForRevision(key, revision, semanticDigest) {
    validateSidecarKey(key);
    const digestSuffix = semanticDigest.replace(/^sha256:/, "").slice(0, 16);
    return join(persistenceRoot(), `${key}.r${revision}.${digestSuffix}.json`);
}
function sidecarCandidates(key) {
    validateSidecarKey(key);
    const prefix = `${key}.r`;
    try {
        return readdirSync(persistenceRoot())
            .filter((name) => name.startsWith(prefix) && name.endsWith(".json"))
            .sort((a, b) => {
            const revisionA = Number(a.slice(prefix.length).split(".", 1)[0] || 0);
            const revisionB = Number(b.slice(prefix.length).split(".", 1)[0] || 0);
            return revisionB - revisionA;
        })
            .map((name) => join(persistenceRoot(), name));
    }
    catch {
        return [];
    }
}
export function writeRecoverySidecar(recoveryState, semanticDigest, revision, faultAt) {
    injectPersistenceFault("prepare", faultAt);
    const projectRoot = String(recoveryState.projectRoot || "").trim();
    const sessionId = String(recoveryState.sessionId || "").trim();
    if (!projectRoot)
        throw new Error("project root required for persistence sidecar");
    if (!sessionId)
        throw new Error("session id required for persistence sidecar");
    const key = sidecarKey(projectRoot, sessionId);
    const directory = persistenceRoot();
    mkdirSync(directory, { recursive: true, mode: 0o700 });
    const target = sidecarPathForRevision(key, revision, semanticDigest);
    injectPersistenceFault("manifest", faultAt);
    const envelope = {
        schema: COMPACTION_PERSISTENCE_SIDECAR_SCHEMA,
        revision,
        semanticDigest,
        recoveryState,
        updatedAt: new Date().toISOString(),
    };
    const serialized = `${JSON.stringify(envelope)}\n`;
    injectPersistenceFault("checksum", faultAt);
    const bytes = Buffer.byteLength(serialized, "utf8");
    if (bytes > PERSISTENCE_SIDECAR_MAX_BYTES) {
        throw new Error(`persistence sidecar exceeds ${PERSISTENCE_SIDECAR_MAX_BYTES} bytes`);
    }
    const temporary = `${target}.tmp-${process.pid}-${Date.now()}`;
    let descriptor = null;
    try {
        injectPersistenceFault("target-create", faultAt);
        descriptor = openSync(temporary, "w", 0o600);
        writeFileSync(descriptor, serialized, "utf8");
        injectPersistenceFault("write", faultAt);
        fsyncSync(descriptor);
        injectPersistenceFault("fsync", faultAt);
        closeSync(descriptor);
        descriptor = null;
        renameSync(temporary, target);
        injectPersistenceFault("resume-verify", faultAt);
        const committed = JSON.parse(readFileSync(target, "utf8"));
        if (committed.semanticDigest !== semanticDigest || committed.revision !== revision) {
            throw new Error("persistence commit verification failed");
        }
        injectPersistenceFault("commit", faultAt);
        for (const stale of sidecarCandidates(key).slice(PERSISTENCE_SIDECAR_GENERATIONS)) {
            try {
                unlinkSync(stale);
            }
            catch {
                // Retention cleanup must not invalidate the newly published generation.
            }
        }
    }
    catch (error) {
        if (descriptor != null) {
            try {
                closeSync(descriptor);
            }
            catch {
                // Best effort after a failed sidecar write.
            }
        }
        try {
            unlinkSync(temporary);
        }
        catch {
            // Best effort after a failed sidecar write.
        }
        throw error;
    }
    return { key, bytes };
}
export function loadPersistedRecoveryState(anchor, fallbackProjectRoot = "") {
    if (!anchor || typeof anchor !== "object")
        return null;
    if (anchor.schema !== COMPACTION_PERSISTENCE_ANCHOR_SCHEMA &&
        anchor.schema !== COMPACTION_PERSISTENCE_ANCHOR_REF_SCHEMA) {
        return anchor;
    }
    const key = String(anchor.sidecarKey || "");
    const projectRoot = String(anchor.projectRoot || fallbackProjectRoot || "").trim();
    const sessionId = String(anchor.sessionId || "").trim();
    if (!projectRoot || !sessionId || key !== sidecarKey(projectRoot, sessionId))
        return null;
    for (const candidate of sidecarCandidates(key)) {
        try {
            const parsed = JSON.parse(readFileSync(candidate, "utf8"));
            if (parsed?.schema !== COMPACTION_PERSISTENCE_SIDECAR_SCHEMA)
                continue;
            const recoveryState = parsed.recoveryState;
            if (!recoveryState || typeof recoveryState !== "object")
                continue;
            if (semanticPersistenceDigest(recoveryState) !== parsed.semanticDigest)
                continue;
            if (String(recoveryState.sessionId || "") !== sessionId)
                continue;
            return recoveryState;
        }
        catch {
            continue;
        }
    }
    return null;
}
