import { captureActiveTab, createOrientationPacket, renderOrientationMission } from './lib/orientation.mjs';
import { startPairing, pollPairing } from './lib/pairing.mjs';
import { listConnections, saveConnection } from './lib/storage.mjs';
import { fetchHealth, fetchRoster, fetchWorkLoop } from './lib/api-client.mjs';
import { projectAudit, projectHealth, projectRoster, projectWorkLoop } from './lib/projections.mjs';
import { runReliableEventStream } from './lib/reconnect.mjs';
import { buildSafeSessionConfig, createPreflightedSession, preflightSafeSession } from './lib/session-create.mjs';
import { orchestrateAction } from './lib/orchestration.mjs';
import { renderAudit, renderRoster, setStatus } from './lib/views.mjs';
import { listNotifications, markNotificationsRead, notificationFromEvent, saveNotification, unreadNotificationCount } from './lib/notifications.mjs';

const $ = (selector) => { const node = document.querySelector(selector); if (!node) throw new Error(`required panel element missing: ${selector}`); return node; };
const elements = { connection: $('#connection-status'), select: $('#connection-select'), pairForm: $('#pair-form'), pairResult: $('#pair-result'), pairCheck: $('#pair-check'),
  capture: $('#capture-tab'), orientationForm: $('#orientation-form'), observation: $('#observation-summary'), mission: $('#mission-preview'), creationForm: $('#creation-form'),
  preflightResult: $('#preflight-result'), create: $('#create-draft'), start: $('#start-session'), refresh: $('#refresh-roster'), loop: $('#loop-summary'),
  roster: $('#roster'), stream: $('#stream-status'), audit: $('#audit'), notifications: $('#notifications'), notificationCount: $('#notification-count'), markNotificationsRead: $('#mark-notifications-read') };
let connection = null, pairing = null, observation = null, packet = null, preflight = null, draft = null, streamAbort = null;
const exactTargets = new Map(), auditEvents = [];
let notifications = [];
function renderNotifications(){ elements.notifications.replaceChildren(); if(!notifications.length){ elements.notifications.append(Object.assign(document.createElement('li'),{className:'empty',textContent:'No notifications yet.'})); } else for(const item of notifications.slice(0,25)){ const li=document.createElement('li'); li.className=item.read?'':'unread'; const dot=document.createElement('span'); dot.className=`notification-dot ${item.severity}`; const body=document.createElement('div'); body.className='notification-body'; const title=document.createElement('strong'); title.className='notification-title'; title.textContent=item.title; const message=document.createElement('small'); message.textContent=`${item.body} · ${item.source} · ${item.timestamp}`; body.append(title,message); li.append(dot,body); elements.notifications.append(li); } elements.notificationCount.textContent=`${unreadNotificationCount(notifications)} unread`; }
const INTENT_KEY = 'focusa.workforce.intents.v1';
const intentStore = { async load(key) { return (await chrome.storage.local.get(INTENT_KEY))[INTENT_KEY]?.[key] ?? null; },
  async persist(record) { const current = (await chrome.storage.local.get(INTENT_KEY))[INTENT_KEY] ?? {}; await chrome.storage.local.set({ [INTENT_KEY]: { ...current, [record.idempotency_key]: record } }); } };
function safeError(error) { return String(error?.kind || error?.failure_class || error?.message || 'unknown failure').slice(0, 180); }
function requestOptions() { if (!connection) throw new Error('paired connection required'); return { baseUrl: connection.base_url, token: connection.token }; }
function randomKey(prefix) { return `${prefix}:${crypto.randomUUID()}`; }

async function loadConnectionOptions(preferred = null) {
  const records = await listConnections(); elements.select.replaceChildren(new Option('Choose a paired daemon',''));
  for (const record of records) elements.select.append(new Option(record.label, record.connection_id));
  const selected = preferred ?? records[0]?.connection_id ?? ''; elements.select.value = selected;
  connection = records.find((item) => item.connection_id === selected) ?? null;
  setStatus(elements.connection, connection ? 'paired' : 'unconfigured');
  if (connection) await refreshObservation();
}
async function refreshObservation() {
  if (!connection) return;
  try {
    const [healthBody, loopBody, rosterBody] = await Promise.all([fetchHealth(requestOptions()), fetchWorkLoop(requestOptions()), fetchRoster(requestOptions())]);
    const health = projectHealth(healthBody), loop = projectWorkLoop(loopBody), roster = projectRoster(rosterBody);
    setStatus(elements.connection, health.status === 'healthy' ? 'paired' : 'degraded', connection.label);
    elements.loop.textContent = `${loop.state} · ${loop.status}${loop.current_task ? ` · ${loop.current_task.id}` : ''}`;
    renderRoster(elements.roster, roster, exactTargets, controlSession);
    startStream();
  } catch (error) { setStatus(elements.connection, error?.kind === 'unauthenticated' ? 'unauthorized' : error?.kind === 'forbidden' ? 'scope_denied' : 'degraded', safeError(error)); }
}
function startStream() {
  streamAbort?.abort(); streamAbort = new AbortController();
  runReliableEventStream({ ...requestOptions(), initialCursor: connection.last_cursor, signal: streamAbort.signal,
    onState: (state) => setStatus(elements.stream, state.phase, state.delay_ms ? `${state.delay_ms / 1000}s` : ''),
    onEvent: async (event) => { auditEvents.push(event); if (auditEvents.length > 200) auditEvents.shift(); renderAudit(elements.audit, projectAudit(auditEvents)); const notification=notificationFromEvent(event); if(notification){ notifications=await saveNotification(notification); renderNotifications(); } },
    commitCursor: async (cursor) => { connection = await saveConnection({ ...connection, last_cursor: cursor, last_connected_at: new Date().toISOString() }); },
  }).catch((error) => { if (error?.name !== 'AbortError') setStatus(elements.stream, error?.status === 401 ? 'unauthorized' : error?.status === 403 ? 'scope_denied' : 'degraded', safeError(error)); });
}

