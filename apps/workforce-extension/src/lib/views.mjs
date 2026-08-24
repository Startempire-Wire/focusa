const STATUS = Object.freeze({
  unconfigured: ['Not connected','warning'], idle: ['Idle','warning'], permission_required: ['Permission required','warning'],
  permission_denied: ['Host permission denied','danger'], awaiting_approval: ['Awaiting pairing approval','warning'], expired: ['Pairing expired','danger'],
  revoked: ['Pairing revoked','danger'], token_consumed_repair_required: ['Pairing repair required','danger'], paired: ['Connected','success'],
  replaying: ['Replaying durable events','info'], live: ['Live','success'], reconnecting: ['Reconnecting','warning'],
  unauthorized: ['Connection revoked','danger'], scope_denied: ['Permission denied','danger'], degraded: ['Degraded','danger'], unknown: ['Unknown','warning'],
});
export function statusView(state, detail = '') {
  const [label, tone] = STATUS[state] ?? STATUS.unknown;
  return Object.freeze({ state: STATUS[state] ? state : 'unknown', label: detail ? `${label}: ${detail}` : label, tone });
}
export function rosterView(projection, exactTargets = new Map()) {
  const rows = projection?.rows ?? [];
  return Object.freeze(rows.map((row) => Object.freeze({ ...row, exact_target: exactTargets.get(row.id) ?? null,
    controls_enabled: exactTargets.has(row.id), status_text: `${row.lifecycle} · ${row.health} · ${row.semantic_activity}` })));
}
export function auditView(events) {
  return Object.freeze((events ?? []).map((event) => Object.freeze({ id: event.event_id, primary: event.event_type,
    secondary: `${event.timestamp ?? 'unknown time'} · cursor ${event.cursor}`, invalidates: event.invalidate.join(', ') || 'none' })));
}
export function setStatus(element, state, detail = '') {
  const view = statusView(state, detail); element.textContent = view.label; element.dataset.state = view.state; element.dataset.tone = view.tone; return view;
}
function text(tag, value, className) { const node = document.createElement(tag); node.textContent = value; if (className) node.className = className; return node; }
export function renderRoster(element, projection, exactTargets, onAction) {
  const rows = rosterView(projection, exactTargets); element.replaceChildren();
  if (!rows.length) { element.append(text('p','No daemon sessions are visible.','empty')); return rows; }
  for (const row of rows) {
    const item = document.createElement('li'); item.append(text('strong', row.display_name), text('span', row.status_text, 'meta'));
    const actions = document.createElement('div'); actions.className = 'actions';
    for (const action of ['pause','resume','steer','cancel']) { const button = text('button', action, 'secondary'); button.type = 'button'; button.disabled = !row.controls_enabled;
      button.title = row.controls_enabled ? `${action} ${row.display_name}` : 'Exact run target unavailable; refresh through a created session'; button.addEventListener('click', () => onAction(action,row)); actions.append(button); }
    item.append(actions); element.append(item);
  }
  return rows;
}
export function renderAudit(element, events) {
  const rows = auditView(events); element.replaceChildren();
  if (!rows.length) { element.append(text('p','No durable events rendered yet.','empty')); return rows; }
  for (const row of rows) { const item = document.createElement('li'); item.append(text('strong',row.primary),text('span',row.secondary,'meta'),text('span',`Invalidates: ${row.invalidates}`,'meta')); element.append(item); }
  return rows;
}
