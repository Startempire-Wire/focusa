<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  export let events: Array<{
    id: string;
    timestamp: string;
    type: string;
    summary: string;
    frame_id?: string;
  }> = [];
  
  export let selectedEventId: string | null = null;
  
  const dispatch = createEventDispatcher<{
    select: { eventId: string };
    replay: { eventId: string };
  }>();
  
  const eventTypeColors: Record<string, string> = {
    'focus_frame_pushed': '#4ade80',
    'focus_frame_popped': '#ef4444',
    'focus_frame_completed': '#60a5fa',
    'turn_completed': '#a78bfa',
    'signal_ingested': '#fbbf24',
    'candidate_surfaced': '#e94560',
    'checkpoint_updated': '#34d399',
    'memory_reinforced': '#f472b6',
  };
  
  const eventTypeIcons: Record<string, string> = {
    'focus_frame_pushed': '▶',
    'focus_frame_popped': '◀',
    'focus_frame_completed': '✓',
    'turn_completed': '💬',
    'signal_ingested': '📡',
    'candidate_surfaced': '🔔',
    'checkpoint_updated': '💾',
    'memory_reinforced': '🧠',
  };
  
  function formatTime(iso: string): string {
    const date = new Date(iso);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }
  
  function formatRelativeTime(iso: string): string {
    const date = new Date(iso);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    
    if (seconds < 60) return `${seconds}s ago`;
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    return date.toLocaleDateString();
  }
  
  function handleEventClick(eventId: string) {
    dispatch('select', { eventId });
  }
  
  function handleReplayClick(eventId: string) {
    dispatch('replay', { eventId });
  }
  
  function getEventColor(type: string): string {
    return eventTypeColors[type] || '#6b7280';
  }
  
  function getEventIcon(type: string): string {
    return eventTypeIcons[type] || '•';
  }
</script>

<div class="timeline-panel">
  <header class="panel-header">
    <svg viewBox="0 0 24 24" width="18" height="18" class="header-icon">
      <path d="M13 3a9 9 0 00-9 9H1l3.89 3.89.07.14L9 12H6c0-3.87 3.13-7 7-7s7 3.13 7 7-3.13 7-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42A8.954 8.954 0 0013 21a9 9 0 000-18zm-1 5v5l4.28 2.54.72-1.21-3.5-2.08V8H12z" fill="currentColor"/>
    </svg>
    <span>Event Timeline</span>
    <span class="event-count">{events.length} events</span>
  </header>
  
  <div class="timeline-content">
    {#if events.length === 0}
      <div class="empty-state">
        <svg viewBox="0 0 24 24" width="48" height="48" opacity="0.3">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" fill="currentColor"/>
        </svg>
        <p>No events recorded yet</p>
      </div>
    {:else}
      <div class="timeline-list">
        {#each events as event, index}
          {@const isSelected = event.id === selectedEventId}
          {@const isLast = index === events.length - 1}
          
          <div 
            class="timeline-item"
            class:selected={isSelected}
            on:click={() => handleEventClick(event.id)}
            on:keypress={(e) => e.key === 'Enter' && handleEventClick(event.id)}
            role="button"
            tabindex="0"
          >
            <!-- Connector line -->
            {#if !isLast}
              <div class="connector"></div>
            {/if}
            
            <!-- Event marker -->
            <div 
              class="event-marker"
              style="background: {getEventColor(event.type)}; box-shadow: 0 0 8px {getEventColor(event.type)}"
            >
              <span class="event-icon">{getEventIcon(event.type)}</span>
            </div>
            
            <!-- Event content -->
            <div class="event-content">
              <div class="event-header">
                <span class="event-type" style="color: {getEventColor(event.type)}">
                  {event.type.replace(/_/g, ' ')}
                </span>
                <span class="event-time" title={event.timestamp}>
                  {formatRelativeTime(event.timestamp)}
                </span>
              </div>
              
              <p class="event-summary">{event.summary}</p>
              
              <div class="event-footer">
                <span class="event-id">{event.id.slice(0, 8)}...</span>
                
                {#if event.frame_id}
                  <span class="event-frame">Frame: {event.frame_id.slice(0, 8)}...</span>
                {/if}
                
                <button 
                  class="replay-btn"
                  on:click|stopPropagation={() => handleReplayClick(event.id)}
                  title="Replay to this point"
                >
                  <svg viewBox="0 0 24 24" width="14" height="14">
                    <path d="M12 5V1L7 6l5 5V7c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6H4c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8z" fill="currentColor"/>
                  </svg>
                  Replay
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .timeline-panel {
    background: var(--panel-bg, rgba(15, 15, 26, 0.95));
    border: 1px solid var(--panel-border, #2d3a4a);
    border-radius: 12px;
    overflow: hidden;
    backdrop-filter: blur(12px);
    display: flex;
    flex-direction: column;
    max-height: 400px;
  }
  
  .panel-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--header-border, #2d3a4a);
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #eaeaea);
    flex-shrink: 0;
  }
  
  .header-icon {
    color: var(--accent, #e94560);
  }
  
  .event-count {
    margin-left: auto;
    font-size: 11px;
    font-weight: 400;
    color: var(--text-tertiary, #6b7280);
    background: var(--badge-bg, rgba(45, 58, 74, 0.5));
    padding: 2px 8px;
    border-radius: 12px;
  }
  
  .timeline-content {
    overflow-y: auto;
    flex: 1;
  }
  
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px;
    color: var(--text-tertiary, #6b7280);
    text-align: center;
  }
  
  .empty-state p {
    margin-top: 12px;
    font-size: 13px;
  }
  
  .timeline-list {
    padding: 16px;
  }
  
  .timeline-item {
    position: relative;
    display: flex;
    gap: 12px;
    padding: 12px;
    margin-bottom: 4px;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .timeline-item:hover {
    background: var(--item-hover, rgba(45, 58, 74, 0.3));
  }
  
  .timeline-item.selected {
    background: var(--item-selected, rgba(15, 52, 96, 0.4));
    border: 1px solid var(--accent, #e94560);
  }
  
  .connector {
    position: absolute;
    left: 27px;
    top: 40px;
    width: 2px;
    height: calc(100% + 4px);
    background: var(--connector, #2d3a4a);
  }
  
  .event-marker {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    z-index: 1;
  }
  
  .event-icon {
    font-size: 12px;
  }
  
  .event-content {
    flex: 1;
    min-width: 0;
  }
  
  .event-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }
  
  .event-type {
    font-size: 11px;
    font-weight: 600;
    text-transform: capitalize;
  }
  
  .event-time {
    font-size: 10px;
    color: var(--text-tertiary, #6b7280);
    margin-left: auto;
  }
  
  .event-summary {
    font-size: 12px;
    color: var(--text-secondary, #9ca3af);
    margin: 0 0 8px 0;
    line-height: 1.4;
  }
  
  .event-footer {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 10px;
    color: var(--text-tertiary, #6b7280);
  }
  
  .event-id, .event-frame {
    font-family: monospace;
  }
  
  .replay-btn {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border: none;
    border-radius: 4px;
    background: var(--replay-bg, rgba(233, 69, 96, 0.2));
    color: var(--accent, #e94560);
    font-size: 10px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .replay-btn:hover {
    background: var(--accent, #e94560);
    color: white;
  }
</style>