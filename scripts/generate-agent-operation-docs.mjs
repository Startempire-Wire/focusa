#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const root = process.cwd();
const registryPath = path.join(root, 'docs/contracts/spec135/generated-contract-v1/operation-registry.json');
const registry = JSON.parse(fs.readFileSync(registryPath, 'utf8'));
const check = process.argv.includes('--check');
const groups = new Map();
for (const operation of registry.operations) {
  if (!operation.docs_ref.startsWith('docs/focusa-api/routes/')) continue;
  const list = groups.get(operation.docs_ref) || [];
  list.push(operation);
  groups.set(operation.docs_ref, list);
}

let drift = 0;
for (const [docRef, operations] of [...groups.entries()].sort()) {
  operations.sort((a, b) => a.operation_id.localeCompare(b.operation_id));
  const family = path.basename(docRef, '.md');
  const lines = [
    `# Focusa ${family.replaceAll('_', ' ')} Agent Operations`,
    '',
    'Generated from `docs/contracts/spec135/generated-contract-v1/operation-registry.json`.',
    '',
    'These operations preserve daemon scope, permission, confirmation, idempotency, receipt, and recovery authority across REST, MCP, OpenAI-compatible tools, and generated UI.',
    '',
  ];
  for (const operation of operations) {
    lines.push(
      `## \`${operation.operation_id}\``,
      '',
      operation.description,
      '',
      `- Method/path: \`${operation.method} ${operation.path}\``,
      `- Family: \`${operation.family}\``,
      `- Input schema: \`${operation.contracts.input_schema_ref}\``,
      `- Output schema: \`${operation.contracts.output_schema_ref}\``,
      `- Error schema: \`${operation.contracts.error_schema_ref}\``,
      `- Permission scopes: ${(operation.control.permission_scopes || []).map((value) => `\`${value}\``).join(', ') || 'none'}`,
      `- Confirmation: \`${operation.control.confirmation}\``,
      `- Idempotency key required: \`${operation.control.idempotency_required}\``,
      `- Receipt required: \`${operation.control.receipt_required}\``,
      `- Reversible: \`${operation.control.reversible}\``,
      `- Required scope keys: ${(operation.scope.required_keys || []).map((value) => `\`${value}\``).join(', ') || 'none'}`,
      '',
      '### Example request',
      '',
      '```json',
      JSON.stringify(operation.examples?.request || {}, null, 2),
      '```',
      '',
      '### Example result',
      '',
      '```json',
      JSON.stringify(operation.examples?.response || {}, null, 2),
      '```',
      '',
      '### Failure and recovery',
      '',
      `Failure classes: ${(operation.errors || []).map((error) => `\`${error.failure_class || error}\``).join(', ') || '`focusa.operation_error.v1`'}.`,
      '',
      'Use the structured error recovery field, preserve the original scope and idempotency key when retry-safe, and run the indicated doctor/verify capability before any authority-sensitive retry.',
      '',
    );
  }
  const body = `${lines.join('\n').trimEnd()}\n`;
  const outputPath = path.join(root, docRef);
  const current = fs.existsSync(outputPath) ? fs.readFileSync(outputPath, 'utf8') : null;
  if (current !== body) {
    drift += 1;
    if (!check) {
      fs.mkdirSync(path.dirname(outputPath), { recursive: true });
      fs.writeFileSync(outputPath, body);
    }
  }
}
if (check && drift) {
  console.error(`Spec141 operation docs drift: ${drift} file(s)`);
  process.exit(1);
}
console.log(JSON.stringify({ status: 'passed', mode: check ? 'check' : 'write', documents: groups.size, operations: [...groups.values()].reduce((sum, items) => sum + items.length, 0) }));
