import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readdir, readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

async function digestTree(directory) {
  const files = (await readdir(directory, { recursive: true })).sort();
  const hash = createHash('sha256');
  for (const relative of files) {
    const path = resolve(directory, relative);
    try {
      const bytes = await readFile(path);
      hash.update(relative); hash.update(bytes);
    } catch (error) {
      if (error?.code !== 'EISDIR') throw error;
    }
  }
  return hash.digest('hex');
}

function build() {
  const result = spawnSync(process.execPath, ['scripts/build.mjs'], { cwd: root, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stdout, /PASS: built Focusa Workforce MV3 unpacked extension/);
}

test('manifest is least-privilege MV3 with no content script', async () => {
  const manifest = JSON.parse(await readFile(resolve(root, 'manifest.json'), 'utf8'));
  assert.equal(manifest.manifest_version, 3);
  assert.deepEqual([...manifest.permissions].sort(), ['activeTab', 'sidePanel', 'storage']);
  assert.deepEqual([...manifest.optional_host_permissions].sort(), ['http://*/*', 'https://*/*']);
  assert.equal(manifest.content_scripts, undefined);
  assert.equal(manifest.host_permissions, undefined);
});

test('unpacked build is deterministic and complete', async () => {
  build(); const first = await digestTree(resolve(root, 'dist'));
  build(); const second = await digestTree(resolve(root, 'dist'));
  assert.equal(second, first);
  for (const file of ['manifest.json', 'background.mjs', 'sidepanel.html', 'sidepanel.mjs', 'styles.css']) {
    await readFile(resolve(root, 'dist', file));
  }
});
