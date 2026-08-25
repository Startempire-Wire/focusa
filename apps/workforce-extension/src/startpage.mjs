import { fetchWorkLoop, ProjectionRequestError } from './lib/api-client.mjs';
import { listConnections } from './lib/storage.mjs';

const WIDGETS=[['focus','Today’s focus'],['workforce','Agents'],['controls','Quick controls'],['activity','Activity'],['brief','Workspace brief']];
const defaults=Object.fromEntries(WIDGETS.map(([id])=>[id,true]));
const storage=globalThis.chrome?.storage?.local;
const read=async()=>storage?.get('focusa_startpage_widgets')||{};
const write=(value)=>storage?.set({focusa_startpage_widgets:value});
const openPanel=()=>chrome.tabs.create({url:chrome.runtime.getURL('sidepanel.html')});
function renderWidgets(state){for(const [id] of WIDGETS){const node=document.querySelector(`[data-widget="${id}"]`);if(node)node.hidden=state[id]===false;}const toggles=document.querySelector('#widget-toggles');toggles.replaceChildren(...WIDGETS.map(([id,label])=>{const button=document.createElement('button');button.className=`toggle${state[id]?' on':''}`;button.type='button';button.setAttribute('aria-pressed',String(Boolean(state[id])));button.textContent=label;button.addEventListener('click',()=>{state[id]=!state[id];write(state);renderWidgets(state);});return button;}));}
function clock(){const node=document.querySelector('#clock');if(node)node.textContent=new Intl.DateTimeFormat(undefined,{weekday:'short',month:'short',day:'numeric',hour:'numeric',minute:'2-digit'}).format(new Date());}
function bind(){document.querySelector('#customize').addEventListener('click',()=>{const drawer=document.querySelector('#widget-drawer');drawer.hidden=!drawer.hidden;});for(const id of ['open-panel','manage-agents','orient-now','new-work','review-activity','pair-daemon'])document.querySelector(`#${id}`)?.addEventListener('click',openPanel);document.querySelector('#pause-work')?.addEventListener('click',()=>{document.querySelector('#runtime-label').textContent='Pause requested in command panel';openPanel();});}
function setRuntimeState(label, tone='ready'){const node=document.querySelector('#runtime-label');if(node)node.textContent=label;const health=document.querySelector('.health');if(health)health.dataset.tone=tone;}
function setWidgetText(selector,text){const node=document.querySelector(selector);if(node)node.textContent=text;}
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
    const connections=await listConnections();
    const connection=connections[0];
    if(!connection){setRuntimeState('Connect a daemon','degraded');return;}
    const projection=await fetchWorkLoop({baseUrl:connection.base_url,token:connection.token});
    renderLiveWorkLoop(projection);
  }catch(error){
    const label=error instanceof ProjectionRequestError ? `Runtime ${error.kind}` : 'Runtime unavailable';
    setRuntimeState(label,'degraded');
  }
}
const state={...defaults,...(await read()).focusa_startpage_widgets};renderWidgets(state);bind();clock();setInterval(clock,30000);loadLiveWorkLoop();
