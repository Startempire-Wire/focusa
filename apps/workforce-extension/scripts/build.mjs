import { cp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const dist = resolve(root, 'dist');
const manifest = JSON.parse(await readFile(resolve(root, 'manifest.json'), 'utf8'));

const exactPermissions = ['activeTab', 'sidePanel', 'storage'];
const actualPermissions = [...(manifest.permissions ?? [])].sort();
if (JSON.stringify(actualPermissions) !== JSON.stringify(exactPermissions)) {
  throw new Error(`manifest permissions must equal ${exactPermissions.join(', ')}`);
}
if ('content_scripts' in manifest) throw new Error('content scripts are forbidden in the MVP');
if ('host_permissions' in manifest) throw new Error('persistent host permissions are forbidden');
const optional = [...(manifest.optional_host_permissions ?? [])].sort();
if (JSON.stringify(optional) !== JSON.stringify(['http://*/*', 'https://*/*'])) {
  throw new Error('optional host permissions must be bounded to http(s) origins');
}
if (manifest.background?.service_worker !== 'background.mjs') {
  throw new Error('background service worker must be background.mjs');
}
if (manifest.side_panel?.default_path !== 'sidepanel.html') {
  throw new Error('side panel entry must be sidepanel.html');
}
if (manifest.chrome_url_overrides?.newtab !== 'startpage.html') {
  throw new Error('new tab entry must be startpage.html');
}

await rm(dist, { recursive: true, force: true });
await mkdir(dist, { recursive: true });
await writeFile(resolve(dist, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`);
await cp(resolve(root, 'src'), dist, { recursive: true });
console.log(`PASS: built Focusa Workforce MV3 unpacked extension at ${dist}`);
