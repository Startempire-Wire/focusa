const ALLOWED_PROTOCOLS = new Set(['http:', 'https:']);
const MAX_URL_BYTES = 2048;
const MAX_TITLE_BYTES = 300;
const MAX_OBJECTIVE_BYTES = 4000;
const MAX_EXCLUSIONS = 10;
const MAX_EXCLUSION_BYTES = 300;
const MAX_MISSION_BYTES = 8192;
const encoder = new TextEncoder();

function byteLength(value) { return encoder.encode(value).byteLength; }
function boundedText(value, field, min, max) {
  if (typeof value !== 'string') throw new TypeError(`${field} must be a string`);
  const normalized = value.trim();
  const size = byteLength(normalized);
  if (size < min || size > max) throw new RangeError(`${field} must contain ${min}..${max} bytes`);
  return normalized;
}
function isoTimestamp(value, field) {
  const date = new Date(value);
  if (!value || Number.isNaN(date.valueOf())) throw new TypeError(`${field} must be an RFC3339 timestamp`);
  return date.toISOString();
}
function deepFreeze(value) {
  if (value && typeof value === 'object' && !Object.isFrozen(value)) {
    Object.freeze(value);
    for (const child of Object.values(value)) deepFreeze(child);
  }
  return value;
}

export function sanitizeBrowserObservation({ title, url, captured_at = new Date().toISOString() }) {
  const parsed = new URL(boundedText(url, 'url', 1, MAX_URL_BYTES));
  if (!ALLOWED_PROTOCOLS.has(parsed.protocol)) throw new TypeError('url must use http or https');
  if (parsed.username || parsed.password) {
    parsed.username = '';
    parsed.password = '';
  }
  parsed.hash = '';
  const sanitizedUrl = parsed.toString();
  if (byteLength(sanitizedUrl) > MAX_URL_BYTES) throw new RangeError(`url must contain at most ${MAX_URL_BYTES} bytes`);
  return deepFreeze({
    schema: 'focusa.browser_observation.v1',
    title: boundedText(title, 'title', 1, MAX_TITLE_BYTES),
    url: sanitizedUrl,
    origin: parsed.origin,
    captured_at: isoTimestamp(captured_at, 'captured_at'),
  });
}

export async function captureActiveTab(chromeApi = globalThis.chrome, now = () => new Date()) {
  if (!chromeApi?.tabs?.query) throw new Error('Chrome activeTab API is unavailable');
  const tabs = await chromeApi.tabs.query({ active: true, currentWindow: true });
  if (tabs.length !== 1 || !tabs[0]?.url || !tabs[0]?.title) throw new Error('One observable active tab is required');
  return sanitizeBrowserObservation({ title: tabs[0].title, url: tabs[0].url, captured_at: now().toISOString() });
}

export function createOrientationPacket(input, now = () => new Date()) {
  if (!input?.observation || input.observation.schema !== 'focusa.browser_observation.v1') {
    throw new TypeError('reviewed browser observation is required');
  }
  const exclusions = input.exclusions ?? [];
  if (!Array.isArray(exclusions) || exclusions.length > MAX_EXCLUSIONS) {
    throw new RangeError(`exclusions must contain at most ${MAX_EXCLUSIONS} entries`);
  }
  const packet = {
    schema: 'focusa.browser_orientation.v1',
    objective: boundedText(input.objective, 'objective', 1, MAX_OBJECTIVE_BYTES),
    exclusions: exclusions.map((value, index) => boundedText(value, `exclusions[${index}]`, 1, MAX_EXCLUSION_BYTES)),
    observation: sanitizeBrowserObservation(input.observation),
    project_root: boundedText(input.project_root, 'project_root', 1, 4096),
    continuity_id: boundedText(input.continuity_id, 'continuity_id', 1, 512),
    work_item_ref: input.work_item_ref == null ? null : boundedText(input.work_item_ref, 'work_item_ref', 1, 512),
    role_profile_ref: input.role_profile_ref == null ? null : boundedText(input.role_profile_ref, 'role_profile_ref', 1, 512),
    agent_identity_ref: boundedText(input.agent_identity_ref ?? 'agent:browser-created', 'agent_identity_ref', 1, 512),
    created_at: now().toISOString(),
  };
  renderOrientationMission(packet);
  return deepFreeze(packet);
}

export function renderOrientationMission(packet) {
  if (packet?.schema !== 'focusa.browser_orientation.v1') throw new TypeError('orientation packet schema is required');
  const exclusions = packet.exclusions.length ? packet.exclusions.map((item) => `- ${item}`).join('\n') : '- None declared';
  const mission = `OBJECTIVE\n${packet.objective}\n\nBROWSER ORIENTATION (OWNER-APPROVED)\nTitle: ${packet.observation.title}\nURL: ${packet.observation.url}\nCaptured: ${packet.observation.captured_at}\n\nEXCLUSIONS\n${exclusions}`;
  if (byteLength(mission) > MAX_MISSION_BYTES) throw new RangeError(`mission exceeds ${MAX_MISSION_BYTES} bytes; shorten it`);
  return mission;
}
