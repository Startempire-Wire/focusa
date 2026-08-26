import { fetchBrowserFleet } from './lib/api-client.mjs';
import { loadNotifPrefs, saveNotifPrefs } from './lib/notifications.mjs';
import { listNotifications, markNotificationsRead, notificationFromEvent, saveNotification, unreadNotificationCount } from './lib/notifications.mjs';
import { listConnections } from './lib/storage.mjs';
import { runReliableEventStream } from './lib/reconnect.mjs';

const WIDGETS=[['focus','Today’s focus'],['workforce','Agents'],['controls','Quick controls'],['activity','Activity'],['notifications','Notifications'],['fleet','Browser Fleet'],['brief','Workspace brief']];
const defaults=Object.fromEntries(WIDGETS.map(([id])=>[id,true]));
const storage=globalThis.chrome?.storage?.local;
const read=async()=>storage?.get('focusa_startpage_widgets')||{};
const write=(value)=>storage?.set({focusa_startpage_widgets:value});
const openPanel=()=>chrome.tabs.create({url:chrome.runtime.getURL('sidepanel.html')});
const openWall=()=>chrome.tabs.create({url:chrome.runtime.getURL('wall.html')});

function renderWidgets(state){for(const [id] of WIDGETS){const node=document.querySelector(`[data-widget="${id}"]`);if(node)node.hidden=state[id]===false;}const toggles=document.querySelector('#widget-toggles');toggles.replaceChildren(...WIDGETS.map(([id,label])=>{const button=document.createElement('button');button.className=`toggle${state[id]?' on':''}`;button.type='button';button.setAttribute('aria-pressed',String(Boolean(state[id])));button.textContent=label;button.addEventListener('click',()=>{state[id]=!state[id];write(state);renderWidgets(state);});return button;}));}
function clock(){const node=document.querySelector('#clock');if(node)node.textContent=new Intl.DateTimeFormat(undefined,{weekday:'short',month:'short',day:'numeric',hour:'numeric',minute:'2-digit'}).format(new Date());}
function bind(){document.querySelector('#customize').addEventListener('click',()=>{const drawer=document.querySelector('#widget-drawer');drawer.hidden=!drawer.hidden;});document.querySelector('#open-wall')?.addEventListener('click',openWall);for(const id of ['open-panel','manage-agents','orient-now','new-work','review-activity','pair-daemon'])document.querySelector(`#${id}`)?.addEventListener('click',openPanel);document.querySelector('#pause-work')?.addEventListener('click',()=>{document.querySelector('#runtime-label').textContent='Pause requested in command panel';openPanel();});}
function setWidgetText(selector,text){const node=document.querySelector(selector);if(node)node.textContent=text;}
let notifications=[];
function renderStartNotifications(){const node=document.querySelector('#start-notifications');if(!node)return;node.replaceChildren();if(!notifications.length){node.append(Object.assign(document.createElement('p'),{className:'muted',textContent:'No important signals yet.'}));}else for(const item of notifications.slice(0,5)){const row=document.createElement('div');row.className=`start-notification ${item.read?'':'unread'}`;const dot=document.createElement('span');dot.className=`notification-dot ${item.severity}`;const copy=document.createElement('div');copy.className='start-notification-copy';const title=document.createElement('strong');title.textContent=item.title;const body=document.createElement('small');body.textContent=`${item.body} · ${item.timestamp}`;copy.append(title,body);row.append(dot,copy);node.append(row);}setWidgetText('#start-notification-count',`${unreadNotificationCount(notifications)} unread`);}

let refreshPromise=null;
let streamAbort=null;
let streamCursor=null;
let selectedConnectionId=null;
let liveConnection=null;

async function loadSelectedConnection(){
  const connections=await listConnections();
  const select=document.querySelector('#daemon-select');
  renderOnboarding(connections.length);
  if(select){select.replaceChildren(...connections.map((item)=>new Option(item.label,item.connection_id)));if(!connections.length)select.append(new Option('No daemon connected',''));selectedConnectionId=selectedConnectionId&&connections.some((item)=>item.connection_id===selectedConnectionId)?selectedConnectionId:(connections[0]?.connection_id||'');select.value=selectedConnectionId;}
  return connections.find((item)=>item.connection_id===selectedConnectionId)||null;
}

function renderLiveWorkLoop(projection){
  const state=projection.status ?? 'unknown';
  const enabled=projection.enabled === true;
  const task=projection.current_task?.title || projection.current_task?.id || 'No current task';
  setRuntimeState(`Work Loop ${state}`, state === 'healthy' || state === 'active' ? 'ready' : 'degraded');
  setWidgetText('#focus-copy',enabled ? `Active task: ${task}. The daemon reports ${projection.status || 'an available'} Work Loop.` : 'Work Loop is idle. Open the command panel to choose the next governed action.');
  setWidgetText('#agent-count',enabled ? '1' : '0');
  setWidgetText('#agent-list',enabled ? `Work Loop · ${projection.status || 'active'}` : 'No active Work Loop.');
}

async function loadLiveWorkLoop(){
  try{
    const connection=await loadSelectedConnection();
    liveConnection=connection;
    if(!connection){setRuntimeState('Connect a daemon','degraded');return null;}
    const projection=await fetchWorkLoop({baseUrl:connection.base_url,token:connection.token});
    renderLiveWorkLoop(projection);
    setRuntimeState(`Work Loop live · ${connection.label}`,'ready');
    return connection;
  }catch(error){
    const label=error instanceof ProjectionRequestError ? `Runtime ${error.kind}` : 'Runtime unavailable';
    setRuntimeState(label,'degraded');
    return null;
  }
}

