<script lang="ts">
  import { onMount } from 'svelte';
  import FocusCanvas from '$lib/canvas/FocusCanvas.svelte';
  import AsccPanel from '$lib/canvas/AsccPanel.svelte';
  import Timeline from '$lib/canvas/Timeline.svelte';
  import ContextCognitionPeek from '$lib/components/ContextCognitionPeek.svelte';
  import ToolsRegistryPeek from '$lib/components/ToolsRegistryPeek.svelte';
  import { focusCanvasStore } from '$lib/stores/focus-canvas.svelte';
  
  let selectedEventId: string | null = null;
  let showAscc = true;
  let showTimeline = true;
  
  function handleFrameSelect(frameId: string) {
    focusCanvasStore.setActiveFrame(frameId);
  }
  
  function handleEventSelect(event: CustomEvent<{ eventId: string }>) {
    selectedEventId = event.detail.eventId;
  }
  
  function handleEventReplay(event: CustomEvent<{ eventId: string }>) {
    // Read-only canvas: replay is intentionally disabled until isolated fixtures exist.
    selectedEventId = event.detail.eventId;
  }
  
  onMount(() => {
    focusCanvasStore.loadLive();
  });
</script>

<div class="canvas-page">
  <header class="page-header">
    <h1>
      <svg viewBox="0 0 24 24" width="24" height="24">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z" fill="currentColor"/>
      </svg>
      Focus Canvas
    </h1>
    
    <div class="header-controls">
      <button 
        class="toggle-btn" 
        class:active={showAscc}
        aria-label={showAscc ? 'Hide ASCC panel' : 'Show ASCC panel'}
        aria-pressed={showAscc}
        on:click={() => showAscc = !showAscc}
      >
        ASCC
      </button>
      <button 
        class="toggle-btn" 
        class:active={showTimeline}
        aria-label={showTimeline ? 'Hide Timeline panel' : 'Show Timeline panel'}
        aria-pressed={showTimeline}
        on:click={() => showTimeline = !showTimeline}
      >
        Timeline
      </button>
    </div>
  </header>
  
  <section class="canvas-summary" aria-label="Canvas text summary">
    <strong>Canvas summary:</strong>
    {$focusCanvasStore.stack.frames.length} focus frames loaded;
    active frame {$focusCanvasStore.activeFrame?.title || 'none'};
    {$focusCanvasStore.events.length} timeline events available.
    Controls: ASCC panel {showAscc ? 'shown' : 'hidden'}, Timeline panel {showTimeline ? 'shown' : 'hidden'}.
  </section>

  <div class="canvas-layout">
    <main class="canvas-main" aria-label="Focus frame graph">
      <div class="canvas-main-summary" aria-label="Focus frame graph summary">
        Focus frame graph: {$focusCanvasStore.stack.frames.length} frames; active {$focusCanvasStore.activeFrame?.title || 'none'}; {$focusCanvasStore.events.length} timeline events.
      </div>
      <FocusCanvas 
        frames={$focusCanvasStore.stack.frames}
        activeFrameId={$focusCanvasStore.stack.active_id}
        onFrameSelect={handleFrameSelect}
      />
    </main>
    
    {#if showAscc}
      <aside class="panel-sidebar">
        <AsccPanel 
          sections={$focusCanvasStore.activeFrame?.ascc || null}
          compact={false}
        />
      </aside>
    {/if}
    
    {#if showTimeline}
      <aside class="timeline-sidebar">
        <Timeline 
          events={$focusCanvasStore.events}
          selectedEventId={selectedEventId}
          on:select={handleEventSelect}
          on:replay={handleEventReplay}
        />
        <ContextCognitionPeek />
        <ToolsRegistryPeek />
      </aside>
    {/if}
  </div>
</div>

<style>
  .canvas-page {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--page-bg, #0a0a0f);
  }
  
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    background: var(--header-bg, rgba(15, 15, 26, 0.95));
    border-bottom: 1px solid var(--header-border, #2d3a4a);
    backdrop-filter: blur(12px);
  }
  
  .page-header h1 {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary, #eaeaea);
  }
  
  .page-header h1 svg {
    color: var(--accent, #e94560);
  }
  
  .header-controls {
    display: flex;
    gap: 8px;
  }
  
  .toggle-btn {
    padding: 6px 12px;
    border: 1px solid var(--btn-border, #2d3a4a);
    border-radius: 6px;
    background: var(--btn-bg, transparent);
    color: var(--text-secondary, #9ca3af);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .toggle-btn:hover {
    border-color: var(--btn-hover-border, #3d4a5a);
    color: var(--text-primary, #eaeaea);
  }
  
  .toggle-btn.active {
    background: var(--accent, #e94560);
    border-color: var(--accent, #e94560);
    color: white;
  }
  
  .canvas-summary {
    padding: 8px 20px;
    border-bottom: 1px solid var(--header-border, #2d3a4a);
    color: var(--text-secondary, #9ca3af);
    background: var(--header-bg, rgba(15, 15, 26, 0.88));
    font-size: 12px;
    line-height: 1.4;
  }

  .canvas-summary strong {
    color: var(--text-primary, #eaeaea);
  }

  .canvas-layout {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 320px 280px;
    gap: 0;
    overflow: hidden;
  }
  
  .canvas-main {
    overflow: hidden;
    position: relative;
  }

  .canvas-main-summary {
    position: absolute;
    z-index: 2;
    top: 12px;
    left: 12px;
    max-width: min(520px, calc(100% - 24px));
    padding: 6px 8px;
    border: 1px solid var(--header-border, #2d3a4a);
    border-radius: 6px;
    color: var(--text-secondary, #9ca3af);
    background: rgba(10, 10, 15, 0.76);
    font-size: 11px;
    line-height: 1.35;
    pointer-events: none;
  }
  
  .panel-sidebar {
    border-left: 1px solid var(--sidebar-border, #2d3a4a);
    background: var(--sidebar-bg, rgba(10, 10, 15, 0.8));
    overflow-y: auto;
    padding: 16px;
  }
  
  .timeline-sidebar {
    border-left: 1px solid var(--sidebar-border, #2d3a4a);
    background: var(--sidebar-bg, rgba(10, 10, 15, 0.8));
    overflow: hidden;
  }
</style>