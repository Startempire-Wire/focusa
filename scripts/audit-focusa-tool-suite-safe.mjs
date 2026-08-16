#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { spawnSync } from 'node:child_process';

const root = process.cwd();
const baseUrl = process.env.FOCUSA_API_BASE_URL || 'http://127.0.0.1:8787';
const timeoutMs = Number(process.env.FOCUSA_AUDIT_TIMEOUT_MS || 5000);
const failures = [];
const warnings = [];
const probes = [];
const processChecks = [];
const skippedColdGetRoutes = [];
const latencyGuardrails = [];
const hotRouteWarnMs = Number(process.env.FOCUSA_AUDIT_HOT_ROUTE_WARN_MS || 500);
const hotRouteFailMs = Number(process.env.FOCUSA_AUDIT_HOT_ROUTE_FAIL_MS || 2000);
const coldRouteTimeoutMs = Number(process.env.FOCUSA_AUDIT_COLD_ROUTE_TIMEOUT_MS || timeoutMs);
const hotRouteLatencySamples = Math.max(1, Number(process.env.FOCUSA_AUDIT_HOT_ROUTE_LATENCY_SAMPLES || 2));

function read(file) { return fs.readFileSync(path.join(root, file), 'utf8'); }
function readJson(file) { return JSON.parse(read(file)); }
function pushFailure(failure_class, surface, message, recovery, details = {}) {
  failures.push({ failure_class, surface, message, recovery, details });
}
function pushWarning(failure_class, surface, message, recovery, details = {}) {
  warnings.push({ failure_class, surface, message, recovery, details });
}
function routeTier(endpoint) {
  const route = String(endpoint || '').toLowerCase();
  if (route.includes('/deep') || route.includes('/replay/') || route.includes('/state/dump') || route.includes('closure-bundle') || route.includes('closure-evidence') || route.includes('include_full_payload=true') || route.includes('mode=full') || /[?&]deep=true/.test(route)) return 'cold';
  return 'hot';
}
function timeoutFailureClass(endpoint) {
  return routeTier(endpoint) === 'cold' ? 'cold_path_timeout' : 'hot_path_timeout';
}
function timeoutForEndpoint(endpoint) {
  return routeTier(endpoint) === 'cold' ? coldRouteTimeoutMs : timeoutMs;
}
function recordLatencyGuardrail(endpoint, elapsed_ms, ok) {
  const tier = routeTier(endpoint);
  const record = { endpoint, route_tier: tier, elapsed_ms, ok, warn_ms: tier === 'hot' ? hotRouteWarnMs : coldRouteTimeoutMs, fail_ms: tier === 'hot' ? hotRouteFailMs : coldRouteTimeoutMs };
  latencyGuardrails.push(record);
  if (tier !== 'hot' || !ok) return;
  if (elapsed_ms > hotRouteFailMs) pushFailure('hot_path_timeout', endpoint, `hot route exceeded fail guardrail ${elapsed_ms}ms > ${hotRouteFailMs}ms`, 'Move cold work off hot route or lower payload/lock work.', record);
  else if (elapsed_ms > hotRouteWarnMs) pushWarning('hot_path_timeout', endpoint, `hot route exceeded warning guardrail ${elapsed_ms}ms > ${hotRouteWarnMs}ms`, 'Monitor and reduce hot path work before it causes tool timeouts.', record);
}
async function getJson(endpoint, options = {}) {
  const url = `${baseUrl}${endpoint}`;
  const started = Date.now();
  const res = await fetch(url, { signal: AbortSignal.timeout(timeoutForEndpoint(endpoint)) });
  const elapsed_ms = Date.now() - started;
  const text = await res.text();
  let body = null;
  try { body = text ? JSON.parse(text) : null; } catch { body = text; }
  probes.push({ endpoint, status: res.status, ok: res.ok, elapsed_ms, route_tier: routeTier(endpoint) });
  if (options.recordLatency !== false) recordLatencyGuardrail(endpoint, elapsed_ms, res.ok);
  if (!res.ok) {
    const err = new Error(`${endpoint} HTTP ${res.status}`);
    err.status = res.status;
    err.body = await res.text().catch(() => "");
    throw err;
  }
  return { body, elapsed_ms };
}

