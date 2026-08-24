import { validateConnectionRecord } from './contracts.mjs';

const STORAGE_KEY = 'focusa.workforce.connections.v1';
function localArea(chromeApi) {
  if (!chromeApi?.storage?.local) throw new Error('chrome.storage.local is unavailable');
  return chromeApi.storage.local;
}

export async function listConnections(chromeApi = globalThis.chrome) {
  const result = await localArea(chromeApi).get(STORAGE_KEY);
  const raw = result?.[STORAGE_KEY] ?? [];
  if (!Array.isArray(raw)) throw new Error('stored connection collection is invalid');
  return raw.map(validateConnectionRecord);
}

export async function saveConnection(record, chromeApi = globalThis.chrome) {
  const valid = validateConnectionRecord(record);
  const current = await listConnections(chromeApi);
  const next = current.filter((item) => item.connection_id !== valid.connection_id);
  next.push(valid);
  next.sort((a, b) => a.connection_id.localeCompare(b.connection_id));
  await localArea(chromeApi).set({ [STORAGE_KEY]: next });
  const committed = (await listConnections(chromeApi)).find((item) => item.connection_id === valid.connection_id);
  if (!committed || committed.token !== valid.token) throw new Error('connection storage commit could not be verified');
  return committed;
}

export async function forgetConnection(connectionId, chromeApi = globalThis.chrome) {
  const current = await listConnections(chromeApi);
  const next = current.filter((item) => item.connection_id !== connectionId);
  await localArea(chromeApi).set({ [STORAGE_KEY]: next });
  return current.length !== next.length;
}

export const connectionStorageKey = STORAGE_KEY;
