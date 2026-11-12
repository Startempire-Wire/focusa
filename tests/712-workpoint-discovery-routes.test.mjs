import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const catalog = read('crates/focusa-api/src/routes/agent_capabilities.rs');
const router = read('crates/focusa-api/src/routes/workpoint.rs');

for (const [operation, handler] of [
  ['focusa.workpoint.resume', 'resume'],
  ['focusa.workpoint.link_evidence', 'link_evidence'],
]) {
  test(`${operation} advertises the method and path of its registered handler`, () => {
    const entry = catalog.match(new RegExp(`op\\(\\s*"${operation.replaceAll('.', '\\.')}"\\s*,\\s*"[^"]+"\\s*,\\s*"[^"]+"\\s*,\\s*"([A-Z]+)"\\s*,\\s*"([^"]+)"`));
    assert.ok(entry, `Missing operation ${operation}`);
    const registered = router.match(new RegExp(`\\.route\\(\\s*"([^"]+)"\\s*,\\s*(get|post)\\(${handler}\\)\\s*\\)`));
    assert.ok(registered, `Missing registered handler ${handler}`);
    assert.equal(entry[1], registered[2].toUpperCase());
    assert.equal(entry[2], registered[1]);
  });
}