async function getJsonWithRetry(endpoint, attempts = 2) {
  let lastErr = null;
  let lastResult = null;
  for (let attempt = 1; attempt <= attempts; attempt++) {
    try {
      lastResult = await getJson(endpoint, { recordLatency: false });
      let bestElapsedMs = lastResult.elapsed_ms;
      const samples = routeTier(endpoint) === 'hot' ? hotRouteLatencySamples : 1;
      for (let sample = 1; sample < samples; sample++) {
        try {
          const sampled = await getJson(endpoint, { recordLatency: false });
          bestElapsedMs = Math.min(bestElapsedMs, sampled.elapsed_ms);
        } catch (sampleErr) {
          pushWarning(sampleErr.name === 'TimeoutError' ? timeoutFailureClass(endpoint) : 'daemon_unavailable', endpoint, `latency sample ${sample + 1} failed: ${sampleErr.message}`, 'Best-of latency sampling keeps the successful probe but flags repeated instability.');
        }
      }
      recordLatencyGuardrail(endpoint, bestElapsedMs, true);
      return lastResult.body;
    } catch (err) {
      lastErr = err;
      if (attempt < attempts) {
        const isValidation = typeof err.message === 'string' && /\bHTTP 4\d\d\b/.test(err.message);
        if (err.name === 'TimeoutError') {
          pushWarning(timeoutFailureClass(endpoint), endpoint, `attempt ${attempt} failed: ${err.message}`, 'Retry succeeded/remaining attempts will determine failure; inspect daemon resource pressure if repeated.');
        } else if (isValidation) {
          // Don't warn on retryable 4xx; the outer catch will classify the
          // final error as `probe_validation_expected`.
          await new Promise((resolve) => setTimeout(resolve, 150));
        } else {
          pushWarning('daemon_unavailable', endpoint, `attempt ${attempt} failed: ${err.message}`, 'Retry succeeded/remaining attempts will determine failure; inspect daemon resource pressure if repeated.');
          await new Promise((resolve) => setTimeout(resolve, 150));
        }
      }
    }
  }
  throw lastErr;
}
function routePath(route) { return String(route).replace(/^(GET|POST|PATCH|PUT|DELETE)\s+/, '').split('?')[0]; }
function routeMethod(route) { return String(route).match(/^(GET|POST|PATCH|PUT|DELETE)\s+/)?.[1] || 'GET'; }

function parseStatusKb(statusText, key) {
  const line = statusText.split('\n').find((entry) => entry.startsWith(`${key}:`));
  return line ? Number(line.trim().split(/\s+/)[1]) : null;
}
function checkDaemonProcessMemory() {
  const pidResult = spawnSync('pidof', ['focusa-daemon'], { encoding: 'utf8' });
  const pid = String(pidResult.stdout || '').trim().split(/\s+/).filter(Boolean)[0];
  if (!pid) {
    pushWarning('daemon_unavailable', 'process_memory', 'focusa-daemon process not found by pidof', 'Check service state.');
    return;
  }
  try {
    const statusText = fs.readFileSync(`/proc/${pid}/status`, 'utf8');
    const rss_kb = parseStatusKb(statusText, 'VmRSS');
    const peak_rss_kb = parseStatusKb(statusText, 'VmHWM');
    const check = { pid, rss_kb, peak_rss_kb };
    processChecks.push(check);
    const warnKb = Number(process.env.FOCUSA_AUDIT_RSS_WARN_KB || 700000);
    const criticalKb = Number(process.env.FOCUSA_AUDIT_RSS_CRITICAL_KB || 1000000);
    if (rss_kb && rss_kb >= criticalKb) pushWarning('resource_exhausted', 'process_memory', `daemon RSS ${rss_kb}KB exceeds critical audit threshold`, 'Enable low-memory mode: use bounded/cached hot paths, avoid cold routes, inspect store caps.', check);
    else if (rss_kb && rss_kb >= warnKb) pushWarning('resource_exhausted', 'process_memory', `daemon RSS ${rss_kb}KB exceeds warning audit threshold`, 'Prefer bounded hot paths and monitor store growth.', check);
  } catch (err) {
    pushWarning('unknown_ambiguous_completion', 'process_memory', `could not read daemon process memory: ${err.message}`, 'Use system ps/proc manually.');
  }
}
checkDaemonProcessMemory();

const staticValidation = spawnSync(process.execPath, ['scripts/validate-focusa-tool-contracts.mjs', '--json'], { cwd: root, encoding: 'utf8' });
if (staticValidation.status !== 0) pushFailure('validation_rejected', 'static_contracts', 'Spec90 static validation failed', 'Fix contract/tool/doc drift before live testing.', { stdout: staticValidation.stdout, stderr: staticValidation.stderr });
const staticResult = staticValidation.stdout ? JSON.parse(staticValidation.stdout) : null;
const registry = readJson('docs/current/focusa-tool-contracts.json');
const choreography = readJson('docs/current/focusa-tool-choreography.json');
const contracts = registry.contracts || [];

