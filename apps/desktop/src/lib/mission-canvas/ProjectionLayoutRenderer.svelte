<script lang="ts">
  import type { Snippet } from 'svelte';
  import ProjectionLayoutRenderer from './ProjectionLayoutRenderer.svelte';
  import { DEFAULT_CONTRIBUTION_REGISTRY } from './default-contribution-registry';
  import { resolveContributionRenderer, type ContributionRendererRegistry } from './contribution-renderers';
  import type { LayoutNode, ResolvedContribution } from './types';

  let {
    node,
    contributions,
    renderContribution,
    onSelectTab,
    registry = DEFAULT_CONTRIBUTION_REGISTRY
  }: {
    node: LayoutNode;
    contributions: ReadonlyMap<string, ResolvedContribution>;
    renderContribution: Snippet<[ResolvedContribution]>;
    onSelectTab?: (contributionId: string) => void;
    registry?: ContributionRendererRegistry;
  } = $props();

  /**
   * A layout node is only renderable when its exact contribution identity is
   * present in the canonical eligible map and resolves through the trusted
   * renderer registry.  The key/DTO identity check is deliberate: a caller
   * may not place a foreign contribution under a requested layout ID.
   */
  function contribution(id: unknown): ResolvedContribution | undefined {
    if (typeof id !== 'string' || id.length === 0 || id.trim() !== id) return undefined;
    const resolved = contributions.get(id);
    if (!resolved || resolved.contribution_id !== id) return undefined;
    return resolveContributionRenderer(registry, resolved) ? resolved : undefined;
  }

  /**
   * Invalid canonical geometry fails closed as one unit.  We do not filter or
   * reflow children locally: Core owns eligibility and recomposition.  Refusing
   * the malformed tree prevents parent split/stack/grid wrappers from leaving
   * reserved gaps around an omitted or untrusted contribution.
   */
  function canRenderNode(value: unknown, seen = new WeakSet<object>()): boolean {
    if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
    const object = value as Record<string, unknown>;
    if (seen.has(object)) return false;
    seen.add(object);
    if (typeof object.node_id !== 'string' || object.node_id.length === 0 || object.node_id.trim() !== object.node_id) return false;

    switch (object.kind) {
      case 'single':
        return contribution(object.contribution_id) !== undefined;
      case 'split': {
        const children = object.children;
        return (object.orientation === 'horizontal' || object.orientation === 'vertical')
          && typeof object.ratio === 'number'
          && Number.isFinite(object.ratio)
          && object.ratio >= 0.1
          && object.ratio <= 0.9
          && Array.isArray(children)
          && children.length === 2
          && children.every((child) => canRenderNode(child, seen));
      }
      case 'stack': {
        const children = object.children;
        return Array.isArray(children)
          && children.length > 0
          && children.every((child) => canRenderNode(child, seen));
      }
      case 'grid': {
        const children = object.children;
        return Number.isSafeInteger(object.columns)
          && Number(object.columns) >= 1
          && Number(object.columns) <= 12
          && Array.isArray(children)
          && children.length > 0
          && children.every((child) => canRenderNode(child, seen));
      }
      case 'tabs': {
        const ids = object.contribution_ids;
        return Array.isArray(ids)
          && ids.length > 0
          && ids.every((id) => contribution(id) !== undefined)
          && ids.includes(object.active_contribution_id);
      }
      case 'inspector': {
        const ids = object.inspector_contribution_ids;
        const span = object.span;
        return (object.side === 'start' || object.side === 'end')
          && Array.isArray(ids)
          && ids.length > 0
          && ids.every((id) => contribution(id) !== undefined)
          && (span === undefined || (Number.isSafeInteger(span) && Number(span) >= 1 && Number(span) <= 6))
          && canRenderNode(object.primary, seen);
      }
      default:
        return false;
    }
  }

  function activeTab(ids: string[], canonicalActive: string): string | undefined {
    return ids.includes(canonicalActive) ? canonicalActive : undefined;
  }

  function stackRole(child: LayoutNode): 'rail' | 'queue' | 'composer' | 'content' {
    const ids = child.kind === 'single'
      ? [child.contribution_id]
      : child.kind === 'tabs'
        ? child.contribution_ids
        : [];
    const kinds = ids.map((id) => contribution(id)?.kind).filter(Boolean);
    if (kinds.includes('work_rail')) return 'rail';
    if (kinds.length > 0 && kinds.every((kind) => kind === 'steering_queue' || kind === 'follow_up_queue')) return 'queue';
    if (kinds.includes('prompt_editor')) return 'composer';
    if (child.kind === 'split' && child.children.map(stackRole).every((role) => role === 'queue')) return 'queue';
    return 'content';
  }

  function navigateTabs(event: KeyboardEvent, ids: string[], selectedId: string): void {
    if (!onSelectTab || ids.length < 2) return;
    const current = ids.indexOf(selectedId);
    let next = current;
    if (event.key === 'ArrowRight' || event.key === 'ArrowDown') next = (current + 1) % ids.length;
    else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') next = (current - 1 + ids.length) % ids.length;
    else if (event.key === 'Home') next = 0;
    else if (event.key === 'End') next = ids.length - 1;
    else return;
    event.preventDefault();
    onSelectTab(ids[next]);
  }
