<script lang="ts">
  import type { CanvasAsccSections as AsccSections } from '$lib/types/focus-canvas';
  
  export let sections: AsccSections | null = null;
  export let compact = false;
  
  const slots = [
    { key: 'intent', label: 'Intent', icon: '🎯', color: '#e94560' },
    { key: 'current_focus', label: 'Current Focus', icon: '🔍', color: '#0f3460' },
    { key: 'decisions', label: 'Decisions', icon: '✓', color: '#4ade80' },
    { key: 'artifacts', label: 'Artifacts', icon: '📎', color: '#60a5fa' },
    { key: 'constraints', label: 'Constraints', icon: '⚡', color: '#fbbf24' },
    { key: 'open_questions', label: 'Open Questions', icon: '?', color: '#a78bfa' },
    { key: 'next_steps', label: 'Next Steps', icon: '→', color: '#34d399' },
    { key: 'recent_results', label: 'Recent Results', icon: '📊', color: '#f472b6' },
    { key: 'failures', label: 'Failures', icon: '⚠', color: '#ef4444' },
    { key: 'notes', label: 'Notes', icon: '📝', color: '#9ca3af' },
  ];
  
  type SlotKey = typeof slots[number]['key'];

  function getSlotContent(key: SlotKey, sections: AsccSections | null): string[] {
    if (!sections) return [];

    switch (key) {
      case 'intent':
        return sections.intent ? [sections.intent] : [];
      case 'current_focus':
        return sections.current_focus ? [sections.current_focus] : [];
      case 'decisions':
        return sections.decisions;
      case 'artifacts':
        return sections.artifacts.map((a) => a.label);
      case 'constraints':
        return sections.constraints;
      case 'open_questions':
        return sections.open_questions;
      case 'next_steps':
        return sections.next_steps;
      case 'recent_results':
        return sections.recent_results;
      case 'failures':
        return sections.failures;
      case 'notes':
        return sections.notes;
    }

    return [];
  }
  
  function truncate(text: string, maxLen: number): string {
    if (text.length <= maxLen) return text;
    return text.slice(0, maxLen) + '...';
  }
</script>

<div class="ascc-panel" class:compact>
  <header class="panel-header">
    <svg viewBox="0 0 24 24" width="18" height="18" class="header-icon">
      <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z" fill="currentColor"/>
    </svg>
    <span>Anchored Structured Context</span>
    {#if sections}
      <span class="slot-count">
        {slots.filter((s) => getSlotContent(s.key, sections).length > 0).length}/10 slots
      </span>
    {/if}
  </header>
  
  <div class="slots-grid">
    {#each slots as slot}
      {@const content = getSlotContent(slot.key, sections)}
      <div 
        class="slot-card" 
        class:empty={content.length === 0}
        class:populated={content.length > 0}
        style="--slot-color: {slot.color}"
      >
        <div class="slot-header">
          <span class="slot-icon" style="color: {slot.color}">{slot.icon}</span>
          <span class="slot-label">{slot.label}</span>
          <span class="slot-badge">{content.length}</span>
        </div>
        
        {#if !compact && content.length > 0}
          <ul class="slot-content">
            {#each content.slice(0, 3) as item}
              <li>{truncate(item, 60)}</li>
            {/each}
            {#if content.length > 3}
              <li class="more">+{content.length - 3} more...</li>
            {/if}
          </ul>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .ascc-panel {
    background: var(--panel-bg, rgba(15, 15, 26, 0.95));
    border: 1px solid var(--panel-border, #2d3a4a);
    border-radius: 12px;
    padding: 16px;
    backdrop-filter: blur(12px);
  }
  
  .ascc-panel.compact {
    padding: 12px;
  }
  
  .panel-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--header-border, #2d3a4a);
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #eaeaea);
  }
  
  .header-icon {
    color: var(--accent, #e94560);
  }
  
  .slot-count {
    margin-left: auto;
    font-size: 11px;
    font-weight: 400;
    color: var(--text-tertiary, #6b7280);
    background: var(--badge-bg, rgba(45, 58, 74, 0.5));
    padding: 2px 8px;
    border-radius: 12px;
  }
  
  .slots-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 12px;
  }
  
  .compact .slots-grid {
    grid-template-columns: repeat(5, 1fr);
    gap: 8px;
  }
  
  .slot-card {
    background: var(--card-bg, rgba(26, 26, 46, 0.6));
    border: 1px solid var(--card-border, #2d3a4a);
    border-radius: 8px;
    padding: 12px;
    transition: all 0.2s ease;
  }
  
  .slot-card:hover {
    border-color: var(--slot-color);
    background: var(--card-hover, rgba(26, 26, 46, 0.8));
  }
  
  .slot-card.empty {
    opacity: 0.4;
  }
  
  .slot-card.populated {
    border-left: 3px solid var(--slot-color);
  }
  
  .slot-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }
  
  .slot-icon {
    font-size: 16px;
  }
  
  .slot-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary, #9ca3af);
    flex: 1;
  }
  
  .slot-badge {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-tertiary, #6b7280);
    background: var(--badge-bg, rgba(45, 58, 74, 0.5));
    padding: 2px 6px;
    border-radius: 10px;
    min-width: 18px;
    text-align: center;
  }
  
  .slot-card.populated .slot-badge {
    background: var(--slot-color);
    color: white;
  }
  
  .slot-content {
    list-style: none;
    padding: 0;
    margin: 0;
    font-size: 11px;
    color: var(--text-tertiary, #6b7280);
  }
  
  .slot-content li {
    padding: 4px 0;
    border-bottom: 1px solid var(--item-border, rgba(45, 58, 74, 0.3));
  }
  
  .slot-content li:last-child {
    border-bottom: none;
  }
  
  .slot-content .more {
    color: var(--accent, #e94560);
    font-style: italic;
  }
</style>
