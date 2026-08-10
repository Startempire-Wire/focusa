// Native coding-agent session pressure policy for Spec 130 §§44–48.
import { statSync } from "fs";
import { getHeapStatistics } from "v8";
const MIB = 1024 * 1024;
const SAMPLE_LIMIT = 2_048;
function boundedFraction(heapLimit, absolute, fraction) {
    return Math.max(1, Math.floor(Math.min(absolute, heapLimit * fraction)));
}
export function nativeSessionBudgets(heapLimitBytes) {
    const heapLimit = Math.max(64 * MIB, heapLimitBytes || 0);
    return {
        focusa_custom_soft_bytes: boundedFraction(heapLimit, 8 * MIB, 0.005),
        focusa_custom_hard_bytes: boundedFraction(heapLimit, 16 * MIB, 0.01),
        native_segment_soft_bytes: boundedFraction(heapLimit, 64 * MIB, 0.05),
        native_segment_hard_bytes: boundedFraction(heapLimit, 128 * MIB, 0.1),
        native_startup_migration_bytes: boundedFraction(heapLimit, 256 * MIB, 0.2),
        soft_headroom_floor: 0.35,
        hard_headroom_floor: 0.2,
        emergency_headroom_floor: 0.1,
    };
}
function entryCustomType(entry) {
    return String(entry?.customType || entry?.custom_type || "");
}
function entryData(entry) {
    return entry?.data ?? entry?.details ?? null;
}
function entryBytes(entry) {
    try {
        return Buffer.byteLength(JSON.stringify(entry), "utf8");
    }
    catch {
        return 0;
    }
}
export function measureNativeSessionPressure(input) {
    const heapStats = getHeapStatistics();
    const heapUsed = Math.max(0, input.heapUsedBytes ?? process.memoryUsage().heapUsed);
    const heapLimit = Math.max(1, input.heapLimitBytes ?? heapStats.heap_size_limit);
    const budgets = nativeSessionBudgets(heapLimit);
    let sessionBytes = 0;
    if (input.sessionFile) {
        try {
            sessionBytes = statSync(input.sessionFile).size;
        }
        catch {
            sessionBytes = 0;
        }
    }
    const entries = Array.isArray(input.entries) ? input.entries : [];
    const sample = entries.slice(-SAMPLE_LIMIT);
    let focusaCustomBytes = 0;
    let focusaCustomEntries = 0;
    let duplicateAnchorCount = 0;
    const anchorDigests = new Set();
    for (const entry of sample) {
        const customType = entryCustomType(entry);
        if (!customType.startsWith("focusa-"))
            continue;
        focusaCustomEntries += 1;
        focusaCustomBytes += entryBytes(entry);
        const digest = String(entryData(entry)?.semanticDigest || "");
        if (digest) {
            if (anchorDigests.has(digest))
                duplicateAnchorCount += 1;
            else
                anchorDigests.add(digest);
        }
    }
    const headroomRatio = Math.max(0, Math.min(1, (heapLimit - heapUsed) / heapLimit));
    let posture = "normal";
    let recommendedAction = "continue";
    if (sessionBytes >= budgets.native_startup_migration_bytes) {
        posture = "oversized_at_start";
        recommendedAction = "refuse_full_load";
    }
    else if (headroomRatio <= budgets.emergency_headroom_floor) {
        posture = "emergency";
        recommendedAction = "stream_migrate";
    }
    else if (sessionBytes >= budgets.native_segment_hard_bytes ||
        focusaCustomBytes >= budgets.focusa_custom_hard_bytes ||
        headroomRatio <= budgets.hard_headroom_floor) {
        posture = "hard_pressure";
        recommendedAction = "rollover";
    }
    else if (sessionBytes >= budgets.native_segment_soft_bytes ||
        focusaCustomBytes >= budgets.focusa_custom_soft_bytes ||
        headroomRatio <= budgets.soft_headroom_floor) {
        posture = "soft_pressure";
        recommendedAction = "checkpoint";
    }
    return {
        schema: "focusa.native_session_pressure.v1",
        adapter: input.adapter || "unknown",
        native_session_ref: input.sessionFile || "unavailable",
        session_bytes: sessionBytes,
        entry_count: entries.length,
        sampled_entry_count: sample.length,
        sample_complete: entries.length <= SAMPLE_LIMIT,
        focusa_custom_bytes: focusaCustomBytes,
        focusa_custom_entries: focusaCustomEntries,
        duplicate_anchor_count: duplicateAnchorCount,
        heap_used_bytes: heapUsed,
        heap_limit_bytes: heapLimit,
        headroom_ratio: headroomRatio,
        budgets,
        posture,
        recommended_action: recommendedAction,
        measured_at: (input.measuredAt || new Date()).toISOString(),
    };
}
const DEFAULT_RECOVERY_MAX_BYTES = 8 * MIB;
const DEFAULT_ENTRY_MAX_BYTES = 256 * 1024;
function safeMigrationBaseName(sourcePath) {
    const name = sourcePath.split(/[\\/]/).filter(Boolean).at(-1) || "session.jsonl";
    return name.replace(/[^A-Za-z0-9._-]/g, "_");
}
async function scanSessionJsonlBounded(sourcePath, recoveryMaxBytes, entryMaxBytes) {
    const { createHash } = await import("crypto");
    const { createReadStream } = await import("fs");
    const sourceHash = createHash("sha256");
    const recoveryEntries = [];
    let recoveryBytes = 0;
    let sourceBytes = 0;
    let entryCount = 0;
    let omittedOversizedEntries = 0;
    let lineBytes = 0;
    let lineParts = [];
    let lineOversized = false;
    let lineHash = createHash("sha256");
    const pushRecovery = (entry) => {
        if (entry.length > recoveryMaxBytes)
            return;
        while (recoveryBytes + entry.length > recoveryMaxBytes && recoveryEntries.length) {
            recoveryBytes -= recoveryEntries.shift().length;
        }
        recoveryEntries.push(entry);
        recoveryBytes += entry.length;
    };
    const finishLine = () => {
        if (lineBytes === 0 && lineParts.length === 0 && !lineOversized)
            return;
        entryCount += 1;
        if (lineOversized) {
            omittedOversizedEntries += 1;
            const ref = Buffer.from(`${JSON.stringify({
                type: "focusa-migration-omitted-entry",
                schema: "focusa.native_session_omitted_entry.v1",
                bytes: lineBytes,
                sha256: `sha256:${lineHash.digest("hex")}`,
                reason: "entry_exceeds_migration_budget",
            })}\n`);
            pushRecovery(ref);
        }
        else {
            pushRecovery(Buffer.concat([...lineParts, Buffer.from("\n")]));
            lineHash.digest();
        }
        lineBytes = 0;
        lineParts = [];
        lineOversized = false;
        lineHash = createHash("sha256");
    };
    for await (const chunkValue of createReadStream(sourcePath, { highWaterMark: 64 * 1024 })) {
        const chunk = Buffer.isBuffer(chunkValue) ? chunkValue : Buffer.from(chunkValue);
        sourceHash.update(chunk);
        sourceBytes += chunk.length;
        let offset = 0;
        while (offset < chunk.length) {
            const newline = chunk.indexOf(0x0a, offset);
            const end = newline === -1 ? chunk.length : newline;
            const part = chunk.subarray(offset, end);
            lineHash.update(part);
            lineBytes += part.length;
            if (!lineOversized && lineBytes <= entryMaxBytes)
                lineParts.push(Buffer.from(part));
            else if (!lineOversized) {
                lineOversized = true;
                lineParts = [];
            }
            if (newline !== -1)
                finishLine();
            offset = newline === -1 ? chunk.length : newline + 1;
        }
    }
    finishLine();
    return {
        source_sha256: `sha256:${sourceHash.digest("hex")}`,
        source_bytes: sourceBytes,
        entry_count: entryCount,
        omitted_oversized_entries: omittedOversizedEntries,
        recovery_entries: recoveryEntries,
        recovery_bytes: recoveryBytes,
    };
}
async function fsyncFileAndParent(path) {
    const { closeSync, fsyncSync, openSync } = await import("fs");
    const { dirname } = await import("path");
    for (const target of [path, dirname(path)]) {
        const descriptor = openSync(target, "r");
        try {
            fsyncSync(descriptor);
        }
        finally {
            closeSync(descriptor);
        }
    }
}
async function commitTemporaryFile(temporary, target) {
    const { linkSync, unlinkSync } = await import("fs");
    await fsyncFileAndParent(temporary);
    linkSync(temporary, target);
    await fsyncFileAndParent(target);
    unlinkSync(temporary);
    const { dirname } = await import("path");
    const { closeSync, fsyncSync, openSync } = await import("fs");
    const descriptor = openSync(dirname(target), "r");
    try {
        fsyncSync(descriptor);
    }
    finally {
        closeSync(descriptor);
    }
}
function injectMigrationFault(request, step) {
    if (request.fault_injection_step === step) {
        throw new Error(`native_session_migration_fault:${step}`);
    }
}
async function writeBuffersAtomic(path, entries) {
    const { createWriteStream, unlinkSync } = await import("fs");
    const { once } = await import("events");
    const temporary = `${path}.tmp-${process.pid}`;
    const stream = createWriteStream(temporary, { mode: 0o600, flags: "wx" });
    try {
        for (const entry of entries) {
            if (!stream.write(entry))
                await once(stream, "drain");
        }
        stream.end();
        await once(stream, "close");
        await commitTemporaryFile(temporary, path);
    }
    catch (error) {
        stream.destroy();
        try {
            unlinkSync(temporary);
        }
        catch {
            // Source remains immutable; cleanup is best effort only.
        }
        throw error;
    }
}
async function copyFileStreaming(source, target) {
    const { createReadStream, createWriteStream, unlinkSync } = await import("fs");
    const { pipeline } = await import("stream/promises");
    const temporary = `${target}.tmp-${process.pid}`;
    try {
        await pipeline(createReadStream(source, { highWaterMark: 64 * 1024 }), createWriteStream(temporary, { flags: "wx", mode: 0o600 }));
        await commitTemporaryFile(temporary, target);
    }
    catch (error) {
        try {
            unlinkSync(temporary);
        }
        catch {
            // Source remains immutable; cleanup is best effort only.
        }
        throw error;
    }
}
export async function migrateNativeSessionBounded(request) {
    const { chmodSync, linkSync, mkdirSync, statSync, writeFileSync, unlinkSync } = await import("fs");
    const { join } = await import("path");
    const sourceBefore = statSync(request.source_path);
    if (!sourceBefore.isFile())
        throw new Error("native_session_source_not_file");
    const recoveryMaxBytes = Math.max(64 * 1024, request.recovery_max_bytes || DEFAULT_RECOVERY_MAX_BYTES);
    const entryMaxBytes = Math.max(16 * 1024, request.entry_max_bytes || DEFAULT_ENTRY_MAX_BYTES);
    const scan = await scanSessionJsonlBounded(request.source_path, recoveryMaxBytes, entryMaxBytes);
    const digestId = scan.source_sha256.slice(7, 23);
    const migrationId = `native-session-${digestId}`;
    const base = safeMigrationBaseName(request.source_path);
    const archivePath = join(request.output_dir, `${base}.${digestId}.immutable.jsonl`);
    const recoveryPath = join(request.output_dir, `${base}.${digestId}.recovery.jsonl`);
    const manifestPath = join(request.output_dir, `${base}.${digestId}.manifest.json`);
    const baseManifest = {
        schema: "focusa.native_session_migration_manifest.v1",
        migration_id: migrationId,
        mode: request.mode,
        scope: request.scope,
        source: {
            path: request.source_path,
            bytes: scan.source_bytes,
            sha256: scan.source_sha256,
            mtime_ms: sourceBefore.mtimeMs,
        },
        archive: null,
        recovery_segment: null,
        manifest_path: null,
        integrity: {
            source_unchanged: true,
            archive_matches_source: false,
            recovery_within_budget: scan.recovery_bytes <= recoveryMaxBytes,
        },
        rollback: {
            action: "resume_immutable_source",
            source_path: request.source_path,
            source_sha256: scan.source_sha256,
        },
    };
    if (request.mode === "dry_run")
        return baseManifest;
    mkdirSync(request.output_dir, { recursive: true, mode: 0o700 });
    const createdFiles = [];
    const temporaryFiles = [archivePath, recoveryPath, manifestPath].map((path) => `${path}.tmp-${process.pid}`);
    try {
        injectMigrationFault(request, "after_prepare");
        await copyFileStreaming(request.source_path, archivePath);
        createdFiles.push(archivePath);
        injectMigrationFault(request, "after_archive_write");
        const archiveScan = await scanSessionJsonlBounded(archivePath, 64 * 1024, entryMaxBytes);
        if (archiveScan.source_sha256 !== scan.source_sha256 || archiveScan.source_bytes !== scan.source_bytes)
            throw new Error("native_session_archive_integrity_mismatch");
        injectMigrationFault(request, "after_archive_checksum");
        chmodSync(archivePath, 0o400);
        injectMigrationFault(request, "after_archive_seal");
        await writeBuffersAtomic(recoveryPath, scan.recovery_entries);
        createdFiles.push(recoveryPath);
        injectMigrationFault(request, "after_recovery_write");
        const recoveryScan = await scanSessionJsonlBounded(recoveryPath, recoveryMaxBytes, entryMaxBytes);
        injectMigrationFault(request, "after_recovery_checksum");
        const sourceAfter = statSync(request.source_path);
        const sourceAfterScan = await scanSessionJsonlBounded(request.source_path, 64 * 1024, entryMaxBytes);
        const sourceUnchanged = sourceAfter.size === sourceBefore.size &&
            sourceAfter.mtimeMs === sourceBefore.mtimeMs &&
            sourceAfterScan.source_sha256 === scan.source_sha256;
        if (!sourceUnchanged)
            throw new Error("native_session_source_changed_during_migration");
        injectMigrationFault(request, "after_source_verify");
        const manifest = {
            ...baseManifest,
            archive: {
                path: archivePath,
                bytes: archiveScan.source_bytes,
                sha256: archiveScan.source_sha256,
                immutable: true,
            },
            recovery_segment: {
                path: recoveryPath,
                bytes: recoveryScan.source_bytes,
                sha256: recoveryScan.source_sha256,
                entry_count: recoveryScan.entry_count,
                omitted_oversized_entries: scan.omitted_oversized_entries,
            },
            manifest_path: manifestPath,
            integrity: {
                source_unchanged: sourceUnchanged,
                archive_matches_source: true,
                recovery_within_budget: recoveryScan.source_bytes <= recoveryMaxBytes,
            },
        };
        const temporaryManifest = `${manifestPath}.tmp-${process.pid}`;
        writeFileSync(temporaryManifest, `${JSON.stringify(manifest, null, 2)}\n`, {
            mode: 0o600,
            flag: "wx",
        });
        await fsyncFileAndParent(temporaryManifest);
        injectMigrationFault(request, "after_manifest_write");
        linkSync(temporaryManifest, manifestPath);
        await fsyncFileAndParent(manifestPath);
        createdFiles.push(manifestPath);
        injectMigrationFault(request, "after_manifest_commit");
        unlinkSync(temporaryManifest);
        return manifest;
    }
    catch (error) {
        for (const path of createdFiles.reverse()) {
            try {
                chmodSync(path, 0o600);
                unlinkSync(path);
            }
            catch {
                // Preserve the immutable source; orphan cleanup is bounded and best effort.
            }
        }
        for (const path of temporaryFiles) {
            try {
                unlinkSync(path);
            }
            catch {
                // Preserve the immutable source; orphan cleanup is bounded and best effort.
            }
        }
        throw error;
    }
}
