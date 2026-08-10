import { MissionCanvasClient, type MissionCanvasOperationInput } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
import type { ResolvedWorkspaceProjection, WorkstreamAuthorityContext } from './types';
import { MissionCanvasHttpTransport, MissionCanvasTransportError } from './http-transport';
import { validateMissionCanvasContract } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';

export interface LiveCanvasBinding {
  authority: WorkstreamAuthorityContext;
  client: MissionCanvasClient;
  projection: ResolvedWorkspaceProjection;
}

export interface LiveCanvasBindingResult {
  binding: LiveCanvasBinding | null;
  kind: 'live' | 'fixture' | 'unavailable';
  reason?: string;
}

const DEFAULT_DAEMON_URL = 'http://127.0.0.1:8787';

/**
 * Browser-preview live binding.  Builds the generated transport + client,
 * binds the exact Workstream authority, and resolves a canonical projection
 * through the daemon.  The daemon is the only composition authority — the
 * preview never invents candidates or layouts.  Falls back to the schema
 * fixture ONLY when the daemon is unreachable (honest degraded mode).
 */
export async function resolveLiveCanvasBinding(
  baseUrl = DEFAULT_DAEMON_URL,
  fetchImplementation: typeof fetch = fetch
): Promise<LiveCanvasBindingResult> {
  const transport = new MissionCanvasHttpTransport(
    baseUrl,
    fetchImplementation,
    undefined,
    15_000,
    ['mission_canvas:*'],
    ['mission_canvas:read', 'mission_canvas:write', 'pi_attachment:attach', 'uiai:read'],
    'desktop-preview',
    'authority:desktop-exact'
  );
  const client = new MissionCanvasClient(transport);

  const authority: WorkstreamAuthorityContext = {
    workstream: {
      scope: {
        scope_kind: 'project',
        scope_key: {
          scope_kind: 'project',
          scope_id: 'project:focusa',
          root_path: '/example/focusa',
          canonical_name: 'Focusa',
          fingerprint: 'host-a:worktree-main'
        }
      },
      workstream_id: 'ws:mission-canvas'
    },
    continuity_id: 'continuity:mission-canvas',
    attachment: {
      workstream: {
        scope: {
          scope_kind: 'project',
          scope_key: {
            scope_kind: 'project',
            scope_id: 'project:focusa',
            root_path: '/example/focusa',
            canonical_name: 'Focusa',
            fingerprint: 'host-a:worktree-main'
          }
        },
        workstream_id: 'ws:mission-canvas'
      },
      continuity_id: 'continuity:mission-canvas',
      instance_id: 'instance:pi',
      session_id: 'session:pi',
      attachment_id: 'attachment:pi',
      workspace_binding_id: 'workspace:mission-canvas'
    },
    workspace_binding_id: 'workspace:mission-canvas',
    runtime_object: { runtime_kind: 'pi_session', runtime_id: 'session:pi' },
    work_surface_id: 'surface:pi'
  };

  const validation = validateMissionCanvasContract('WorkstreamAuthorityContext', authority);
  if (!validation.valid) {
    return { binding: null, kind: 'fixture', reason: `invalid_authority:${validation.errors.join(',')}` };
  }

  const input: MissionCanvasOperationInput = {
    ...authority,
    workspace_profile_id: 'software',
    workspace_profile_revision: 2,
    activity_mode_id: 'overview',
    activity_mode_revision: 1,
    focused_work_surface_id: 'surface:pi',
    canonical_read_model_revision: 0,
    available_operations: [],
    capabilities: ['mission_canvas:read', 'mission_canvas:write', 'pi_attachment:attach', 'uiai:read'],
    permissions: ['mission_canvas:*'],
    viewport: {
      class: 'standard',
      css_height: 900,
      css_width: 1440,
      device_pixel_ratio: 2,
      zoom_percent: 100,
      high_contrast: false,
      reduced_motion: false,
      reduced_transparency: false,
      platform: 'macOS'
    },
    project_constraint_refs: [],
    user_preference_ref: null,
    resolver_rule_revision: 'adaptive-composition:v1',
    observed_at: new Date().toISOString(),
    idempotency_key: `preview-live-${crypto.randomUUID()}`,
    previous_projection_revision: 0
  };

  try {
    // A live projection may already exist for this exact scope (from a prior
    // resolve).  GET first; only resolve when none exists.  This keeps the
    // preview idempotent across reloads instead of conflicting on revision.
    try {
      const existing = await client.projectionGet({ ...authority });
      return { binding: { authority, client, projection: existing }, kind: 'live' };
    } catch (getError) {
      const isNotFound = getError instanceof MissionCanvasTransportError && getError.status === 404;
      if (!isNotFound) throw getError;
    }
    const projection = await client.projectionResolve(input);
    return { binding: { authority, client, projection }, kind: 'live' };
  } catch (error) {
    const reason = error instanceof MissionCanvasTransportError
      ? error.message
      : error instanceof Error
        ? error.message
        : 'live_binding_failed';
    // 404 = projection already resolved at a different revision (idempotent
    // replay is handled by Core); 409 revision conflict means a live
    // projection already exists for this exact scope.
    return { binding: null, kind: 'fixture', reason };
  }
}
