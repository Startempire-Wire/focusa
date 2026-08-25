const WIDGETS=[['focus','Today’s focus'],['workforce','Agents'],['controls','Quick controls'],['activity','Activity'],['brief','Workspace brief']];
const defaults=Object.fromEntries(WIDGETS.map(([id])=>[id,true]));
const storage=globalThis.chrome?.storage?.local;
const read=async()=>storage?.get('focusa_startpage_widgets')||{};
const write=(value)=>storage?.set({focusa_startpage_widgets:value});
const openPanel=()=>chrome.tabs.create({url:chrome.runtime.getURL('sidepanel.html')});
function renderWidgets(state){for(const [id] of WIDGETS){const node=document.querySelector(`[data-widget="${id}"]`);if(node)node.hidden=state[id]===false;}const toggles=document.querySelector('#widget-toggles');toggles.replaceChildren(...WIDGETS.map(([id,label])=>{const button=document.createElement('button');button.className=`toggle${state[id]?' on':''}`;button.type='button';button.setAttribute('aria-pressed',String(Boolean(state[id])));button.textContent=label;button.addEventListener('click',()=>{state[id]=!state[id];write(state);renderWidgets(state);});return button;}));}
function clock(){const node=document.querySelector('#clock');if(node)node.textContent=new Intl.DateTimeFormat(undefined,{weekday:'short',month:'short',day:'numeric',hour:'numeric',minute:'2-digit'}).format(new Date());}
function bind(){document.querySelector('#customize').addEventListener('click',()=>{const drawer=document.querySelector('#widget-drawer');drawer.hidden=!drawer.hidden;});for(const id of ['open-panel','manage-agents','orient-now','new-work','review-activity','pair-daemon'])document.querySelector(`#${id}`)?.addEventListener('click',openPanel);document.querySelector('#pause-work')?.addEventListener('click',()=>{document.querySelector('#runtime-label').textContent='Pause requested in command panel';openPanel();});}
const state={...defaults,...(await read()).focusa_startpage_widgets};renderWidgets(state);bind();clock();setInterval(clock,30000);