</script>

{#if canRenderNode(node)}
  {#if node.kind === 'single'}
  {@const resolved = contribution(node.contribution_id)}
  {#if resolved}
    <div class="layout-single" data-layout-node={node.node_id} data-contribution-id={resolved.contribution_id}>
      {@render renderContribution(resolved)}
    </div>
  {/if}
{:else if node.kind === 'split'}
  <div
    class:vertical={node.orientation === 'vertical'}
    class="layout-split"
    data-layout-node={node.node_id}
    style={`--split-ratio:${Math.max(0.1, Math.min(0.9, node.ratio))};--split-tail-count:${Math.max(1, node.children.length - 1)}`}
  >
    {#each node.children as child, index (`${child.node_id}:${index}`)}
      <div class="split-child" class:first={index === 0}>
        <ProjectionLayoutRenderer node={child} {contributions} {renderContribution} {onSelectTab} {registry}/>
      </div>
    {/each}
  </div>
{:else if node.kind === 'stack'}
  <div class="layout-stack" data-layout-node={node.node_id} data-gap-token={node.gap_token ?? 'default'}>
    {#each node.children as child (`${child.node_id}`)}
      <div class="stack-child" class:rail-region={stackRole(child) === 'rail'} class:queue-region={stackRole(child) === 'queue'} class:composer-region={stackRole(child) === 'composer'}>
        <ProjectionLayoutRenderer node={child} {contributions} {renderContribution} {onSelectTab} {registry}/>
      </div>
    {/each}
  </div>
{:else if node.kind === 'grid'}
  <div class="layout-grid" data-layout-node={node.node_id} style={`--layout-columns:${Math.max(1, node.columns)}`}>
    {#each node.children as child (`${child.node_id}`)}
      <ProjectionLayoutRenderer node={child} {contributions} {renderContribution} {onSelectTab} {registry}/>
    {/each}
  </div>
{:else if node.kind === 'tabs'}
  {@const availableIds = node.contribution_ids.filter((id) => contribution(id))}
  {@const selectedId = activeTab(availableIds, node.active_contribution_id)}
  {#if selectedId}
    {@const selected = contribution(selectedId)}
    <div class="layout-tabs" class:with-strip={availableIds.length > 1} class:interactive={Boolean(onSelectTab)} data-layout-node={node.node_id}>
      {#if availableIds.length > 1}
        <div class="tab-list" role="tablist" aria-label="Work Surfaces">
          {#each availableIds as id}
            {@const item = contribution(id)}
            {#if item}
              <button
                type="button"
                role="tab"
                aria-selected={id === selectedId}
                aria-controls={`panel-${node.node_id}-${id}`}
                tabindex={id === selectedId ? 0 : -1}
                disabled={!onSelectTab}
                onclick={() => onSelectTab?.(id)}
                onkeydown={(event) => navigateTabs(event, availableIds, selectedId)}
              >{item.accessibility.label}</button>
            {/if}
          {/each}
        </div>
      {/if}
      {#if selected}
        <div id={`panel-${node.node_id}-${selected.contribution_id}`} role="tabpanel" data-contribution-id={selected.contribution_id}>
          {@render renderContribution(selected)}
        </div>
      {/if}
    </div>
  {/if}
{:else if node.kind === 'inspector'}
  {@const inspectorItems = node.inspector_contribution_ids.map(contribution).filter((item): item is ResolvedContribution => Boolean(item))}
  <div
    class:end={node.side === 'end'}
    class:has-inspector={inspectorItems.length > 0}
    class="layout-inspector"
    data-layout-node={node.node_id}
    style={`--inspector-span:${Math.max(1, node.span ?? 4)}`}
  >
    <main><ProjectionLayoutRenderer node={node.primary} {contributions} {renderContribution} {onSelectTab} {registry}/></main>
    {#if inspectorItems.length > 0}
      <aside aria-label="Canvas inspector">
        {#each inspectorItems as item (item.contribution_id)}
          <div data-contribution-id={item.contribution_id}>{@render renderContribution(item)}</div>
        {/each}
      </aside>
    {/if}
  </div>
  {/if}
{/if}

<style>
  .layout-single{min-width:0;min-height:0;height:100%}
  .layout-split{min-width:0;min-height:0;height:100%;display:grid;grid-template-columns:minmax(0,calc(var(--split-ratio) * 100%)) repeat(var(--split-tail-count),minmax(0,1fr));gap:var(--layout-cluster-gap)}
  .layout-split.vertical{grid-template-columns:1fr;grid-template-rows:minmax(0,calc(var(--split-ratio) * 100%)) repeat(var(--split-tail-count),minmax(0,1fr))}
  .split-child{min-width:0;min-height:0}
  .layout-stack{min-width:0;min-height:0;height:100%;display:flex;flex-direction:column;gap:var(--layout-cluster-gap)}
  .stack-child{min-width:0;min-height:0;flex:1 1 auto}.stack-child.rail-region,.stack-child.queue-region{flex:0 0 auto}.stack-child.composer-region{flex:0 1 30%;min-height:140px}
  .layout-grid{min-width:0;min-height:0;display:grid;grid-template-columns:repeat(var(--layout-columns),minmax(0,1fr));gap:var(--layout-cluster-gap)}
  .layout-tabs{min-width:0;min-height:0;display:grid;grid-template-rows:minmax(0,1fr)}
  .layout-tabs.with-strip{grid-template-rows:auto minmax(0,1fr);gap:var(--space-2)}
  .tab-list{display:flex;align-items:center;gap:var(--space-1);overflow-x:auto}
  .tab-list button{min-height:28px;padding:0 var(--space-2);border:1px solid var(--color-border);border-radius:var(--radius-control);color:var(--color-text-tertiary);background:transparent;font:var(--type-caption);white-space:nowrap;cursor:pointer}
  .tab-list button[aria-selected='true']{color:var(--color-text);border-color:var(--color-border-strong);background:var(--color-raised)}
  .tab-list button:disabled{cursor:default;opacity:1}
  .layout-inspector{min-width:0;min-height:0;display:grid;grid-template-columns:minmax(0,1fr)}
  .layout-inspector.has-inspector.end{grid-template-columns:minmax(0,1fr) minmax(220px,calc(var(--inspector-span) * 4%));gap:var(--layout-cluster-gap)}
  .layout-inspector.has-inspector:not(.end){grid-template-columns:minmax(220px,calc(var(--inspector-span) * 4%)) minmax(0,1fr);gap:var(--layout-cluster-gap)}
  .layout-inspector.has-inspector:not(.end)>main{grid-column:2;grid-row:1}.layout-inspector:not(.end)>aside{grid-column:1;grid-row:1}
  .layout-inspector main,.layout-inspector aside{min-width:0;min-height:0}.layout-inspector aside{display:grid;align-content:start;gap:var(--layout-cluster-gap)}
  @container mission-canvas (max-width:820px){.layout-split{grid-template-columns:1fr;grid-template-rows:auto}.layout-grid{grid-template-columns:1fr}.layout-inspector,.layout-inspector:not(.end){grid-template-columns:1fr}.layout-inspector:not(.end)>main,.layout-inspector:not(.end)>aside{grid-column:1;grid-row:auto}}
</style>