// Doc/affordance checks.
for (const c of contracts) {
  const docAbs = path.join(root, c.doc_path || '');
  if (!c.doc_path || !fs.existsSync(docAbs)) pushFailure('validation_rejected', c.name, 'Missing tool doc page', 'Create one doc page per official tool.', { doc_path: c.doc_path });
  else {
    const doc = fs.readFileSync(docAbs, 'utf8');
    const hasPurpose =
      doc.includes('## Purpose') ||
      /^Use (?:before|when|to|for)\b/m.test(doc) ||
      /^# `?[^\n`]+`?\n\n(?![-#|])[A-Z][^\n]+/m.test(doc);
    const hasExpectedResult =
      doc.includes('## Expected result') ||
      doc.includes('## Output') ||
      /Result envelope:/i.test(doc);
    if (!hasPurpose)
      pushWarning(
        'unknown_ambiguous_completion',
        c.name,
        'Doc missing Purpose section',
        'Add model-facing purpose guidance.',
        { doc_path: c.doc_path },
      );
    if (!hasExpectedResult)
      pushWarning(
        'unknown_ambiguous_completion',
        c.name,
        'Doc missing Expected result section',
        'Add model-facing result guidance.',
        { doc_path: c.doc_path },
      );
    if (!doc.includes('failure_class')) pushWarning('unknown_ambiguous_completion', c.name, 'Doc does not mention failure_class', 'Document tool_result_v1 failure_class recovery.', { doc_path: c.doc_path });
  }
  for (const field of ['name','label','purpose','family','ontology_action','ontology_objects','side_effect_profile','result_envelope','live_check']) {
    if (c[field] == null || (Array.isArray(c[field]) && c[field].length === 0) || c[field] === '') pushFailure('validation_rejected', c.name, `Contract missing ${field}`, 'Complete model-facing tool contract metadata.');
  }
}


// Spec96 traversal facade must never default to unbounded hot payloads.
const traverseRouteSrc = fs.existsSync(path.join(root, 'crates/focusa-api/src/routes/traverse.rs'))
  ? read('crates/focusa-api/src/routes/traverse.rs')
  : '';
if (!/bounded_window|bounded_metadata|full_payload_blocked_by_pressure|include_full_payload|force_full_payload/.test(traverseRouteSrc)) {
  pushFailure('unbounded_hot_traversal', 'focusa_traverse', 'Traversal facade lacks bounded window/full-payload guard evidence.', 'Route /v1/traverse through bounded helpers and explicit cold opt-in.');
}
if (!/"trajectory"|"metacognition"|"predictions"|"snapshots"/.test(traverseRouteSrc)) {
  pushFailure('missing_traversal_adapter', 'focusa_traverse', 'Traversal facade missing major surface adapters.', 'Wire trajectory/metacog/predictions/snapshots adapters with bounded defaults.');
}


const compactionSrc = fs.existsSync(path.join(root, 'apps/pi-extension/src/compaction.ts'))
  ? read('apps/pi-extension/src/compaction.ts')
  : '';
const workpointSrc = fs.existsSync(path.join(root, 'crates/focusa-api/src/routes/workpoint.rs'))
  ? read('crates/focusa-api/src/routes/workpoint.rs')
  : '';
const toolsSrc = fs.existsSync(path.join(root, 'apps/pi-extension/src/tools.ts'))
  ? read('apps/pi-extension/src/tools.ts')
  : '';

if (!/WorkpointResumePacketV2|Never use transcript tail as authority|transcript tail as authority/.test(compactionSrc)) {
  pushFailure('transcript_tail_canonical_resume', 'focusa_workpoint_resume', 'Compaction resume can fall back to transcript-tail authority.', 'Inject WorkpointResumePacketV2 and explicit transcript-tail do_not_use guidance.');
}
if (!/resume_packet_v2|traversal_slices|api_provenance|failure_class/.test(workpointSrc)) {
  pushFailure('missing_resume_packet_v2_provenance', 'focusa_workpoint_resume', 'Workpoint Resume Packet v2 lacks traversal/provenance/failure taxonomy.', 'Render v2 structured packet with traversal_slices, api_provenance, and failure_class.');
}
if (!/verified_tags|stale_tags|invalid_tag_format|stale_or_missing_tag/.test(traverseRouteSrc)) {
  pushFailure('stale_tag_unverified', 'focusa_traverse', 'Traversal tag verification cannot prove stale or invalid tags.', 'Expose verified_tags and stale_tags with explicit stale/invalid reasons.');
}
if (!/timeoutFailureClassForRoute|cold_path_timeout|hot_path_timeout|daemon_unavailable/.test(toolsSrc)) {
  pushFailure('missing_traverse_timeout_taxonomy', 'focusa_traverse', 'Pi tool bridge lacks route-tier timeout taxonomy.', 'Classify cold-path timeout, hot-path timeout, and daemon unavailable distinctly.');
}


