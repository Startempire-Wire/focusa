<script lang="ts">
  import StatusBadge from '$lib/ui/StatusBadge.svelte';
  import type { OperationBinding, ResolvedContribution, ResolvedWorkspaceProjection } from '../types';
  import {
    ArtifactRenderer,
    BrowserArtifactRef,
    browserSessionContextFromDescriptor,
    parseBrowserArtifactDescriptor,
    type BrowserArtifactDescriptor,
    type UIAISessionRef
  } from '../browser-artifact';

  let {
    contribution,
    projection
  }: {
    contribution: ResolvedContribution;
    projection: ResolvedWorkspaceProjection;
    onOperation?: (binding: OperationBinding) => void | Promise<void>;
  } = $props();

  const tone = $derived(contribution.freshness.status === 'current' ? 'ready' : contribution.freshness.status === 'stale' ? 'watch' : 'neutral');
  const artifact = $derived(parseBrowserArtifactDescriptor(contribution.data_ref.ref));
  const isStructuredArtifact = $derived(Boolean(artifact && BrowserArtifactRef.validate(artifact)));
  const sessionContext = $derived(isStructuredArtifact
    ? browserSessionContextFromDescriptor(artifact as BrowserArtifactDescriptor)
    : ({ session_origin: contribution.data_ref.ref } as UIAISessionRef));

  const lines = $derived(() => isStructuredArtifact
    ? ArtifactRenderer.render(artifact as BrowserArtifactDescriptor)
    : [`Unable to parse browser artifact descriptor`, `Data reference: ${contribution.data_ref.ref}`]
  );

  const hasExactBrowserContext = $derived(
    sessionContext && (Boolean(sessionContext.browser_context_ref) || Boolean(sessionContext.browser_context_id))
  );
  const hasUiTarget = $derived(Boolean(sessionContext.browser_target_ref) || Boolean(sessionContext.browser_target_id));
  const sessionIdLine = $derived(sessionContext.session_origin || 'not-reported');
</script>

<section class="browser-artifact" aria-label={contribution.accessibility.label} data-browser-artifact={contribution.data_ref.ref}>
  <header>
    <div class="identity">
      <strong>{contribution.accessibility.label}</strong>
      {#if contribution.accessibility.description}<span>{contribution.accessibility.description}</span>{/if}
    </div>
    <StatusBadge {tone} label={contribution.freshness.status}/>
  </header>

  <div class="artifact-card">
    <div class="artifact-metadata" aria-label="Browser identity">
      <p data-browser-session={sessionIdLine}>UIAI session: {sessionIdLine}</p>
      <p data-browser-context={
        hasExactBrowserContext
          ? (sessionContext.browser_context_ref || sessionContext.browser_context_id)
          : 'not available'
      }>
        Browser context: {hasExactBrowserContext
          ? (sessionContext.browser_context_ref || sessionContext.browser_context_id)
          : 'not available'}
      </p>
      <p data-browser-target={hasUiTarget ? (sessionContext.browser_target_ref || sessionContext.browser_target_id) : 'none'}>
        Browser target: {hasUiTarget ? (sessionContext.browser_target_ref || sessionContext.browser_target_id) : 'not available'}
      </p>
      <p data-descriptor-artifact={artifact?.artifact_id || 'unresolved'}>
        Artifact: {artifact?.artifact_id || 'unresolved'}
      </p>
    </div>
    <pre class="artifact-render-lines" aria-label="Browser artifact render output">
      {#each lines() as line, index (index)}
        <span data-browser-artifact-line={index}>{line}</span>
      {/each}
    </pre>
    <p class="policy-note" data-browser-prompt="uiai-owned">
      Desktop is a metadata renderer only. Browser execution and state remain owned by UIAI Engine.
    </p>
  </div>
</section>

<style>
  .browser-artifact{display:grid;grid-template-rows:auto 1fr;min-width:0;height:100%;border:1px solid var(--color-border);border-radius:var(--radius-panel);background:var(--color-panel);overflow:hidden}
  header,.actions{display:flex;align-items:center;gap:var(--space-2)}
  header{justify-content:space-between;padding:var(--space-3) var(--space-4);border-bottom:1px solid var(--color-border)}
  .identity{display:grid;gap:var(--space-1);min-width:0}
  .identity strong{color:var(--color-text);font:var(--type-label)}
  .identity span{overflow:hidden;color:var(--color-text-tertiary);font:var(--type-caption);text-overflow:ellipsis;white-space:nowrap}
  .artifact-card{display:grid;gap:var(--space-3);padding:var(--layout-card-padding-roomy)}
  .artifact-metadata{display:grid;gap:var(--space-2)}
  .artifact-metadata p{font:var(--type-caption);color:var(--color-text-secondary);margin:0}
  .artifact-metadata p strong{color:var(--color-text)}
  .artifact-render-lines{display:grid;gap:var(--space-1);white-space:pre-wrap;line-height:var(--line-height-body);font:var(--type-code);color:var(--color-text-tertiary);margin:0;padding:var(--space-2);border:1px solid var(--color-border);border-radius:var(--radius-control);background:var(--color-panel-2)}
  .artifact-render-lines span{display:block}
  .policy-note{margin:0;color:var(--color-text-tertiary);font:var(--type-caption)}
</style>