async function controlSession(action, row) {
  if (!row.exact_target || !connection) return;
  let payload = null;
  if (action === 'steer') { const instruction = window.prompt('Steering instruction'); if (!instruction) return; payload = { instruction }; }
  try { setStatus(elements.connection, 'replaying', `${action} pending canonical refresh`);
    await orchestrateAction({ action, target: row.exact_target, payload, idempotency_key: randomKey(action), idempotencyStore: intentStore, requestOptions: requestOptions() });
    await refreshObservation();
  } catch (error) { setStatus(elements.connection, error?.kind === 'stale_target' ? 'unknown' : 'degraded', safeError(error)); }
}

elements.select.addEventListener('change', async () => { streamAbort?.abort(); const records = await listConnections(); connection = records.find((item) => item.connection_id === elements.select.value) ?? null; setStatus(elements.connection, connection ? 'paired' : 'unconfigured'); if (connection) await refreshObservation(); });
elements.pairForm.addEventListener('submit', async (event) => { event.preventDefault(); const form = new FormData(elements.pairForm); try { pairing = await startPairing({ base_url: form.get('base_url'), label: form.get('label') });
  elements.pairResult.textContent = pairing.state === 'awaiting_approval' ? `Code ${pairing.code}. Approve it in Focusa, then check again.` : pairing.state; elements.pairCheck.hidden = pairing.state !== 'awaiting_approval'; setStatus(elements.connection, pairing.state);
} catch (error) { elements.pairResult.textContent = safeError(error); setStatus(elements.connection,'degraded'); } });
elements.pairCheck.addEventListener('click', async () => { if (!pairing) return; try { const result = await pollPairing(pairing); if (result.state === 'paired') { pairing = null; elements.pairCheck.hidden = true; await loadConnectionOptions(result.connection.connection_id); } else { pairing = result; elements.pairResult.textContent = result.state; setStatus(elements.connection,result.state); } } catch (error) { elements.pairResult.textContent = safeError(error); setStatus(elements.connection,'degraded'); } });
elements.capture.addEventListener('click', async () => { try { observation = await captureActiveTab(); elements.observation.textContent = `${observation.title} · ${observation.url}`; } catch (error) { elements.observation.textContent = safeError(error); } });
elements.orientationForm.addEventListener('submit', (event) => { event.preventDefault(); try { if (!observation) throw new Error('Capture a tab before review');
  const value = (id) => $(`#${id}`).value; packet = createOrientationPacket({ objective: value('objective'), exclusions: value('exclusions').split('\n').map((item) => item.trim()).filter(Boolean), observation,
    project_root: value('project-root'), continuity_id: value('continuity-id'), work_item_ref: value('work-item-ref') || null, role_profile_ref: value('role-profile-ref'), agent_identity_ref: 'agent:browser-created' });
  elements.mission.textContent = renderOrientationMission(packet); preflight = null; draft = null; elements.create.disabled = true; elements.start.disabled = true;
} catch (error) { elements.mission.textContent = safeError(error); } });
elements.creationForm.addEventListener('submit', async (event) => { event.preventDefault(); try { if (!packet) throw new Error('Review orientation first');
  const config = buildSafeSessionConfig({ packet, display_name: $('#agent-name').value, provider: $('#provider').value, model: $('#model').value, auth_profile_ref: $('#auth-profile').value });
  preflight = await preflightSafeSession(config, requestOptions()); elements.preflightResult.textContent = `Safe preflight approved. Redacted hash: ${preflight.redacted_config_hash}`; elements.create.disabled = false;
} catch (error) { preflight = null; elements.create.disabled = true; elements.preflightResult.textContent = safeError(error); } });
elements.create.addEventListener('click', async () => { try { draft = await createPreflightedSession({ preflight, idempotency_key: randomKey('create'), idempotencyStore: intentStore, requestOptions: requestOptions() });
  const target = { session_id: draft.session.id, run_id: draft.run.id, generation: draft.run.generation }; exactTargets.set(draft.session.id, target); elements.preflightResult.textContent = `Canonical draft ${draft.session.id}. Hash ${draft.redacted_config_hash}`; elements.start.disabled = false; await refreshObservation();
} catch (error) { elements.preflightResult.textContent = safeError(error); } });
elements.start.addEventListener('click', async () => { if (!draft) return; const target = exactTargets.get(draft.session.id); try { const result = await orchestrateAction({ action: 'start', target, idempotency_key: randomKey('start'), idempotencyStore: intentStore, requestOptions: requestOptions() });
  elements.preflightResult.textContent = `Canonical lifecycle: ${result.canonical.session.lifecycle}`; await refreshObservation();
} catch (error) { elements.preflightResult.textContent = safeError(error); } });
elements.refresh.addEventListener('click', refreshObservation);
elements.markNotificationsRead.addEventListener('click', async () => { notifications=await markNotificationsRead(); renderNotifications(); });
window.addEventListener('pagehide', () => streamAbort?.abort());
Promise.resolve(listNotifications()).then((items) => { notifications=items; renderNotifications(); }).catch(() => renderNotifications());
loadConnectionOptions().catch((error) => setStatus(elements.connection,'degraded',safeError(error)));