async function refreshLiveWorkLoop(){
  if(!refreshPromise)refreshPromise=loadLiveWorkLoop().finally(()=>{refreshPromise=null;});
  return refreshPromise;
}

async function startLiveUpdates(){
  const connection=await refreshLiveWorkLoop();
  if(!connection)return;
  streamAbort?.abort();
  streamAbort=new AbortController();
  try{
    await runReliableEventStream({
      baseUrl:connection.base_url,
      token:connection.token,
      initialCursor:streamCursor,
      signal:streamAbort.signal,
      onEvent:async(event)=>{await refreshLiveWorkLoop();const notification=notificationFromEvent(event);if(notification){notifications=await saveNotification(notification);renderStartNotifications();}},
      commitCursor:async(cursor)=>{streamCursor=cursor;},
      onState:(state)=>{
        const source=liveConnection?.label||'daemon';
        if(state.phase==='reconnecting')setRuntimeState(`Events reconnecting · ${source}`,'degraded');
        else if(state.phase==='unauthorized')setRuntimeState(`Events unauthorized · ${source}`,'degraded');
        else if(state.phase==='live')setRuntimeState(`Work Loop live · ${source}`,'ready');
      },
    });
  }catch(error){
    if(!streamAbort.signal.aborted)setRuntimeState(error?.name==='StreamAuthError'?'Events unauthorized':'Events unavailable','degraded');
  }
}

function setRuntimeState(label, tone='ready'){const node=document.querySelector('#runtime-label');if(node)node.textContent=label;const health=document.querySelector('.health');if(health)health.dataset.tone=tone;}

// ── Onboarding (F-speedrun) ──────────────────────────────────────────────
function renderOnboarding(connectionCount){
  const el=document.querySelector('#onboarding');
  if(!el)return;
  el.hidden=connectionCount!==0;
}

// ── Browser Fleet (F1 client) ────────────────────────────────────────────
function renderFleet(fleet){
  const body=document.querySelector('#fleet-body'); if(!body) return;
  const pools=Array.isArray(fleet?.pools)?fleet.pools:[];
  setWidgetText('#fleet-pools',String(pools.length));
  body.replaceChildren();
  if(!pools.length){body.append(Object.assign(document.createElement('p'),{className:'muted',textContent:'No pools reported.'}));return;}
  for(const p of pools){
    const row=document.createElement('div'); row.className='fleet-row';
    const label=document.createElement('strong');
    label.textContent=`pool ${p.max_pages ?? '?'}p · ${p.browser_state ?? '?'}`;
    const meta=document.createElement('small'); meta.className='muted';
    meta.textContent=`active ${p.active_pages ?? 0} · fails ${p.fail_count ?? 0}`;
    row.append(label,meta); body.append(row);
  }
}
async function loadFleet(){
  try{ const c=liveConnection||await loadSelectedConnection(); if(!c){setWidgetText('#fleet-body','Connect a daemon first.');return;}
    renderFleet(await fetchBrowserFleet({baseUrl:c.base_url,token:c.token}));
  }catch(e){ const b=document.querySelector('#fleet-body'); if(b) b.replaceChildren(Object.assign(document.createElement('p'),{className:'muted',textContent:`Fleet unavailable: ${e?.kind||'error'}`})); }
}

// ── Notification severity prefs (speedrun) ───────────────────────────────
async function renderNotifPrefToggles(){
  const drawer=document.querySelector('#widget-drawer');
  if(!drawer||drawer.dataset.prefs==='1')return;
  drawer.dataset.prefs='1';
  const prefs=await loadNotifPrefs();
  const wrap=document.createElement('div');
  wrap.className='notif-prefs';
  wrap.append(Object.assign(document.createElement('strong'),{textContent:'Notify severities'}));
  for(const sev of ['info','success','warning','danger']){
    const label=document.createElement('label');
    label.className='toggle';
    label.style.cursor='pointer';
    const box=document.createElement('input');
    box.type='checkbox'; box.checked=prefs[sev]!==false; box.style.marginRight='6px';
    box.addEventListener('change',async()=>{
      const next={...(await loadNotifPrefs())}; next[sev]=box.checked;
      await saveNotifPrefs(next);
    });
    label.append(box, document.createTextNode(sev));
    wrap.append(label);
  }
  drawer.append(wrap);
}

bind();
document.querySelector('#onboard-open-panel')?.addEventListener('click',openPanel);
document.querySelector('#mark-start-notifications-read')?.addEventListener('click',async()=>{notifications=await markNotificationsRead();renderStartNotifications();});
document.querySelector('#daemon-select')?.addEventListener('change',async(event)=>{selectedConnectionId=event.target.value;await storage?.set({'focusa_startpage_connection.v1':selectedConnectionId});streamAbort?.abort();await startLiveUpdates();});
document.querySelector('#fleet-refresh')?.addEventListener('click',loadFleet);

notifications=await listNotifications().catch(()=>[]);renderStartNotifications();
const savedSelection=await storage?.get('focusa_startpage_connection.v1');selectedConnectionId=savedSelection?.['focusa_startpage_connection.v1']||null;
const state={...defaults,...(await read()).focusa_startpage_widgets};renderWidgets(state);renderNotifPrefToggles();bind();clock();setInterval(clock,30000);startLiveUpdates();
window.addEventListener('pagehide',()=>streamAbort?.abort());