const requiredTraversalSurfaces = [
  'trajectory',
  'lineage',
  'ontology',
  'focus_stack',
  'evidence',
  'metacognition',
  'telemetry',
  'snapshots',
  'workpoints',
];
for (const surface of requiredTraversalSurfaces) {
  const pattern = new RegExp(`"${surface}"|${surface.replace('_', '[_-]')}`);
  if (!pattern.test(traverseRouteSrc)) {
    pushFailure('missing_surgical_surface', 'focusa_traverse', `Traversal facade missing ${surface} surface.`, 'Add a bounded focusa_traverse adapter with selector/limit/cursor/field projection.');
  }
}
if (!/budgeted_default_limit|budgeted_requested_limit|bounded_window|field_projection|full_payload_blocked_by_pressure/.test(traverseRouteSrc)) {
  pushFailure('missing_traversal_budget_controls', 'focusa_traverse', 'Traversal facade lacks complete budget controls.', 'Require default limits, requested limit caps, bounded windows, field projection, and cold full-payload guard.');
}

// Live registry and safe GET probes.
let health = null, liveRegistry = null, liveChoreography = null;
try { health = await getJsonWithRetry('/v1/health'); if (!health?.ok) pushFailure('daemon_unavailable', 'health', 'Daemon health non-ok', 'Check daemon service and API base URL.', health); }
catch (err) { pushFailure(err.name === 'TimeoutError' ? timeoutFailureClass('/v1/health') : 'daemon_unavailable', 'health', err.message, 'Check daemon service/API base URL and resource pressure.'); }
try { liveRegistry = await getJsonWithRetry('/v1/ontology/tool-contracts'); }
catch (err) { pushFailure(err.name === 'TimeoutError' ? timeoutFailureClass('/v1/ontology/tool-contracts') : 'daemon_unavailable', 'tool_contracts_live', err.message, 'Check ontology tool-contracts API route.'); }
try { liveChoreography = await getJsonWithRetry('/v1/ontology/tool-choreography'); }
catch (err) { pushFailure(err.name === 'TimeoutError' ? timeoutFailureClass('/v1/ontology/tool-choreography') : 'daemon_unavailable', 'tool_choreography_live', err.message, 'Check ontology tool-choreography API route.'); }
const sortJson = (value) => Array.isArray(value) ? value.map(sortJson) : value && typeof value === 'object' ? Object.fromEntries(Object.keys(value).sort().map(k => [k, sortJson(value[k])])) : value;
if (liveRegistry) {
  const payloadEqual = JSON.stringify(sortJson(liveRegistry)) === JSON.stringify(sortJson(registry));
  if (!payloadEqual) pushWarning('stale_runtime_registry', 'tool_contracts_live', 'Live daemon registry differs from static docs/source registry', 'Static validation remains source until approved rebuild/restart reloads daemon registry.', { static_count: contracts.length, live_count: (liveRegistry.contracts || []).length });
}
if (liveChoreography) {
  const stripRuntimeChoreography = (value) => {
    const clone = JSON.parse(JSON.stringify(value || {}));
    delete clone.runtime_weight_adjustments;
    delete clone.effective_edges;
    return clone;
  };
  const choreographyEqual = JSON.stringify(sortJson(stripRuntimeChoreography(liveChoreography))) === JSON.stringify(sortJson(stripRuntimeChoreography(choreography)));
  if (!choreographyEqual) pushWarning('stale_runtime_choreography', 'tool_choreography_live', 'Live daemon choreography base registry differs from static docs/source registry', 'Rebuild/restart daemon after choreography registry changes.', { static_edge_count: choreography.edge_count, live_edge_count: liveChoreography.edge_count });
  if (!liveChoreography.dynamic_weight_policy || !Array.isArray(liveChoreography.runtime_weight_adjustments)) pushFailure('validation_rejected', 'tool_choreography_live', 'Choreography route lacks dynamic weight policy/runtime adjustments', 'Expose dynamic weighting metadata even when no predictions are available.');
}

