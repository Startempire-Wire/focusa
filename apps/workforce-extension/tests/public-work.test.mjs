import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';
import { MAX_PUBLIC_WORK_BYTES, validatePublicWorkSnapshot } from '../src/lib/contracts.mjs';
import { loadPublicWorkSnapshot, renderPublicWork, mountPublicWork } from '../src/lib/public-work.mjs';

const sample = () => ({ schema:'focusa.public_work_snapshot.v1', visibility:'public', project:'Example project',
  mission:'Deliver the Work view', state:'active', stage:'Verification', next_action:'Verify the visible view',
  checkpoint_at:'2026-09-08T20:00:00Z', published_at:'2026-09-08T20:01:00Z', stale:false });
class Element {
  constructor(tag='div') { this.tagName=tag; this.children=[]; this.value=''; this.listeners={}; this.classList={add(){}}; }
  set textContent(value) { this.value=String(value); this.children=[]; }
  get textContent() { return this.value + this.children.map(child=>child.textContent).join(' '); }
  set innerHTML(_) { throw new Error('HTML injection forbidden'); }
  append(...children) { this.children.push(...children); }
  replaceChildren(...children) { this.value=''; this.children=children; }
  addEventListener(type, callback) { this.listeners[type]=callback; }
}
function dom() {
  const host=new Element(), other=new Element(), button=new Element('button'), nodes=new Map();
  nodes.set('[data-widget="focus"]',host);
  return {host,other,button, body:new Element('body'),createElement:tag=>new Element(tag),
    querySelector(selector){if(!nodes.has(selector))nodes.set(selector,new Element());return nodes.get(selector);},
    querySelectorAll(selector){return selector==='[data-widget]'?[host,other]:selector==='button'?[button]:[];}};
}

test('public contract rejects extra private fields, versions, states and malformed timestamps',()=>{
  assert.equal(validatePublicWorkSnapshot(sample()).state,'active');
  for(const change of [{token:'PRIVATE'},{schema:'focusa.public_work_snapshot.v2'}, {visibility:'private'},
    {state:'running-workers'},{stale:'false'},{published_at:'invalid'},{mission:'x'.repeat(481)}]) {
    assert.throws(()=>validatePublicWorkSnapshot({...sample(),...change}));
  }
});
test('loader omits credentials and authorization, reads bounded JSON',async()=>{
  const result=await loadPublicWorkSnapshot('chrome-extension://test/public-work.json',async(url,options)=>{
    assert.equal(options.credentials,'omit'); assert.equal(options.headers,undefined);
    assert.equal(options.cache,'no-store'); assert.ok(options.signal instanceof AbortSignal);
    return new Response(JSON.stringify(sample()));
  });
  assert.equal(result.project,sample().project);
});
test('loader rejects errors, invalid JSON and excessive output',async()=>{
  for(const response of [new Response('',{status:404}),new Response('not JSON'),new Response('x'.repeat(MAX_PUBLIC_WORK_BYTES+1))]) {
    await assert.rejects(loadPublicWorkSnapshot('chrome-extension://test/public-work.json',async()=>response));
  }
});
test('renderer uses text nodes and never represents snapshot as live telemetry',()=>{
  const document=dom(), value={...sample(),mission:'<img src=x onerror=alert(1)>'};
  renderPublicWork(document,document.host,value,()=>{});
  assert.ok(document.host.textContent.includes(value.mission));
  assert.ok(document.host.textContent.includes('not live agent telemetry'));
  assert.ok(document.host.textContent.includes('Next action'));
  assert.ok(document.host.textContent.includes('2026-09-08'));
  renderPublicWork(document,document.host,null,()=>{});
  assert.ok(document.host.textContent.includes('unavailable'));
  assert.ok(!document.host.textContent.includes(value.mission));
});
test('public mount hides private areas and exposes only read-only refresh',async()=>{
  const document=dom();
  await mountPublicWork(document,'chrome-extension://test/public-work.json',async()=>new Response(JSON.stringify(sample())));
  assert.equal(document.other.hidden,true);assert.equal(document.button.disabled,true);
  assert.equal(document.host.children.at(-1).textContent,'Refresh snapshot');
  assert.ok(document.host.textContent.includes(sample().mission));
});
test('explicit public bootstrap never reads private storage; normal route is retained',async()=>{
  const original=await readFile(new URL('../src/startpage.mjs',import.meta.url),'utf8');
  const anchor="if (new URL(window.location.href).searchParams.get('public-work') === '1') {";
  assert.ok(original.includes(anchor));
  const source=original.replace(/^import .*;\n/gm,'').replace(anchor,
    "renderStartNotifications=()=>{};renderWidgets=()=>{};renderNotifPrefToggles=()=>{};bind=()=>{};clock=()=>{};startLiveUpdates=()=>events.push('private-runtime');startFleetEventStream=()=>{};\n"+anchor);
  for(const publicMode of [true,false]) {
    const events=[], document=dom();
    const context={events,document,URL,console,Intl,setInterval(){},
      window:{location:{href:'chrome-extension://test/startpage.html'+(publicMode?'?public-work=1':'')},addEventListener(){}},
      chrome:{runtime:{getURL:path=>'chrome-extension://test/'+path},storage:{local:{get:async()=>{events.push('storage');return {};}}}},
      mountPublicWork:async()=>events.push('public'),listNotifications:async()=>{events.push('notifications');return [];}};
    await vm.runInNewContext('(async()=>{'+source+'})()',context);
    if(publicMode)assert.deepEqual(events,['public']);
    else {assert.ok(events.includes('private-runtime'));assert.ok(events.includes('storage'));assert.ok(!events.includes('public'));}
  }
});
