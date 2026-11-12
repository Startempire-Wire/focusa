import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';

const source = await readFile(new URL('./index.ts', import.meta.url), 'utf8');
const output = stripTypeScriptTypes(source, { mode: 'strip' });
if (!process.argv.includes('--check')) {
  await mkdir(new URL('./dist/', import.meta.url), { recursive: true });
  await writeFile(new URL('./dist/index.js', import.meta.url), output);
}
