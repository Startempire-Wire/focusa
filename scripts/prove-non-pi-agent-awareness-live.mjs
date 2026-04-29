#!/usr/bin/env node
import http from 'node:http';

function get(path) {
  return new Promise((resolve, reject) => {
    const req = http.get({ host: '127.0.0.1', port: 8787, path, timeout: 5000 }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        if (res.statusCode < 200 || res.statusCode >= 300) return reject(new Error(`HTTP ${res.statusCode}: ${data}`));
        try { resolve(JSON.parse(data)); } catch (e) { reject(e); }
      });
    });
    req.on('timeout', () => req.destroy(new Error('timeout')));
    req.on('error', reject);
  });
}

const path = '/v1/awareness/card?adapter_id=openclaw&workspace_id=wirebot&agent_id=wirebot&operator_id=verious.smith&session_id=spec93-live&project_root=%2Fdata%2Fwirebot%2Fusers%2Fverious';
const body = await get(path);
const card = String(body.rendered_card || '');
const required = ['Focusa Utility Card', 'adapter=openclaw', 'workspace=wirebot', '/v1/doctor', 'checkpoint a scoped Workpoint', 'fetch Workpoint resume', 'capture or link evidence', 'record a prediction', 'cognition_degraded=true', 'Operator steering always wins'];
const missing = required.filter((needle) => !card.includes(needle));
if (body.status !== 'completed' || missing.length) {
  console.error('Spec93 live awareness proof: failed');
  console.error(JSON.stringify({ status: body.status, missing, body }, null, 2));
  process.exit(1);
}
console.log('Spec93 live awareness proof: passed');
console.log(`surface=${body.surface} adapter=${body.adapter_id} workspace=${body.workspace_id} workpoint_canonical=${body.workpoint_canonical}`);
