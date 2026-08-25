const STORAGE_KEY='focusa.workforce.notifications.v1';
const MAX_NOTIFICATIONS=100;
const MAX_TEXT=180;
const IMPORTANT=/(approval|approved|blocked|failed|failure|degraded|completed|complete|attention|warning|revoked|expired|denied|error|stopped|paused)/i;
function area(chromeApi=globalThis.chrome){if(!chromeApi?.storage?.local)throw new Error('chrome.storage.local is unavailable');return chromeApi.storage.local;}
function text(value,fallback=''){return typeof value==='string'&&value.trim()?value.trim().slice(0,MAX_TEXT):fallback;}
function severity(event){const kind=String(event.event_type||'');if(/failed|failure|error|denied|revoked|expired|degraded/i.test(kind))return 'danger';if(/blocked|approval|attention|warning|paused|stopped/i.test(kind))return 'warning';if(/completed|complete|approved/i.test(kind))return 'success';return 'info';}
export function notificationFromEvent(event){
  if(!event||typeof event!=='object')return null;
  const kind=text(event.event_type,'Focusa event');
  const payload=event.payload&&typeof event.payload==='object'?event.payload:{};
  const important=payload.notification===true||IMPORTANT.test(kind);
  if(!important)return null;
  const summary=text(payload.summary||payload.message||payload.reason,`Focusa reported ${kind}.`);
  const scope=event.scope&&typeof event.scope==='object'?event.scope:{};
  return Object.freeze({id:text(event.event_id),cursor:text(event.cursor),timestamp:text(event.timestamp),event_type:kind,title:kind.replaceAll('_',' '),body:summary,severity:severity(event),source:text(scope.organization_id||scope.continuity_id,'Focusa daemon'),read:false});
}
export async function listNotifications(chromeApi=globalThis.chrome){const raw=(await area(chromeApi).get(STORAGE_KEY))?.[STORAGE_KEY]??[];if(!Array.isArray(raw))throw new Error('stored notification collection is invalid');return raw.slice(0,MAX_NOTIFICATIONS);}
export async function saveNotification(notification,chromeApi=globalThis.chrome){if(!notification?.id)throw new TypeError('notification id is required');const current=await listNotifications(chromeApi);const next=[notification,...current.filter(item=>item.id!==notification.id)].slice(0,MAX_NOTIFICATIONS);await area(chromeApi).set({[STORAGE_KEY]:next});return next;}
export async function markNotificationsRead(chromeApi=globalThis.chrome){const current=await listNotifications(chromeApi);const next=current.map(item=>({...item,read:true}));await area(chromeApi).set({[STORAGE_KEY]:next});return next;}
export function unreadNotificationCount(items){return (items??[]).filter(item=>item.read!==true).length;}
export const notificationStorageKey=STORAGE_KEY;
