#!/usr/bin/env node
// Read-only observations, not release acceptance. Never execute gates/tests or
// consume ambient evidence. The canonical release pipeline owns readiness.
import { execFileSync } from 'node:child_process';
import { realpathSync, statfsSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

const checkout = fileURLToPath(new URL('..', import.meta.url));
const code = error => String(error?.code || (error?.status ? `EXIT_${error.status}` : error?.name) || 'UNAVAILABLE');

export async function main({ legacy = false, argv = process.argv.slice(2) } = {}) {
  const result = {
    schema: legacy ? 'focusa.release_readiness_evaluation.v1' : 'focusa.release_diagnostics_advisory.v1',
    generated_at: new Date().toISOString(),
    authority: 'advisory_only', advisory: true, ready: false, release_ready: false,
    execution_admission: false,
    readiness_authority: 'canonical_release_pipeline',
    evidence_policy: 'no_gate_or_extension_evidence_evaluated',
    scope: { canonical: false, verification: 'repository_observation_only' },
    checks: [],
    ...(legacy ? { deprecated_entrypoint: 'release-readiness-evaluation.mjs',
      replacement: 'release-diagnostics-advisory.mjs',
      exit_semantics: 'diagnostic_completion_not_release_acceptance' } : {}),
  };
  const check = (name, status, detail, data = {}) => result.checks.push({
    name, status, ok: status === 'observed', detail, ...data,
  });
  try {
    const { values } = parseArgs({ args: argv, strict: true, allowPositionals: false, options: {
      json: { type: 'boolean' }, advisory: { type: 'boolean' },
      'project-root': { type: 'string' }, 'disk-path': { type: 'string' },
      'daemon-url': { type: 'string' }, 'expected-version': { type: 'string' },
    } });
    const requestedRoot = resolve(values['project-root'] || process.env.FOCUSA_PROJECT_ROOT || checkout);
    const diskPath = resolve(values['disk-path'] || requestedRoot);
    const endpoint = values['daemon-url'] || null;
    const expectedVersion = values['expected-version'] || process.env.FOCUSA_EXPECTED_VERSION || null;
    Object.assign(result.scope, { requested_project_root: requestedRoot, disk_path: diskPath });
    result.expected_version = expectedVersion;
    result.expected_version_source = expectedVersion ? 'explicit_argument_or_environment' : 'unavailable';
    try {
      const observedRoot = realpathSync(requestedRoot);
      const git = args => execFileSync('git', ['-C', observedRoot, ...args], {
        encoding: 'utf8', timeout: 2000, stdio: ['ignore', 'pipe', 'pipe'],
      }).trim();
      const gitRoot = realpathSync(git(['rev-parse', '--show-toplevel']));
      const commit = git(['rev-parse', '--verify', 'HEAD']);
      Object.assign(result.scope, { project_root: observedRoot, repository_root: gitRoot, commit_sha: commit });
      check('repository_scope', gitRoot === observedRoot ? 'observed' : 'error',
        gitRoot === observedRoot ? 'Exact repository root and commit observed; not canonical execution authority' : 'Requested directory is not the repository root');
    } catch (error) {
      check('repository_scope', 'unavailable', 'Repository identity could not be observed', { error_code: code(error) });
    }
    try {
      const fs = statfsSync(diskPath, { bigint: true });
      check('disk_observation', 'observed', 'Filesystem space observed; no headroom or readiness decision', {
        available_bytes: String(fs.bavail * fs.bsize), total_bytes: String(fs.blocks * fs.bsize),
      });
    } catch (error) {
      check('disk_observation', 'unavailable', 'Filesystem space could not be observed', { error_code: code(error) });
    }
    if (!endpoint) {
      check('daemon_health', 'unavailable', 'No explicit --daemon-url health endpoint supplied');
      check('runtime_version', 'unavailable', 'No runtime observation available');
    } else {
      let url;
      try {
        url = new URL(endpoint);
        if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password || url.hash || url.search) {
          throw new Error('Invalid endpoint');
        }
      } catch {
        url = null;
        check('daemon_health', 'error', 'Explicit health endpoint must be HTTP(S), without credentials, query or fragment', { error_code: 'INVALID_ENDPOINT' });
      }
      if (url) {
        result.daemon_endpoint = url.href;
        try {
          const response = await fetch(url, { signal: AbortSignal.timeout(2500), redirect: 'error' });
          if (!response.ok) {
            check('daemon_health', 'error', 'Health endpoint returned an HTTP failure', { http_status: response.status });
          } else {
            let body;
            try { body = await response.json(); }
            catch { throw Object.assign(new Error('Invalid health JSON'), { code: 'INVALID_HEALTH_JSON' }); }
            const healthy = body?.ok === true;
            check('daemon_health', healthy ? 'observed' : 'error', healthy ? 'Endpoint reports healthy; this is not release acceptance' : 'Endpoint did not report ok:true');
            const observedVersion = typeof body?.version === 'string' ? body.version : null;
            check('runtime_version', !expectedVersion || !observedVersion ? 'unavailable' :
              observedVersion === expectedVersion ? 'observed' : 'error',
              !expectedVersion ? 'No explicit expected version supplied' : !observedVersion ? 'Endpoint omitted version' :
                observedVersion === expectedVersion ? 'Observed runtime version matches explicit expectation' : 'Observed runtime version differs from explicit expectation',
              { observed_version: observedVersion, expected_version: expectedVersion });
          }
        } catch (error) {
          check('daemon_health', 'unavailable', 'Health observation failed', { error_code: code(error) });
        }
      }
      if (!result.checks.some(c => c.name === 'runtime_version')) {
        check('runtime_version', 'unavailable', 'No runtime version observation available');
      }
    }
    check('release_acceptance', 'unavailable', 'Not evaluated: use the canonical release pipeline and its exact evidence; stale logs cannot confer acceptance');
  } catch (error) {
    check('diagnostic_invocation', 'error', 'Diagnostics could not be evaluated', { error_code: code(error) });
    process.exitCode = 2;
  }
  console.log(JSON.stringify(result, null, 2));
  return result;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
