#!/usr/bin/env node
const endpoint = process.argv[2] || 'http://127.0.0.1:9334';
const timeoutMs = Number(process.env.FOCUSA_SMS_PROBE_TIMEOUT_MS || 5000);
const timer = setTimeout(() => {
  process.stderr.write('connector readiness timeout\n');
  process.exit(2);
}, timeoutMs);

async function cdp(wsUrl) {
  const ws = new WebSocket(wsUrl);
  await new Promise((resolve, reject) => {
    ws.onopen = resolve;
    ws.onerror = () => reject(new Error('cdp connection rejected'));
  });
  let nextId = 0;
  const pending = new Map();
  ws.onmessage = event => {
    const message = JSON.parse(event.data);
    const callback = pending.get(message.id);
    if (callback) {
      pending.delete(message.id);
      callback(message);
    }
  };
  const call = (method, params = {}, sessionId = undefined) => new Promise((resolve, reject) => {
    const id = ++nextId;
    const bounded = setTimeout(() => {
      pending.delete(id);
      reject(new Error('cdp command timeout'));
    }, timeoutMs);
    pending.set(id, message => {
      clearTimeout(bounded);
      if (message.error) reject(new Error('cdp command rejected'));
      else resolve(message.result || {});
    });
    const request = { id, method, params };
    if (sessionId) request.sessionId = sessionId;
    ws.send(JSON.stringify(request));
  });
  return { ws, call };
}

try {
  const versionResponse = await fetch(`${endpoint}/json/version`, { signal: AbortSignal.timeout(timeoutMs) });
  if (!versionResponse.ok) throw new Error('cdp unavailable');
  const version = await versionResponse.json();
  const { ws, call } = await cdp(version.webSocketDebuggerUrl);
  const targets = (await call('Target.getTargets')).targetInfos || [];
  const page = targets.find(target => target.type === 'page' && target.url.startsWith('https://messages.google.com/'));
  if (!page) throw new Error('messages page unavailable');
  const sessionId = (await call('Target.attachToTarget', { targetId: page.targetId, flatten: true })).sessionId;
  const expression = `(()=>{
    const list=document.querySelector('mws-conversations-list');
    const unable=!!document.querySelector('mw-unable-to-connect-container');
    const path=location.pathname;
    const rows=list?[...list.querySelectorAll('mws-conversation-list-item')]:[];
    return {
      origin_ok:location.origin==='https://messages.google.com',
      conversations:path.includes('/conversations'),
      unable,
      list_ready:!!list,
      list_probe_ok:!!list && Number.isInteger(rows.length)
    };
  })()`;
  const result = await call('Runtime.evaluate', { expression, returnByValue: true }, sessionId);
  const value = result.result?.value || {};
  const ready = value.origin_ok && value.conversations && !value.unable && value.list_ready && value.list_probe_ok;
  ws.close();
  clearTimeout(timer);
  process.stdout.write(JSON.stringify({ schema: 'focusa.sms_readiness.v1', ready, semantic: true }) + '\n');
  process.exit(ready ? 0 : 1);
} catch (error) {
  clearTimeout(timer);
  process.stderr.write(`connector readiness failed: ${error.constructor?.name || 'Error'}\n`);
  process.exit(1);
}
