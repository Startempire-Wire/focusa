<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchJson } from '$lib/api';

  interface ToolContract {
    name: string;
    label?: string;
    purpose?: string;
    family?: string;
    doc_path?: string;
    api_routes?: string[];
    cli_commands?: string[];
  }

  let tools = $state<ToolContract[]>([]);
  let toolCount = $state<number>(0);
  let familyCounts = $state<Record<string, number>>({});
  let loading = $state<boolean>(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      const resp = await fetchJson<{ contracts: ToolContract[]; tool_count: number }>(
        '/v1/ontology/tool-contracts',
        5000,
      );
      tools = Array.isArray(resp?.contracts) ? resp.contracts : [];
      toolCount = resp?.tool_count || tools.length;
      const counts: Record<string, number> = {};
      for (const t of tools) {
        const f = t.family || 'unknown';
        counts[f] = (counts[f] || 0) + 1;
      }
      familyCounts = counts;
    } catch (e) {
      error = e instanceof Error ? e.message : 'fetch failed';
    } finally {
      loading = false;
    }
  });

  function text(v: any, fallback = 'unknown') {
    if (v === null || v === undefined || v === '') return fallback;
    if (Array.isArray(v)) return v.length ? v.join(', ') : fallback;
    if (typeof v === 'string') return v;
    return String(v);
  }
</script>

<section class="tools-registry-peek" aria-label="Spec 90 tool registry">
  <header class="peek-header">
    <div>
      <div class="eyebrow">Spec 90 · Tool Registry</div>
      <div class="value">
        {loading ? 'loading…' : error ? 'unavailable' : `${toolCount} tools`}
      </div>
      <div class="meta">
        auto-rendered from <code>/v1/ontology/tool-contracts</code> (live)
      </div>
    </div>
    <span class="chip" class:ok={!error && !loading} class:bad={!!error}>
      {error ? 'err' : loading ? '...' : 'live'}
    </span>
  </header>

  {#if familyCounts && Object.keys(familyCounts).length > 0}
    <div class="proof-row">
      <section>
        <div class="label">Family distribution</div>
        <ul>
          {#each Object.entries(familyCounts).sort((a, b) => b[1] - a[1]) as [family, count]}
            <li><code>{family}</code> = {count}</li>
          {/each}
        </ul>
      </section>
    </div>
  {/if}

  {#if tools.length > 0}
    <div class="proof-row">
      <section>
        <div class="label">Recent tools (top 8)</div>
        <ul>
          {#each tools.slice(0, 8) as tool}
            <li>
              <code>{tool.name}</code>
              <span class="muted">({tool.family || 'unknown'})</span>
            </li>
          {/each}
        </ul>
      </section>
    </div>
  {/if}

  {#if error}
    <p class="muted">error: {error}</p>
  {/if}
</section>

<style>
  .tools-registry-peek {
    padding: var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }
  .peek-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--sp-3);
  }
  .eyebrow,
  .label {
    font-size: 0.7rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--fg-muted);
  }
  .value {
    font-size: 1.05rem;
    font-weight: 600;
  }
  .meta {
    font-size: 0.8rem;
    color: var(--fg-muted);
  }
  .chip {
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 0.75rem;
    background: var(--bg-2);
    color: var(--fg-muted);
  }
  .chip.ok { background: var(--ok); color: var(--bg-0); }
  .chip.bad { background: var(--bad); color: var(--bg-0); }
  .proof-row {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--sp-2);
  }
  .proof-row section p,
  .proof-row section ul {
    font-size: 0.85rem;
  }
  .muted { color: var(--fg-muted); font-size: 0.8rem; }
  code {
    font-family: var(--mono);
    font-size: 0.78rem;
  }
</style>
