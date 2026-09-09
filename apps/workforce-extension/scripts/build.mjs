import { cp, mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises';
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

// ── White-label: brand substitution from FOCUSA_BRAND (default Focusa Workforce). ──
const brand = process.env.FOCUSA_BRAND || 'Focusa Workforce';
const brandU = brand.toUpperCase();
const chip = brand.trim().charAt(0).toUpperCase();
const repls = [
  [/Focusa Workforce Chrome/g, `${brand} Chrome`],
  [/Focusa Workforce/g, brand],
  [/FOCUSA WORKFORCE/g, brandU],
  [/aria-hidden="true">F<\/div>/g, `aria-hidden="true">${chip}</div>`],
  [/<span class="mark">F<\/span>/g, `<span class="mark">${chip}</span>`],
];
let touched = 0;
for (const ent of await readdir(dist, { recursive: true })) {
  if (!ent.endsWith('.html') && !ent.endsWith('.mjs')) continue;
  const p = resolve(dist, ent);
  const txt = await readFile(p, 'utf8');
  let out = txt;
  for (const [re, to] of repls) out = out.replace(re, to);
  if (out !== txt) { await writeFile(p, out); touched++; }
}
console.log(`PASS: built ${brand} MV3 unpacked extension at ${dist} (${touched} files re-branded)`);

// ── Public demo: FOCUSA_PUBLIC_NEWTAB=1 makes the default new tab render the
// public Work view (dated, curated snapshot). Private builds are unchanged. ──
if (process.env.FOCUSA_PUBLIC_NEWTAB === '1') {
  await writeFile(resolve(dist, 'public-newtab.html'),
    '<!doctype html><meta http-equiv="refresh" content="0; url=startpage.html?public-work=1">\n');
  const distManifest = JSON.parse(await readFile(resolve(dist, 'manifest.json'), 'utf8'));
  distManifest.chrome_url_overrides = { newtab: 'public-newtab.html' };
  await writeFile(resolve(dist, 'manifest.json'), `${JSON.stringify(distManifest, null, 2)}\n`);
  console.log('PASS: default new tab points at the public Work view (FOCUSA_PUBLIC_NEWTAB=1)');
}
