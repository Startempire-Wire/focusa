import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const appRoot = resolve(here, '..');
const repoRoot = resolve(appRoot, '../..');
const config = JSON.parse(readFileSync(resolve(appRoot, 'src-tauri/tauri.conf.json'), 'utf8'));
const cargo = readFileSync(resolve(appRoot, 'src-tauri/Cargo.toml'), 'utf8');
const workflow = readFileSync(resolve(repoRoot, '.github/workflows/ci.yml'), 'utf8');

assert.deepEqual(
  config.plugins?.['deep-link']?.desktop?.schemes,
  ['focusa'],
  'Focusa Menubar must own only the focusa desktop URL scheme',
);
assert.equal(config.bundle?.active, true, 'scheme registration must be emitted into an app bundle');
assert.match(cargo, /^tauri-plugin-deep-link\s*=\s*"2"$/m, 'Rust deep-link plugin dependency missing');

assert.match(
  workflow,
  /codesign --verify --deep --strict/,
  'CI must verify the app bundle signature after Tauri emits the configured focusa scheme',
);

console.log('deep_link_registration_ok scheme=focusa bundle=active signing=ci-verified');
