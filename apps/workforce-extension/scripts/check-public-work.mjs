import { lstat, readFile } from 'node:fs/promises';
import { MAX_PUBLIC_WORK_BYTES, validatePublicWorkSnapshot } from '../src/lib/contracts.mjs';

try {
  const path = process.argv[2];
  const stat = await lstat(path);
  if (!stat.isFile() || stat.isSymbolicLink() || stat.size > MAX_PUBLIC_WORK_BYTES) throw new Error();
  validatePublicWorkSnapshot(JSON.parse(await readFile(path, 'utf8')));
  console.log('Public Work snapshot contract verified.');
} catch {
  console.error('Public Work snapshot rejected; no content was published.');
  process.exitCode = 2;
}
