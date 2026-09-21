import { isAbsolute, normalize, resolve, sep } from "node:path";

export type WorkpointMutationDecision = {
  applicable: boolean;
  block: boolean;
  attemptedPath?: string;
  workpointId?: string;
  checkpointRef?: string;
  targetObjects: string[];
  doNotDrift: string[];
  reason: string;
};

function strings(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === "string" && item.trim().length > 0)
    : [];
}

function enforcementMode(packet: Record<string, any> | null, env: NodeJS.ProcessEnv): string {
  const packetMode =
    packet?.mutation_enforcement ??
    packet?.enforcement?.mutation ??
    packet?.checkpoint?.mutation_enforcement;
  return String(packetMode ?? env.FOCUSA_WORKPOINT_MUTATION_ENFORCEMENT ?? "advisory")
    .trim()
    .toLowerCase();
}

function packetList(packet: Record<string, any>, key: "target_objects" | "do_not_drift"): string[] {
  const candidates = [
    packet?.[key],
    packet?.checkpoint?.[key],
    packet?.resume_packet?.[key],
    packet?.workpoint?.[key],
    packet?.data?.[key],
  ];
  return candidates.flatMap(strings).filter((item, index, values) => values.indexOf(item) === index);
}

function pathLike(value: string): boolean {
  if (/^[a-z][a-z0-9+.-]*:/i.test(value) && !/^[a-z]:[\\/]/i.test(value)) return false;
  return value.includes("/") || value.includes("\\") || value.startsWith(".");
}

function canonicalPath(value: string, projectRoot: string): string {
  const resolved = isAbsolute(value) ? value : resolve(projectRoot, value);
  return normalize(resolved);
}

function targetAllowsPath(target: string, attemptedPath: string, projectRoot: string): boolean {
  if (!pathLike(target)) return false;
  const directoryTarget = /[\\/]$/.test(target) || target.endsWith("/**");
  const cleanTarget = target.endsWith("/**") ? target.slice(0, -3) : target;
  const canonicalTarget = canonicalPath(cleanTarget, projectRoot);
  if (attemptedPath === canonicalTarget) return true;
  return directoryTarget && attemptedPath.startsWith(`${canonicalTarget}${sep}`);
}

export function evaluateWorkpointMutation(input: {
  toolName: string;
  toolInput: Record<string, unknown>;
  packet: Record<string, any> | null;
  cwd: string;
  env?: NodeJS.ProcessEnv;
}): WorkpointMutationDecision {
  const { toolName, toolInput, packet, cwd } = input;
  if (toolName !== "write" && toolName !== "edit") {
    return {
      applicable: false,
      block: false,
      targetObjects: [],
      doNotDrift: [],
      reason: "tool is not a file mutation",
    };
  }

  const rawPath = typeof toolInput.path === "string" ? toolInput.path.trim() : "";
  const targetObjects = packet ? packetList(packet, "target_objects") : [];
  const doNotDrift = packet ? packetList(packet, "do_not_drift") : [];
  const workpointId = String(packet?.workpoint_id ?? packet?.id ?? "").trim() || undefined;
  const checkpointRef =
    String(packet?.checkpoint_ref ?? packet?.checkpoint?.checkpoint_ref ?? "").trim() || undefined;
  const mode = enforcementMode(packet, input.env ?? process.env);
  const enabled = mode === "block" || mode === "enforce" || mode === "strict";

  if (!enabled) {
    return {
      applicable: true,
      block: false,
      workpointId,
      checkpointRef,
      targetObjects,
      doNotDrift,
      reason: "workpoint mutation enforcement is advisory",
    };
  }

  if (!packet || !workpointId) {
    return {
      applicable: true,
      block: true,
      targetObjects,
      doNotDrift,
      reason: "strict mutation enforcement requires an active canonical Workpoint",
    };
  }

  if (!rawPath) {
    return {
      applicable: true,
      block: true,
      workpointId,
      checkpointRef,
      targetObjects,
      doNotDrift,
      reason: "file mutation omitted its target path",
    };
  }

  const projectRoot = String(packet.project_root ?? packet.scope?.project_root ?? cwd).trim() || cwd;
  const attemptedPath = canonicalPath(rawPath, projectRoot);
  const allowed = targetObjects.some((target) => targetAllowsPath(target, attemptedPath, projectRoot));
  return {
    applicable: true,
    block: !allowed,
    attemptedPath,
    workpointId,
    checkpointRef,
    targetObjects,
    doNotDrift,
    reason: allowed
      ? "file mutation matches an active Workpoint target"
      : targetObjects.length === 0
        ? "strict mutation enforcement requires at least one file target"
        : "file mutation is outside the active Workpoint target_objects allowlist",
  };
}
