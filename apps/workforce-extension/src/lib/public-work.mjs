import { MAX_PUBLIC_WORK_BYTES, validatePublicWorkSnapshot } from './contracts.mjs';

export async function loadPublicWorkSnapshot(url, fetchImpl = globalThis.fetch) {
  const response = await fetchImpl(url, {
    cache: 'no-store', credentials: 'omit', signal: AbortSignal.timeout(5000),
  });
  if (!response.ok || !response.body) throw new Error('public snapshot unavailable');
  const reader = response.body.getReader();
  const parts = []; let size = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.byteLength;
      if (size > MAX_PUBLIC_WORK_BYTES) throw new Error('public snapshot exceeds bound');
      parts.push(value);
    }
  } catch (error) {
    await reader.cancel();
    throw error;
  } finally { reader.releaseLock(); }
  const bytes = new Uint8Array(size); let offset = 0;
  for (const part of parts) { bytes.set(part, offset); offset += part.byteLength; }
  return validatePublicWorkSnapshot(JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes)));
}

export function renderPublicWork(document, host, snapshot, refresh) {
  const node = (tag, text, className) => {
    const element = document.createElement(tag);
    element.textContent = text;
    if (className) element.className = className;
    return element;
  };
  const button = node('button', 'Refresh snapshot', 'secondary');
  button.type = 'button'; button.addEventListener('click', refresh);
  if (!snapshot) {
    host.replaceChildren(node('p', 'PUBLIC WORK', 'eyebrow'),
      node('h2', 'Work information unavailable'),
      node('p', 'The public snapshot could not be verified. No private connection was attempted.'), button);
    return;
  }
  const work = validatePublicWorkSnapshot(snapshot);
  const head = node('div', '', 'widget-head');
  head.append(node('h2', work.project), node('span', work.stale ? 'Stale checkpoint' : 'Read-only checkpoint', 'badge'));
  const details = node('dl', '', 'public-work-details');
  for (const [label, value] of [['Workpoint state', work.state], ['Current stage', work.stage], ['Next action', work.next_action]]) {
    details.append(node('dt', label), node('dd', value));
  }
  host.replaceChildren(node('p', 'PUBLIC WORK', 'eyebrow'), head,
    node('p', work.mission, 'focus-copy'), details,
    node('p', `Checkpoint: ${work.checkpoint_at} · Published: ${work.published_at}`, 'muted'),
    node('p', 'This is a dated snapshot, not live agent telemetry. It grants no execution permission.', 'muted'), button);
}

export async function mountPublicWork(document, assetUrl, fetchImpl = globalThis.fetch) {
  const host = document.querySelector('[data-widget="focus"]');
  if (!host) throw new Error('Work host unavailable');
  host.classList.add('public-work');
  document.body.classList.add('public-work-mode');
  // This opt-in route does not read connections, notifications or layout preferences.
  for (const item of document.querySelectorAll('[data-widget]')) item.hidden = item !== host;
  for (const button of document.querySelectorAll('button')) {
    button.disabled = true; button.title = 'Unavailable in this public read-only view';
  }
  document.querySelector('#runtime-label').textContent = 'Public snapshot · read-only';
  document.querySelector('.welcome h1').textContent = 'Veragensia, in focus.';
  document.querySelector('.welcome .lead').textContent = 'Real checkpoint information from Focusa. Private connections and controls stay private.';
  const refresh = async () => {
    try { renderPublicWork(document, host, await loadPublicWorkSnapshot(assetUrl, fetchImpl), refresh); }
    catch (error) {
      console.warn('Public Work snapshot unavailable:', error?.name || 'Error');
      renderPublicWork(document, host, null, refresh);
    }
  };
  await refresh();
}
