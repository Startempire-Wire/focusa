<script lang="ts">
  import type { Snippet } from 'svelte';
  import ProjectionLayoutRenderer from './ProjectionLayoutRenderer.svelte';
  import type { LayoutNode, ResolvedContribution } from './types';

  let {
    node,
    contributions,
    renderContribution,
    onSelectTab
  }: {
    node: LayoutNode;
    contributions: ReadonlyMap<string, ResolvedContribution>;
    renderContribution: Snippet<[ResolvedContribution]>;
    onSelectTab?: (contributionId: string) => void;
  } = $props();

  function contribution(id: string): ResolvedContribution | undefined {
    return contributions.get(id);
  }

  function activeTab(ids: string[], canonicalActive: string): string | undefined {
    if (ids.includes(canonicalActive)) return canonicalActive;
    return ids[0];
  }
</script>

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
    style={`--split-ratio:${Math.max(0.1, Math.min(0.9, node.ratio))}`}
  >
    {#each node.children as child, index (`${child.node_id}:${index}`)}
      <div class="split-child" class:first={index === 0}>
        <ProjectionLayoutRenderer node={child} {contributions} {renderContribution} {onSelectTab}/>
      </div>
    {/each}
  </div>
{:else if node.kind === 'stack'}
  <div class="layout-stack" data-layout-node={node.node_id} data-gap-token={node.gap_token ?? 'default'}>
    {#each node.children as child (`${child.node_id}`)}
      <ProjectionLayoutRenderer node={child} {contributions} {renderContribution} {onSelectTab}/>
    {/each}
  </div>
{:else if node.kind === 'grid'}
  <div class="layout-grid" data-layout-node={node.node_id} style={`--layout-columns:${Math.max(1, node.columns)}`}>
    {#each node.children as child (`${child.node_id}`)}
      <ProjectionLayoutRenderer node={child} {contributions} {renderContribution} {onSelectTab}/>
    {/each}
  </div>
{:else if node.kind === 'tabs'}
  {@const availableIds = node.contribution_ids.filter((id) => contributions.has(id))}
  {@const selectedId = activeTab(availableIds, node.active_contribution_id)}
  {#if selectedId}
    {@const selected = contribution(selectedId)}
    <div class="layout-tabs" class:interactive={Boolean(onSelectTab)} data-layout-node={node.node_id}>
      {#if onSelectTab}
        <div class="tab-list" role="tablist" aria-label="Canvas contribution tabs">
          {#each availableIds as id}
            {@const item = contribution(id)}
            {#if item}
              <button
                type="button"
                role="tab"
                aria-selected={id === selectedId}
                onclick={() => onSelectTab(id)}
              >{item.accessibility.label}</button>
            {/if}
          {/each}
        </div>
      {/if}
      {#if selected}
        <div role="tabpanel" data-contribution-id={selected.contribution_id}>
          {@render renderContribution(selected)}
        </div>
      {/if}
    </div>
  {/if}
{:else if node.kind === 'inspector'}
  {@const inspectorItems = node.inspector_contribution_ids.map(contribution).filter((item): item is ResolvedContribution => Boolean(item))}
  <div
    class:end={node.side === 'end'}
    class="layout-inspector"
    data-layout-node={node.node_id}
    style={`--inspector-span:${Math.max(1, node.span ?? 4)}`}
  >
    <main><ProjectionLayoutRenderer node={node.primary} {contributions} {renderContribution} {onSelectTab}/></main>
    {#if inspectorItems.length > 0}
      <aside aria-label="Canvas inspector">
        {#each inspectorItems as item (item.contribution_id)}
          <div data-contribution-id={item.contribution_id}>{@render renderContribution(item)}</div>
        {/each}
      </aside>
    {/if}
  </div>
{/if}

<style>
  .layout-single{min-width:0;min-height:0;height:100%}
  .layout-split{min-width:0;min-height:0;display:grid;grid-template-columns:minmax(0,calc(var(--split-ratio) * 100%)) minmax(0,1fr);gap:var(--layout-cluster-gap)}
  .layout-split.vertical{grid-template-columns:1fr;grid-template-rows:minmax(0,calc(var(--split-ratio) * 100%)) minmax(0,1fr)}
  .split-child{min-width:0;min-height:0}
  .layout-stack{min-width:0;min-height:0;display:grid;gap:var(--layout-cluster-gap)}
  .layout-grid{min-width:0;min-height:0;display:grid;grid-template-columns:repeat(var(--layout-columns),minmax(0,1fr));gap:var(--layout-cluster-gap)}
  .layout-tabs{min-width:0;min-height:0;display:grid;grid-template-rows:minmax(0,1fr)}
  .layout-tabs.interactive{grid-template-rows:auto minmax(0,1fr);gap:var(--space-2)}
  .tab-list{display:flex;align-items:center;gap:var(--space-1);overflow-x:auto}
  .tab-list button{min-height:28px;padding:0 var(--space-2);border:1px solid var(--color-border);border-radius:var(--radius-control);color:var(--color-text-tertiary);background:transparent;font:var(--type-caption);white-space:nowrap;cursor:pointer}
  .tab-list button[aria-selected='true']{color:var(--color-text);border-color:var(--color-border-strong);background:var(--color-raised)}
  .layout-inspector{min-width:0;min-height:0;display:grid;grid-template-columns:minmax(0,1fr) minmax(220px,calc(var(--inspector-span) * 4%));gap:var(--layout-cluster-gap)}
  .layout-inspector:not(.end){grid-template-columns:minmax(220px,calc(var(--inspector-span) * 4%)) minmax(0,1fr)}
  .layout-inspector:not(.end)>main{grid-column:2;grid-row:1}.layout-inspector:not(.end)>aside{grid-column:1;grid-row:1}
  .layout-inspector main,.layout-inspector aside{min-width:0;min-height:0}.layout-inspector aside{display:grid;align-content:start;gap:var(--layout-cluster-gap)}
  @media(max-width:820px){.layout-split{grid-template-columns:1fr;grid-template-rows:auto}.layout-grid{grid-template-columns:1fr}.layout-inspector,.layout-inspector:not(.end){grid-template-columns:1fr}.layout-inspector:not(.end)>main,.layout-inspector:not(.end)>aside{grid-column:1;grid-row:auto}}
</style>
