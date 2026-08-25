import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root=path.resolve(new URL('..',import.meta.url).pathname,'src');
const html=fs.readFileSync(path.join(root,'startpage.html'),'utf8');
const js=fs.readFileSync(path.join(root,'startpage.mjs'),'utf8');
const css=fs.readFileSync(path.join(root,'startpage.css'),'utf8');

test('start page exposes high-level work view and widgetized controls',()=>{
  for(const id of ['dashboard','customize','widget-drawer','widget-toggles','open-panel','orient-now','new-work','pause-work','activity-list']) assert.match(html,new RegExp(`id="${id}"`),id);
  for(const widget of ['focus','workforce','controls','activity','notifications','brief']) assert.match(html,new RegExp(`data-widget="${widget}"`),widget);
  assert.match(html,/chrome_url_overrides|Start page/);
});

test('start page persists widget visibility locally and routes actions to the command panel',()=>{
  assert.match(js,/focusa_startpage_widgets/);
  assert.match(js,/state\[id\]=!state\[id\]/);
  assert.match(js,/chrome\.tabs\.create/);
});

test('start page consumes the canonical Work Loop projection and preserves degraded states',()=>{
  assert.match(js,/fetchWorkLoop/);
  assert.match(js,/listConnections/);
  assert.match(js,/ProjectionRequestError/);
  assert.match(js,/Runtime unavailable/);
  assert.match(js,/projection\.status/);
});

test('start page refreshes from governed SSE events and stops on page hide',()=>{
  assert.match(js,/runReliableEventStream/);
  assert.match(js,/initialCursor:streamCursor/);
  assert.match(js,/commitCursor:async\(cursor\)=>/);
  assert.match(js,/window\.addEventListener\('pagehide'/);
  assert.match(js,/streamAbort\?\.abort/);
});

test('start page and sidepanel share persisted notification projections',()=>{
  assert.match(js,/notificationFromEvent/);
  assert.match(js,/saveNotification/);
  assert.match(js,/markNotificationsRead/);
  assert.match(js,/start-notifications/);
});

test('start page has responsive widget grid and accessible motion/theme handling',()=>{
  assert.match(css,/grid-template-columns/);
  assert.match(css,/@media\(max-width:850px\)/);
  assert.match(css,/prefers-reduced-motion:reduce/);
  assert.match(css,/prefers-color-scheme:light/);
});
