import { createHash } from "node:crypto";
import type { MissionCanvasApiClient } from "./api-client.js";
import { lifecycleState } from "./api-client.js";
import {
  NativeProcessAdapter,
  probeRichHost,
  removeHandshakeFile,
  resolveRichHostRenderer,
  writeHandshakeFile,
} from "./platform.js";
import type {
  MissionCanvasScope,
  RichHostLaunchRequest,
  RichHostLifecycleState,
  RichHostProcessAdapter,
  RichHostProcessHandle,
} from "./types.js";

interface HostRecord {
  request: RichHostLaunchRequest;
  client: MissionCanvasApiClient;
  state: RichHostLifecycleState;
  handle: RichHostProcessHandle;
  handshakePath: string;
  heartbeat?: NodeJS.Timeout;
}

export class RichHostLifecycleManager {
  private readonly hosts = new Map<string, HostRecord>();

  constructor(private readonly adapter: RichHostProcessAdapter = new NativeProcessAdapter()) {}

  async on(request: RichHostLaunchRequest, client: MissionCanvasApiClient): Promise<RichHostLifecycleState> {
    const key = attachmentKey(request.scope);
    const existing = this.hosts.get(key);
    if (existing && (await this.adapter.isAlive(existing.handle))) {
      await this.adapter.focus(existing.handle);
      existing.state = nextState(existing.state, "focused", existing.handle);
      await client.updateHostLifecycle("focus", existing.state, existing.state.lifecycle_revision - 1);
      return existing.state;
    }
    if (existing) await this.disposeRecord(key, existing, false);
    await client.ensureProjection();
    const probe = await probeRichHost(request.package_root);
    const resolution = resolveRichHostRenderer(request.interaction_mode, probe, request.asset_version);
    const handshake = await writeHandshakeFile(request);
    resolution.asset_digest = `sha256:${handshake.digest}`;
    let state = lifecycleState(request.scope, "launching", 1, resolution);
    const handle = await this.adapter.launch(request, resolution, handshake.path);
    state = nextState(state, resolution.selected_renderer === "headless_none" ? "hidden" : "focused", handle);
    const record: HostRecord = { request, client, state, handle, handshakePath: handshake.path };
    this.hosts.set(key, record);
    this.startHeartbeat(key, record);
    await client.updateHostLifecycle("launch", state, undefined);
    setTimeout(() => void removeHandshakeFile(handshake.path), 60_000).unref?.();
    return state;
  }

  async off(scope: MissionCanvasScope, close = false): Promise<RichHostLifecycleState | undefined> {
    const key = attachmentKey(scope);
    const record = this.hosts.get(key);
    if (!record) return undefined;
    if (close) {
      record.state = nextState(record.state, "closing", record.handle);
      await record.client.updateHostLifecycle("close", record.state, record.state.lifecycle_revision - 1);
      await this.disposeRecord(key, record, true);
      return record.state;
    }
    await this.adapter.hide(record.handle);
    record.state = nextState(record.state, "hidden", record.handle);
    await record.client.updateHostLifecycle("hide", record.state, record.state.lifecycle_revision - 1);
    return record.state;
  }

  async reconnect(scope: MissionCanvasScope): Promise<RichHostLifecycleState | undefined> {
    const record = this.hosts.get(attachmentKey(scope));
    if (!record) return undefined;
    record.state = nextState(record.state, "reconnecting", record.handle);
    try {
      await record.client.getProjection();
      record.state = nextState(record.state, "focused", record.handle);
      return record.state;
    } catch (error) {
      record.state = {
        ...nextState(record.state, "failed", record.handle),
        last_error_ref: `error:${createHash("sha256").update(String(error)).digest("hex").slice(0, 16)}`,
      };
      return record.state;
    }
  }

  state(scope: MissionCanvasScope): RichHostLifecycleState | undefined {
    return this.hosts.get(attachmentKey(scope))?.state;
  }

  async appendPiEvent(
    scope: MissionCanvasScope,
    eventKind: "pi_turn_started" | "pi_turn_completed" | "pi_message_updated" | "pi_tool_started" | "pi_tool_completed",
    payload: unknown,
    eventId: string
  ): Promise<void> {
    const record = this.hosts.get(attachmentKey(scope));
    if (!record) return;
    await record.client.appendPiSessionEvent(eventKind, payload, eventId);
  }

  async shutdown(): Promise<void> {
    for (const [key, record] of [...this.hosts]) await this.disposeRecord(key, record, true);
  }

  private startHeartbeat(key: string, record: HostRecord): void {
    record.heartbeat = setInterval(async () => {
      if (!(await this.adapter.isAlive(record.handle))) {
        record.state = nextState(record.state, "failed", record.handle);
        if (record.heartbeat) clearInterval(record.heartbeat);
        return;
      }
      try {
        await record.client.events();
        record.state.durable_event_cursor = record.client.durableEventCursor();
      } catch {
        await this.reconnect(record.request.scope);
      }
    }, 5_000);
    record.heartbeat.unref?.();
    this.hosts.set(key, record);
  }

  private async disposeRecord(key: string, record: HostRecord, close: boolean): Promise<void> {
    if (record.heartbeat) clearInterval(record.heartbeat);
    if (close) await this.adapter.close(record.handle);
    await removeHandshakeFile(record.handshakePath);
    this.hosts.delete(key);
  }
}

function nextState(
  current: RichHostLifecycleState,
  state: RichHostLifecycleState["state"],
  handle: RichHostProcessHandle
): RichHostLifecycleState {
  return {
    ...current,
    state,
    process_id: handle.process_id ?? null,
    window_id: handle.window_id,
    focused: state === "focused",
    lifecycle_revision: current.lifecycle_revision + 1,
    updated_at: new Date().toISOString(),
  };
}

export function attachmentKey(scope: MissionCanvasScope): string {
  return JSON.stringify([scope.project_root, scope.continuity_id, scope.session_id, scope.attachment_id]);
}
