<script lang="ts">
  import type { Snippet } from 'svelte';
  import { validateMissionCanvasContract } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
  import ProjectionLayoutRenderer from './ProjectionLayoutRenderer.svelte';
  import { DEFAULT_CONTRIBUTION_REGISTRY } from './default-contribution-registry';
  import { resolveContributionRenderer, type ContributionRendererRegistry } from './contribution-renderers';
  import type { LayoutNode, ResolvedContribution, TabLayoutNode } from './types';

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
   *
   * The generated node validators own the DTO shape; this pass adds the
   * recursive invariants which the union validator cannot express: exactly two
   * split children, unique node/contribution identities, and one trusted
   * renderer for every contribution that would receive geometry.
   */
  type LayoutValidationState = {
    seenNodes: WeakSet<object>;
    nodeIds: Set<string>;
    contributionIds: Set<string>;
  };

  function createLayoutValidationState(): LayoutValidationState {
    return {
      seenNodes: new WeakSet<object>(),
      nodeIds: new Set<string>(),
      contributionIds: new Set<string>()
    };
  }

  function canRenderNode(
    value: unknown,
    state: LayoutValidationState = createLayoutValidationState()
  ): boolean {
    if (!isPlainRecord(value)) return false;
    const object = value as Record<string, unknown>;
    if (!hasOwnProperties(object, ['node_id', 'kind'])) return false;
    if (state.seenNodes.has(object)) return false;
    state.seenNodes.add(object);
    if (typeof object.node_id !== 'string' || object.node_id.length === 0 || object.node_id.trim() !== object.node_id) return false;
    if (state.nodeIds.has(object.node_id)) return false;
    state.nodeIds.add(object.node_id);

    const schema = object.kind === 'single'
      ? 'SingleLayoutNode'
      : object.kind === 'split'
        ? 'SplitLayoutNode'
        : object.kind === 'stack'
          ? 'StackLayoutNode'
          : object.kind === 'grid'
            ? 'GridLayoutNode'
            : object.kind === 'tabs'
              ? 'TabLayoutNode'
              : object.kind === 'inspector'
                ? 'InspectorLayoutNode'
                : undefined;
    if (!schema || !validateMissionCanvasContract(schema, object).valid) return false;

    const reserveContribution = (id: unknown): boolean => {
      if (typeof id !== 'string' || id.length === 0 || id.trim() !== id) return false;
      if (state.contributionIds.has(id) || !contribution(id)) return false;
      state.contributionIds.add(id);
      return true;
    };

    switch (object.kind) {
      case 'single':
        return hasOwnProperties(object, ['contribution_id'])
          && reserveContribution(object.contribution_id);
      case 'split': {
        const children = object.children;
        return hasOwnProperties(object, ['orientation', 'ratio', 'children'])
          && (object.orientation === 'horizontal' || object.orientation === 'vertical')
          && typeof object.ratio === 'number'
          && Number.isFinite(object.ratio)
          && object.ratio >= 0.1
          && object.ratio <= 0.9
          && isCanonicalArray(children)
          && children.length === 2
          && children.every((child) => canRenderNode(child, state));
      }
      case 'stack': {
        const children = object.children;
        return hasOwnProperties(object, ['children'])
          && isValidGapToken(ownValue(object, 'gap_token'))
          && isCanonicalArray(children)
          && children.length > 0
          && children.every((child) => canRenderNode(child, state));
      }
      case 'grid': {
        const children = object.children;
        return hasOwnProperties(object, ['columns', 'children'])
          && isValidGapToken(ownValue(object, 'gap_token'))
          && Number.isSafeInteger(object.columns)
          && Number(object.columns) >= 1
          && Number(object.columns) <= 12
          && isCanonicalArray(children)
          && children.length > 0
          && children.every((child) => canRenderNode(child, state));
      }
      case 'tabs': {
        const ids = object.contribution_ids;
        return hasOwnProperties(object, ['contribution_ids', 'active_contribution_id'])
          && isCanonicalArray(ids)
          && ids.length > 0
          && ids.includes(object.active_contribution_id)
          && ids.every((id) => reserveContribution(id));
      }
      case 'inspector': {
        const ids = object.inspector_contribution_ids;
        const span = object.span;
        return hasOwnProperties(object, ['side', 'primary', 'inspector_contribution_ids'])
          && (object.side === 'start' || object.side === 'end')
          && isCanonicalArray(ids)
          && ids.length > 0
          && (span === undefined || (Number.isSafeInteger(span) && Number(span) >= 1 && Number(span) <= 6))
          && canRenderNode(object.primary, state)
          && ids.every((id) => reserveContribution(id));
      }
      default:
        return false;
    }
  }

  function isPlainRecord(value: unknown): value is Record<string, unknown> {
    if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
    const prototype = Object.getPrototypeOf(value);
    return prototype === Object.prototype || prototype === null;
  }

  function hasOwnProperties(object: Record<string, unknown>, fields: readonly string[]): boolean {
    return fields.every((field) => Object.prototype.hasOwnProperty.call(object, field));
  }

  function ownValue(object: Record<string, unknown>, field: string): unknown {
    return Object.prototype.hasOwnProperty.call(object, field) ? object[field] : undefined;
  }

  /** JSON transport arrays are dense and have no non-index members. */
  function isCanonicalArray(value: unknown): value is unknown[] {
    if (!Array.isArray(value)) return false;
    for (const key of Reflect.ownKeys(value)) {
      if (key === 'length') continue;
      if (typeof key !== 'string' || !/^(0|[1-9]\d*)$/.test(key) || Number(key) >= value.length) return false;
    }
    for (let index = 0; index < value.length; index += 1) {
      if (!Object.prototype.hasOwnProperty.call(value, index)) return false;
    }
    return true;
  }

  function isValidGapToken(value: unknown): boolean {
    return value === undefined
      || (typeof value === 'string' && value.length > 0 && value.trim() === value);
  }

  function stackGapToken(node: LayoutNode): string {
    if (node.kind !== 'stack') return 'default';
    const value = ownValue(node as unknown as Record<string, unknown>, 'gap_token');
    return typeof value === 'string' ? value : 'default';
  }

  function gridGapToken(node: LayoutNode): string {
    if (node.kind !== 'grid') return 'default';
    const value = ownValue(node as unknown as Record<string, unknown>, 'gap_token');
    return typeof value === 'string' ? value : 'default';
  }

  /**
   * Resolve the complete canonical tab contribution list without filtering or
   * reordering it in the client.  `canRenderNode` has already checked every
   * ID, but keeping this all-or-nothing guard at the presentation boundary
   * means a malformed update can never leave a tab strip or panel behind.
   */
  function presentationTabs(node: TabLayoutNode): readonly ResolvedContribution[] | undefined {
    const items: ResolvedContribution[] = [];
    for (const id of node.contribution_ids) {
      const item = contribution(id);
      if (!item) return undefined;
      items.push(item);
    }
    return items;
  }

  /**
   * The active child is a canonical projection choice.  It is not a local tab
   * index or a client-side fallback: an invalid active ID renders no tab node.
   */
  function renderActiveChild(
    node: TabLayoutNode,
    items: readonly ResolvedContribution[]
  ): ResolvedContribution | undefined {
    return items.find((item) => item.contribution_id === node.active_contribution_id);
  }

  /**
   * Presentation selection is intentionally a one-way callback.  The parent
   * owns the generated `focusa.mission_canvas.layout.mutate` operation and
   * canonical active index; this helper never mutates the LayoutNode locally.
   */
  const presentationTab = {
    select(contributionId: string, items: readonly ResolvedContribution[]): void {
      if (!onSelectTab || !items.some((item) => item.contribution_id === contributionId)) return;
      onSelectTab(contributionId);
    }
  };

  function stackRole(child: LayoutNode): 'control' | 'rail' | 'queue' | 'composer' | 'content' {
    const ids = child.kind === 'single'
      ? [child.contribution_id]
      : child.kind === 'tabs'
        ? child.contribution_ids
        : [];
    const kinds = ids.map((id) => contribution(id)?.kind).filter(Boolean);
    if (kinds.length > 0 && kinds.every((kind) => kind === 'toolbar_control' || kind === 'transient_notification')) return 'control';
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
    data-layout-orientation={node.orientation}
    data-split-ratio={node.ratio}
    style={`--split-ratio:${node.ratio}`}
  >
    {#each node.children as child, index (`${child.node_id}:${index}`)}
      <div class="split-child" class:first={index === 0}>
        <ProjectionLayoutRenderer node={child} {contributions} {renderContribution} {onSelectTab} {registry}/>
      </div>
    {/each}
  </div>
{:else if node.kind === 'stack'}
  <div class="layout-stack" data-layout-node={node.node_id} data-gap-token={stackGapToken(node)}>
    {#each node.children as child, index (`${child.node_id}:${index}`)}
      <div data-stack-index={index} class="stack-child" class:control-region={stackRole(child) === 'control'} class:rail-region={stackRole(child) === 'rail'} class:queue-region={stackRole(child) === 'queue'} class:composer-region={stackRole(child) === 'composer'}>
        <ProjectionLayoutRenderer node={child} {contributions} {renderContribution} {onSelectTab} {registry}/>
      </div>
    {/each}
  </div>
{:else if node.kind === 'grid'}
  <div
    class="layout-grid"
    data-layout-node={node.node_id}
    data-layout-columns={node.columns}
    data-gap-token={gridGapToken(node)}
    style={`--layout-columns:${node.columns}`}
  >
    {#each node.children as child, index (`${child.node_id}:${index}`)}
      <div class="grid-child" data-grid-index={index}>
        <ProjectionLayoutRenderer node={child} {contributions} {renderContribution} {onSelectTab} {registry}/>
      </div>
    {/each}
  </div>
{:else if node.kind === 'tabs'}
  {@const tabItems = presentationTabs(node)}
  {#if tabItems}
    {@const selected = renderActiveChild(node, tabItems)}
    {@const tabIds = tabItems.map((item) => item.contribution_id)}
    {#if selected}
      <div class="layout-tabs" class:with-strip={tabItems.length > 1} class:interactive={Boolean(onSelectTab)} data-layout-node={node.node_id}>
        {#if tabItems.length > 1}
          <div class="tab-list" role="tablist" aria-label="Work Surfaces">
            {#each tabItems as item (item.contribution_id)}
              <button
                type="button"
                role="tab"
                aria-selected={item.contribution_id === selected.contribution_id}
                aria-controls={`panel-${node.node_id}-${item.contribution_id}`}
                tabindex={item.contribution_id === selected.contribution_id ? 0 : -1}
                disabled={!onSelectTab}
                onclick={() => presentationTab.select(item.contribution_id, tabItems)}
                onkeydown={(event) => navigateTabs(event, tabIds, selected.contribution_id)}
              >{item.accessibility.label}</button>
            {/each}
          </div>
        {/if}
        <div id={`panel-${node.node_id}-${selected.contribution_id}`} role="tabpanel" data-contribution-id={selected.contribution_id}>
          {@render renderContribution(selected)}
        </div>
      </div>
    {/if}
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
  .layout-split{min-width:0;min-height:0;height:100%;display:grid;grid-template-columns:minmax(0,calc(var(--split-ratio) * 100%)) minmax(0,1fr);gap:var(--layout-cluster-gap)}
  .layout-split.vertical{grid-template-columns:1fr;grid-template-rows:minmax(0,calc(var(--split-ratio) * 100%)) minmax(0,1fr)}
  .split-child{min-width:0;min-height:0}
  .layout-stack{min-width:0;min-height:0;height:100%;display:flex;flex-direction:column;gap:var(--layout-cluster-gap)}
  .stack-child{min-width:0;min-height:0;flex:1 1 auto}.stack-child.control-region,.stack-child.rail-region,.stack-child.queue-region{flex:0 0 auto}.stack-child.composer-region{flex:0 1 30%;min-height:140px}
  .layout-grid{min-width:0;min-height:0;height:100%;display:grid;grid-template-columns:repeat(var(--layout-columns),minmax(0,1fr));gap:var(--layout-cluster-gap)}
  .grid-child{min-width:0;min-height:0}
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
