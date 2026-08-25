const STORAGE_KEY='focusa.workforce.audit.v1';
const MAX_EVENTS=500;
const MAX_TEXT=220;
function area(chromeApi=globalThis.chrome){if(!chromeApi?.storage?.local)throw new Error('chrome.storage.local is unavailable');return chromeApi.storage.local;}
function bounded(value,fallback='unknown'){return typeof value==='string'&&value.trim()?value.trim().slice(0,MAX_TEXT):fallback;}
export function auditRecordFromEvent(event,source='Focusa daemon'){
  if(!event||event.schema!=='focusa.stream_event.v1')return null;
  return Object.freeze({event_id:bounded(event.event_id),cursor:bounded(event.cursor),timestamp:bounded(event.timestamp),event_type:bounded(event.event_type),schema_version:bounded(event.schema_version),correlation_id:bounded(event.correlation_id,''),invalidate:Array.isArray(event.invalidate)?event.invalidate.slice(0,20).map((item)=>bounded(String(item))):[],source:bounded(source),scope_keys:event.scope&&typeof event.scope==='object'?Object.keys(event.scope).slice(0,12):[]});
}
export async function listAuditRecords(chromeApi=globalThis.chrome){const raw=(await area(chromeApi).get(STORAGE_KEY))?.[STORAGE_KEY]??[];if(!Array.isArray(raw))throw new Error('stored audit collection is invalid');return raw.slice(0,MAX_EVENTS);}
export async function saveAuditRecord(record,chromeApi=globalThis.chrome){if(!record?.event_id)throw new TypeError('audit event id is required');const current=await listAuditRecords(chromeApi);const next=[record,...current.filter((item)=>item.event_id!==record.event_id)].slice(0,MAX_EVENTS);await area(chromeApi).set({[STORAGE_KEY]:next});return next;}
export async function clearAuditRecords(chromeApi=globalThis.chrome){await area(chromeApi).set({[STORAGE_KEY]:[]});return [];}
export const auditStorageKey=STORAGE_KEY;
