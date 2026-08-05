<script lang="ts">
  import { tick } from 'svelte';
  import { fade } from 'svelte/transition';
  import Icon from './Icon.svelte';
  import { pop } from './motion';
  import { filterPresentationCommands, type PresentationCommand } from '$lib/shell/command-manifest';

  let { open = $bindable(false), onSelect }: { open?: boolean; onSelect: (command: PresentationCommand) => void } = $props();
  let query = $state('');
  let activeIndex = $state(0);
  let input = $state<HTMLInputElement>();
  let commands = $derived(filterPresentationCommands(query));

  $effect(() => {
    if (open) void tick().then(() => input?.focus());
    else { query = ''; activeIndex = 0; }
  });

  function select(command: PresentationCommand | undefined) {
    if (!command) return;
    onSelect(command);
    open = false;
  }
  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') { event.preventDefault(); open = false; return; }
    if (!commands.length) return;
    if (event.key === 'ArrowDown') { event.preventDefault(); activeIndex = (activeIndex + 1) % commands.length; }
    if (event.key === 'ArrowUp') { event.preventDefault(); activeIndex = (activeIndex - 1 + commands.length) % commands.length; }
    if (event.key === 'Home') { event.preventDefault(); activeIndex = 0; }
    if (event.key === 'End') { event.preventDefault(); activeIndex = commands.length - 1; }
    if (event.key === 'Enter') { event.preventDefault(); select(commands[activeIndex] ?? commands[0]); }
  }
</script>

{#if open}
  <div class="command-backdrop" role="presentation" transition:fade={{ duration: 140 }} onclick={(event) => event.target === event.currentTarget && (open = false)}>
    <dialog open class="command-palette" aria-label="Find or do" in:pop={{ duration: 190, y: 5 }} out:pop={{ duration: 105, y: 2 }}>
      <header><span><Icon name="search" size={16}/>Find or do</span><small>Presentation commands</small><kbd>Esc</kbd></header>
      <div class="command-input"><Icon name="search" size={18}/><input bind:this={input} bind:value={query} aria-label="Search Focusa Desktop commands" placeholder="Search workspaces and interface actions…" oninput={() => (activeIndex = 0)} onkeydown={onKeydown}/></div>
      <div class="result-status" aria-live="polite"><span>{commands.length} {commands.length === 1 ? 'result' : 'results'}</span><span>↑↓ Navigate · ↵ Run</span></div>
      {#if commands.length}
        <div class="command-list" role="listbox" aria-label="Presentation commands">
          {#each commands as command, index}
            <button type="button" role="option" aria-selected={activeIndex === index} class:active={activeIndex === index} onmouseenter={() => (activeIndex = index)} onclick={() => select(command)}>
              <span><strong>{command.label}</strong><small>{command.hint}</small></span><em>{command.authority}</em>
            </button>
          {/each}
        </div>
      {:else}<p class="empty">No matching presentation commands.</p>{/if}
    </dialog>
  </div>
{/if}

<style>
  .command-backdrop{position:fixed;inset:0;z-index:80;display:grid;place-items:start center;padding:clamp(90px,16vh,160px) var(--space-4);background:rgb(0 0 0 / 38%);backdrop-filter:blur(8px)}
  .command-palette{position:relative;inset:auto;width:min(540px,calc(100vw - 32px));max-height:min(620px,calc(84vh - 32px));margin:0;display:flex;flex-direction:column;overflow:hidden;padding:0;border:1px solid var(--color-border-strong);border-radius:var(--radius-card);color:var(--color-text);background:var(--color-elevated);box-shadow:var(--shadow-popover);transform-origin:top center}
  header{display:flex;align-items:center;gap:var(--space-3);padding:var(--space-3) var(--space-4) var(--space-2);color:var(--color-text-tertiary);font:var(--type-caption)}header>span{display:flex;align-items:center;gap:var(--space-2);margin-right:auto;color:var(--color-text-soft)}header small{font-size:9px}kbd{padding:2px 5px;border:1px solid var(--color-border);border-radius:4px;color:var(--color-text-tertiary);background:var(--color-raised);font-size:9px}
  .command-input{display:grid;grid-template-columns:22px 1fr;align-items:center;gap:var(--space-2);margin:0 var(--space-4);padding:var(--space-2) 0;border-bottom:1px solid var(--color-border);color:var(--color-accent-bright)}input{width:100%;padding:0;border:0;outline:0;color:var(--color-text);background:transparent;font:var(--type-body)}input::placeholder{color:var(--color-text-tertiary)}
  .result-status{display:flex;justify-content:space-between;padding:var(--space-2) var(--space-4) var(--space-1);color:var(--color-text-tertiary);font-size:9px}.command-list{min-height:0;display:grid;gap:2px;overflow-y:auto;padding:var(--space-1) var(--space-2) var(--space-2);scrollbar-width:thin}.command-list button{position:relative;min-height:42px;display:flex;align-items:center;justify-content:space-between;gap:var(--space-4);padding:var(--space-2) var(--space-3);border:0;border-radius:var(--radius-control);color:var(--color-text);background:transparent;text-align:left;cursor:pointer;transition:background var(--motion-fast) var(--ease-out-quart),transform var(--motion-micro) var(--ease-out-quart)}.command-list button:hover,.command-list button.active{background:color-mix(in srgb,var(--color-accent) 13%,var(--color-raised))}.command-list button.active::before{position:absolute;inset-block:9px;left:0;width:2px;border-radius:2px;background:var(--color-accent);content:''}.command-list button:active{transform:scale(.98)}.command-list button>span{min-width:0;display:grid;gap:1px}.command-list strong{overflow:hidden;font:var(--type-caption);text-overflow:ellipsis;white-space:nowrap}.command-list small{overflow:hidden;color:var(--color-text-tertiary);font-size:9px;text-overflow:ellipsis;white-space:nowrap}.command-list em{flex:0 0 auto;color:var(--color-text-tertiary);font-size:8px;font-style:normal}.empty{margin:var(--space-4);color:var(--color-text-tertiary);font:var(--type-body)}
</style>
