#!/usr/bin/env node
// Spec101 Bloatgaurd budget audit wrapper: delegates to the canonical static audit.
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const audit = path.join(root, 'tests', 'spec101_bloatgaurd_budgets_static_test.py');
const result = spawnSync('python3', [audit], { cwd: root, stdio: 'inherit' });
process.exit(result.status ?? 1);