const getRoutes = new Set([
  '/v1/focus/frame/current',
  '/v1/trajectory/view',
  '/v1/workpoint/current',
  '/v1/work-loop/status?summary_only=true',
]);
const coldGetRoutes = new Set([]);
const includeColdGets = process.env.FOCUSA_AUDIT_INCLUDE_COLD_GET === '1';
for (const c of contracts) {
  for (const route of c.api_routes || []) {
    if (routeMethod(route) === 'GET' && !route.includes('{')) getRoutes.add(route.replace(/^GET\s+/, ''));
  }
}
for (const endpoint of [...getRoutes].sort()) {
  const endpointPath = endpoint.split('?')[0];
  if ((coldGetRoutes.has(endpointPath) || endpoint.includes('include_full_payload=true')) && !includeColdGets) {
    skippedColdGetRoutes.push({ endpoint, reason: 'cold_get_skipped_by_default_for_low_memory_reliability' });
    continue;
  }
  try {
    const body = await getJsonWithRetry(endpoint);
    if (endpoint.startsWith('/v1/focus/frame/current') && body?.active_frame_id && !body?.frame) {
      pushWarning('frame_unavailable', endpoint, 'Focus current-frame route returned active_frame_id but no frame', 'Route should fall back to active frame when no scoped query is provided, or safe fixture must pass frame/session query.', body);
    }
    if (endpoint.startsWith('/v1/work-loop/status') && !body?.status && !body?.work_loop?.status) {
      pushWarning('unknown_ambiguous_completion', endpoint, 'Work-loop summary lacks status field', 'Ensure summary route returns bounded status and writer fields.', body);
    }
    if (endpoint.startsWith('/v1/workpoint/current') && body?.status && body?.workpoint?.status && body.status !== body.workpoint.status) {
      pushWarning('read_model_lag', endpoint, `Workpoint route status mismatch: envelope=${body.status} nested=${body.workpoint.status}`, 'Use nested workpoint status for canonical object state; wrapper should not block solely on envelope status.', { envelope_status: body.status, workpoint_status: body.workpoint.status, workpoint_id: body.workpoint_id });
    }
    if (endpoint.startsWith('/v1/trajectory/view') && !body?.project_identity) {
      pushWarning('unknown_ambiguous_completion', endpoint, 'Trajectory view lacks project_identity', 'Trajectory must be per-project and ProjectIdentity-gated.', body);
    }
  } catch (err) {
    if (err.name === 'TimeoutError') {
      pushFailure(timeoutFailureClass(endpoint), endpoint, err.message, 'Move cold work off hot route or lower payload/lock work.');
    } else if (typeof err.message === 'string' && /\bHTTP 4\d\d\b/.test(err.message)) {
      // #305: a 403 with a structured authority/entitlement body is a policy
      // gate, not a validation gap and not a daemon failure.
      if (err?.status === 403 || /\bentitlement\b|\bauthority\b/.test(String(err?.body || err?.message || ''))) {
        pushWarning('entitlement_blocked', endpoint, `hot probe is authority-gated (healthy, unactivated daemon): ${err.message}`, 'Activate authority or record typed authority-gated coverage; do not classify as daemon_unavailable.');
      } else {
        // 4xx means the route exists and responded with a validation error
        // (e.g. required query param missing). Treat as a soft warning, not a
        // daemon failure, because the route is alive.
        pushWarning('probe_validation_expected', endpoint, `hot probe returned 4xx (expected for required-param routes): ${err.message}`, 'If a route legitimately requires query params, this warning is expected; otherwise add default query to the audit probe.');
      }
    } else {
      pushFailure('daemon_unavailable', endpoint, err.message, 'Fix route availability or mark contract with explicit exemption.');
    }
  }
}

// Post-probe memory check catches supposedly-safe probes that grow RSS.
checkDaemonProcessMemory();

// Known live Pi runtime probes from this session cannot be scripted here; document externally via direct tool audit results.
const result = {
  status: failures.length ? 'failed' : warnings.length ? 'passed_with_warnings' : 'passed',
  static_validation: staticResult,
  health: health ? { ok: health.ok, version: health.version } : null,
  contract_count: contracts.length,
  get_route_count: getRoutes.size,
  skipped_cold_get_routes: skippedColdGetRoutes,
  latency_guardrails: latencyGuardrails,
  latency_guardrail_config: { hot_warn_ms: hotRouteWarnMs, hot_fail_ms: hotRouteFailMs, cold_timeout_ms: coldRouteTimeoutMs, hot_latency_samples: hotRouteLatencySamples },
  process_checks: processChecks,
  probes,
  failures,
  warnings,
};
console.log(JSON.stringify(result, null, 2));
process.exit(failures.length ? 1 : 0);
