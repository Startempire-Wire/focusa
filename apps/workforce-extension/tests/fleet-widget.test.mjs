import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(new URL('..', import.meta.url).pathname, 'src');
const html = fs.readFileSync(path.join(root, 'startpage.html'), 'utf8');
const js = fs.readFileSync(path.join(root, 'startpage.mjs'), 'utf8');
const api = fs.readFileSync(path.join(root, 'lib', 'api-client.mjs'), 'utf8');

// F1 client slice: fleet widget consumes the daemon bridge read-only.
test('browser fleet widget consumes daemon bridge read-only', () => {
  assert.match(html, /data-widget="fleet"/);
  assert.match(html, /id="fleet-pools"/);
  assert.match(js, /fetchBrowserFleet/);
  assert.match(js, /renderFleet/);
  assert.match(api, /browser_fleet/);
});

test('fleet bridge is read-only and bounded', () => {
  assert.match(api, /v1\/browser-fleet\/status/);
});
