export type RichHostPlatform = "macOS" | "Windows" | "Linux";
export type InteractionMode = "canvas-guided" | "terminal-guided" | "headless";
export type RendererKind =
  | "focusa_pi_rich_window"
  | "mission_deck_web"
  | "pi_terminal_projection"
  | "native_tui"
  | "headless_none";
export type HostLifecycleStatus =
  | "absent"
  | "launching"
  | "visible"
  | "focused"
  | "hidden"
  | "closing"
  | "reconnecting"
  | "failed";

export interface MissionCanvasScope {
  project_root: string;
  continuity_id: string;
  instance_id?: string | null;
  session_id: string;
  attachment_id: string;
  working_subpath_id?: string | null;
}

export interface HostCapabilityProbe {
  platform: RichHostPlatform;
  architecture: string;
  native_binary_path?: string;
  native_binary_available: boolean;
  system_browser_available: boolean;
  tui_available: boolean;
  headless: boolean;
  reason: string;
}

export interface HostRendererResolution {
  interaction_mode: InteractionMode;
  selected_renderer: RendererKind;
  platform: RichHostPlatform;
  availability: "available" | "fallback" | "unavailable" | "headless";
  resolution_reason: string;
  asset_version: string | null;
  asset_digest: string | null;
  resolver_revision: "host-resolver:v1";
  diagnostic_ref: string | null;
}

export interface RichHostLaunchRequest {
  scope: MissionCanvasScope;
  daemon_base_url: string;
  token?: string;
  interaction_mode: InteractionMode;
  package_root: string;
  asset_version: string;
}

export interface RichHostProcessHandle {
  process_id?: number;
  window_id: string;
  renderer: RendererKind;
}

export interface RichHostLifecycleState {
  host_instance_id: string;
  scope: MissionCanvasScope;
  renderer_resolution: HostRendererResolution;
  state: HostLifecycleStatus;
  process_id: number | null;
  window_id: string | null;
  focused: boolean;
  durable_event_cursor: string;
  pi_draft_ref: string | null;
  canvas_draft_ref: string | null;
  last_error_ref: string | null;
  lifecycle_revision: number;
  updated_at: string;
}

export interface RichHostProcessAdapter {
  launch(request: RichHostLaunchRequest, resolution: HostRendererResolution, handshakePath: string): Promise<RichHostProcessHandle>;
  focus(handle: RichHostProcessHandle): Promise<void>;
  hide(handle: RichHostProcessHandle): Promise<void>;
  close(handle: RichHostProcessHandle): Promise<void>;
  isAlive(handle: RichHostProcessHandle): Promise<boolean>;
}
