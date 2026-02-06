<script lang="ts">
  import { onMount } from 'svelte';
  import { zoom, zoomIdentity, type ZoomBehavior } from 'd3-zoom';
  import { select, type Selection } from 'd3-selection';
  import type { CanvasFocusFrame as FocusFrame } from '$lib/types/focus-canvas';
  
  // Props
  export let frames: FocusFrame[] = [];
  export let activeFrameId: string | null = null;
  export let onFrameSelect: (frameId: string) => void = () => {};
  
  // Canvas state
  let svg: SVGSVGElement | null = null;
  let g: SVGGElement | null = null;
  let zoomBehavior: ZoomBehavior<SVGSVGElement, unknown> | null = null;
  let transform = zoomIdentity;
  
  // Layout constants
  const FRAME_WIDTH = 280;
  const FRAME_HEIGHT = 180;
  const FRAME_GAP_Y = 100;
  const STACK_OFFSET_X = 30;
  
  // Reactive computed values
  $: layout = computeLayout(frames, activeFrameId);
  $: connections = computeConnections(layout);
  
  interface FrameNode {
    id: string;
    x: number;
    y: number;
    width: number;
    height: number;
    frame: FocusFrame;
    isActive: boolean;
    depth: number;
  }
  
  function computeLayout(frames: FocusFrame[], activeId: string | null): FrameNode[] {
    const nodes: FrameNode[] = [];

    frames.forEach((frame, index) => {
      const depth = index;
      const isActive = frame.id === activeId;
      
      // Stack layout: newer frames above and slightly offset
      const x = 100 + depth * STACK_OFFSET_X;
      const y = 100 + depth * (FRAME_HEIGHT + FRAME_GAP_Y);
      
      nodes.push({
        id: frame.id,
        x,
        y,
        width: FRAME_WIDTH,
        height: FRAME_HEIGHT,
        frame,
        isActive,
        depth
      });
    });
    
    return nodes;
  }
  
  function computeConnections(nodes: FrameNode[]): Array<{from: FrameNode; to: FrameNode}> {
    const conns: Array<{from: FrameNode; to: FrameNode}> = [];
    for (let i = 0; i < nodes.length - 1; i++) {
      conns.push({ from: nodes[i], to: nodes[i + 1] });
    }
    return conns;
  }
  
  onMount(() => {
    if (!svg || !g) return;

    const svgSel: Selection<SVGSVGElement, unknown, null, undefined> = select(svg);
    const gSel: Selection<SVGGElement, unknown, null, undefined> = select(g);

    zoomBehavior = zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        transform = event.transform;
        gSel.attr('transform', transform.toString());
      });

    svgSel.call(zoomBehavior);

    // Initial center
    const bbox = svg.getBoundingClientRect();
    const initialTransform = zoomIdentity
      .translate(bbox.width / 2 - 200, 50)
      .scale(0.9);
    svgSel.call(zoomBehavior.transform, initialTransform);
  });
  
  function handleFrameClick(frameId: string) {
    onFrameSelect(frameId);
  }
  
  const timeFmt = new Intl.DateTimeFormat(undefined, {
    hour: '2-digit',
    minute: '2-digit'
  });

  function formatTimestamp(iso: string): string {
    const ms = Date.parse(iso);
    if (!Number.isFinite(ms)) return '';
    return timeFmt.format(ms);
  }
</script>

<div class="canvas-container">
  <svg
    bind:this={svg}
    class="focus-canvas"
    viewBox="0 0 1000 800"
    preserveAspectRatio="xMidYMid meet"
  >
    <defs>
      <!-- Gradients -->
      <linearGradient id="frameGradient" x1="0%" y1="0%" x2="0%" y2="100%">
        <stop offset="0%" stop-color="var(--frame-bg-top, #1a1a2e)" />
        <stop offset="100%" stop-color="var(--frame-bg-bottom, #16213e)" />
      </linearGradient>
      
      <linearGradient id="activeFrameGradient" x1="0%" y1="0%" x2="0%" y2="100%">
        <stop offset="0%" stop-color="var(--active-top, #0f3460)" />
        <stop offset="100%" stop-color="var(--active-bottom, #1a4d6e)" />
      </linearGradient>
      
      <linearGradient id="connectionGradient" x1="0%" y1="0%" x2="0%" y2="100%">
        <stop offset="0%" stop-color="var(--connection-start, #e94560)" stop-opacity="0.8" />
        <stop offset="100%" stop-color="var(--connection-end, #0f3460)" stop-opacity="0.4" />
      </linearGradient>
      
      <!-- Glow filter -->
      <filter id="glow" x="-50%" y="-50%" width="200%" height="200%">
        <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
        <feMerge>
          <feMergeNode in="coloredBlur"/>
          <feMergeNode in="SourceGraphic"/>
        </feMerge>
      </filter>
      
      <!-- Drop shadow -->
      <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
        <feDropShadow dx="0" dy="4" stdDeviation="6" flood-color="#000" flood-opacity="0.3"/>
      </filter>
    </defs>
    
    <g bind:this={g}>
      <!-- Background grid -->
      <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
        <circle cx="20" cy="20" r="1" fill="var(--grid-dot, #333)" opacity="0.3"/>
      </pattern>
      <rect x="-5000" y="-5000" width="10000" height="10000" fill="url(#grid)" />
      
      <!-- Connection lines between frames -->
      <g class="connections">
        {#each connections as conn}
          <path
            d={`M ${conn.from.x + FRAME_WIDTH / 2} ${conn.from.y + FRAME_HEIGHT} 
                C ${conn.from.x + FRAME_WIDTH / 2} ${conn.from.y + FRAME_HEIGHT + 50},
                  ${conn.to.x + FRAME_WIDTH / 2} ${conn.to.y - 50},
                  ${conn.to.x + FRAME_WIDTH / 2} ${conn.to.y}`}
            fill="none"
            stroke="url(#connectionGradient)"
            stroke-width="3"
            stroke-linecap="round"
          />
          <!-- Arrow head -->
          <polygon
            points={`${conn.to.x + FRAME_WIDTH / 2},${conn.to.y} 
                     ${conn.to.x + FRAME_WIDTH / 2 - 8},${conn.to.y - 12}
                     ${conn.to.x + FRAME_WIDTH / 2 + 8},${conn.to.y - 12}`}
            fill="var(--connection-end, #0f3460)"
          />
        {/each}
      </g>
      
      <!-- Frame nodes -->
      <g class="frames">
        {#each layout as node}
          <g
            class="frame-node"
            class:active={node.isActive}
            transform={`translate(${node.x}, ${node.y})`}
            on:click={() => handleFrameClick(node.id)}
            on:keydown={(e) => e.key === 'Enter' && handleFrameClick(node.id)}
            tabindex="0"
            role="button"
            aria-label={`Focus frame: ${node.frame.title}`}
          >
            <!-- Frame shadow/background -->
            <rect
              class="frame-bg"
              width={FRAME_WIDTH}
              height={FRAME_HEIGHT}
              rx="12"
              ry="12"
              fill={node.isActive ? 'url(#activeFrameGradient)' : 'url(#frameGradient)'}
              stroke={node.isActive ? 'var(--active-border, #e94560)' : 'var(--frame-border, #2d3a4a)'}
              stroke-width={node.isActive ? 3 : 2}
              filter={node.isActive ? 'url(#glow)' : 'url(#shadow)'}
            />
            
            <!-- Frame content -->
            <g class="frame-content" transform="translate(16, 16)">
              <!-- Title -->
              <text
                class="frame-title"
                x="0"
                y="20"
                fill="var(--text-primary, #eaeaea)"
                font-size="14"
                font-weight="600"
                font-family="system-ui, -apple-system, sans-serif"
              >
                {node.frame.title.slice(0, 30)}{node.frame.title.length > 30 ? '...' : ''}
              </text>
              
              <!-- Status indicator -->
              <circle
                cx={FRAME_WIDTH - 40}
                cy="12"
                r="6"
                fill={node.isActive ? 'var(--status-active, #4ade80)' : 'var(--status-inactive, #6b7280)'}
              />
              
              <!-- Metadata -->
              <text
                class="frame-meta"
                x="0"
                y="44"
                fill="var(--text-secondary, #9ca3af)"
                font-size="11"
                font-family="system-ui, -apple-system, sans-serif"
              >
                {node.frame.status} • Depth {node.depth}
              </text>
              
              <!-- Intent preview -->
              <text
                class="frame-intent"
                x="0"
                y="70"
                fill="var(--text-tertiary, #6b7280)"
                font-size="10"
                font-family="system-ui, -apple-system, sans-serif"
              >
                {node.frame.intent?.slice(0, 40) || 'No intent set'}
                {node.frame.intent?.length > 40 ? '...' : ''}
              </text>
              
              <!-- Timestamp -->
              <text
                class="frame-time"
                x="0"
                y={FRAME_HEIGHT - 32}
                fill="var(--text-tertiary, #6b7280)"
                font-size="9"
                font-family="monospace"
              >
                {formatTimestamp(node.frame.started_at)}
              </text>
              
              <!-- ASCC preview if available -->
              {#if node.frame.ascc}
                <rect
                  x="0"
                  y={FRAME_HEIGHT - 60}
                  width={FRAME_WIDTH - 32}
                  height="20"
                  rx="4"
                  fill="var(--ascc-bg, rgba(15, 52, 96, 0.3))"
                />
                <text
                  x="8"
                  y={FRAME_HEIGHT - 46}
                  fill="var(--ascc-text, #60a5fa)"
                  font-size="9"
                  font-family="system-ui, -apple-system, sans-serif"
                >
                  ASCC: {node.frame.ascc.current_focus.slice(0, 25)}...
                </text>
              {/if}
            </g>
          </g>
        {/each}
      </g>
      
      <!-- Active frame highlight ring -->
      {#each layout.filter(n => n.isActive) as active}
        <rect
          class="active-highlight"
          x={active.x - 8}
          y={active.y - 8}
          width={FRAME_WIDTH + 16}
          height={FRAME_HEIGHT + 16}
          rx="16"
          ry="16"
          fill="none"
          stroke="var(--highlight-ring, #e94560)"
          stroke-width="2"
          stroke-dasharray="8 4"
          opacity="0.6"
        >
          <animate
            attributeName="stroke-dashoffset"
            from="0"
            to="12"
            dur="2s"
            repeatCount="indefinite"
          />
        </rect>
      {/each}
    </g>
  </svg>
  
  <!-- Controls overlay -->
  <div class="canvas-controls">
    <button
      class="control-btn"
      on:click={() => zoomBehavior && svg && select(svg).call(zoomBehavior.transform, zoomIdentity)}
      title="Reset view"
    >
      <svg viewBox="0 0 24 24" width="20" height="20">
        <path d="M12 5V1L7 6l5 5V7c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6H4c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8z" fill="currentColor"/>
      </svg>
    </button>
    
    <div class="zoom-level">
      {Math.round(transform.k * 100)}%
    </div>
  </div>
</div>

<style>
  .canvas-container {
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
    background: var(--canvas-bg, #0f0f1a);
  }
  
  .focus-canvas {
    width: 100%;
    height: 100%;
    cursor: grab;
  }
  
  .focus-canvas:active {
    cursor: grabbing;
  }
  
  .frame-node {
    cursor: pointer;
    transition: filter 0.2s ease;
  }
  
  .frame-node:hover .frame-bg {
    filter: brightness(1.1);
  }
  
  .frame-node:focus {
    outline: none;
  }
  
  .frame-node:focus .frame-bg {
    stroke: var(--focus-ring, #60a5fa);
    stroke-width: 3;
  }
  
  .canvas-controls {
    position: absolute;
    bottom: 20px;
    right: 20px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: center;
  }
  
  .control-btn {
    width: 40px;
    height: 40px;
    border-radius: 8px;
    border: 1px solid var(--control-border, #2d3a4a);
    background: var(--control-bg, rgba(26, 26, 46, 0.9));
    color: var(--control-text, #eaeaea);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    backdrop-filter: blur(8px);
  }
  
  .control-btn:hover {
    background: var(--control-hover, rgba(45, 58, 74, 0.9));
    border-color: var(--control-hover-border, #3d4a5a);
  }
  
  .zoom-level {
    font-size: 12px;
    color: var(--text-secondary, #9ca3af);
    font-family: monospace;
    background: var(--control-bg, rgba(26, 26, 46, 0.9));
    padding: 4px 8px;
    border-radius: 4px;
    backdrop-filter: blur(8px);
  }
</style>
