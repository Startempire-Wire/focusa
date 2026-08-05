<script lang="ts">
  export type IconName = 'panel-left' | 'x' | 'chevron-left' | 'chevron-right' | 'chevron-down' | 'scope' | 'deck' | 'canvas' | 'terminal' | 'crist' | 'context' | 'target' | 'route' | 'sessions' | 'approvals' | 'evidence' | 'documents' | 'research' | 'runtime' | 'search' | 'settings' | 'warning' | 'blocked' | 'check' | 'sparkles';
  let { name, size = 16, strokeWidth = 1.7, label }: { name: IconName; size?: 14 | 16 | 18 | 20; strokeWidth?: number; label?: string } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width={strokeWidth} stroke-linecap="round" stroke-linejoin="round" role={label ? 'img' : undefined} aria-label={label} aria-hidden={label ? undefined : 'true'}>
  {#if name === 'panel-left'}<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16"/>
  {:else if name === 'x'}<path d="m6 6 12 12M18 6 6 18"/>
  {:else if name === 'chevron-left'}<path d="m15 18-6-6 6-6"/>
  {:else if name === 'chevron-right'}<path d="m9 18 6-6-6-6"/>
  {:else if name === 'chevron-down'}<path d="m6 9 6 6 6-6"/>
  {:else if name === 'scope'}<path d="M4 7h6l2 2h8v10H4z"/><path d="M4 7V5h6l2 2"/>
  {:else if name === 'deck'}<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="18" height="7" rx="1"/>
  {:else if name === 'canvas'}<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M8 4v16M8 10h13"/>
  {:else if name === 'terminal'}<rect x="3" y="4" width="18" height="16" rx="2"/><path d="m7 9 3 3-3 3M13 15h4"/>
  {:else if name === 'crist'}<path d="M9.5 4.5 12 2l2.5 2.5L17 4l1 3 3 1-1 3 2 2-2 2 1 3-3 1-1 3-3-1-2.5 2.5L9 21l-3 1-1-3-3-1 1-3-2-2 2-2-1-3 3-1 1-3z"/><circle cx="12" cy="12" r="3"/>
  {:else if name === 'context'}<circle cx="7" cy="7" r="3"/><circle cx="17" cy="17" r="3"/><path d="m9 9 6 6M14 5h5v5M10 19H5v-5"/>
  {:else if name === 'target'}<circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3"/>
  {:else if name === 'route'}<circle cx="6" cy="18" r="2"/><circle cx="18" cy="6" r="2"/><path d="M8 18h3a3 3 0 0 0 3-3V9a3 3 0 0 1 3-3"/>
  {:else if name === 'sessions'}<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>
  {:else if name === 'approvals'}<path d="M12 3 4 6v5c0 5 3.4 8.7 8 10 4.6-1.3 8-5 8-10V6z"/><path d="m9 12 2 2 4-4"/>
  {:else if name === 'evidence'}<path d="M6 3h9l3 3v15H6z"/><path d="M14 3v4h4M9 13l2 2 4-4"/>
  {:else if name === 'documents'}<path d="M6 3h9l3 3v15H6z"/><path d="M14 3v4h4M9 11h6M9 15h6"/>
  {:else if name === 'research'}<path d="M9 3h6M10 3v6l-5 9a2 2 0 0 0 2 3h10a2 2 0 0 0 2-3l-5-9V3"/><path d="M8 15h8"/>
  {:else if name === 'runtime'}<rect x="4" y="4" width="16" height="6" rx="2"/><rect x="4" y="14" width="16" height="6" rx="2"/><path d="M8 7h.01M8 17h.01"/>
  {:else if name === 'search'}<circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/>
  {:else if name === 'settings'}<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .34 1.88l.06.06-2.83 2.83-.06-.06A1.7 1.7 0 0 0 15 19.4a1.7 1.7 0 0 0-1 .6 1.7 1.7 0 0 0-.4 1v.1h-4v-.1a1.7 1.7 0 0 0-1.1-1.6 1.7 1.7 0 0 0-1.88.34l-.06.06-2.83-2.83.06-.06A1.7 1.7 0 0 0 4.6 15a1.7 1.7 0 0 0-.6-1 1.7 1.7 0 0 0-1-.4h-.1v-4H3A1.7 1.7 0 0 0 4.6 8.5a1.7 1.7 0 0 0-.34-1.88l-.06-.06 2.83-2.83.06.06A1.7 1.7 0 0 0 9 4.6a1.7 1.7 0 0 0 1-.6 1.7 1.7 0 0 0 .4-1v-.1h4V3A1.7 1.7 0 0 0 15.5 4.6a1.7 1.7 0 0 0 1.88-.34l.06-.06 2.83 2.83-.06.06A1.7 1.7 0 0 0 19.4 9c.4.28.73.62 1 .98.25.3.4.7.4 1.02v.1h.1v4h-.1a1.7 1.7 0 0 0-1.4 1.9z"/>
  {:else if name === 'warning'}<path d="m12 3 10 18H2z"/><path d="M12 9v4M12 17h.01"/>
  {:else if name === 'blocked'}<path d="M8 3h8l5 5v8l-5 5H8l-5-5V8z"/><path d="m9 9 6 6M15 9l-6 6"/>
  {:else if name === 'check'}<path d="m5 12 4 4L19 6"/>
  {:else}<path d="m12 3 1.4 5.6L19 10l-5.6 1.4L12 17l-1.4-5.6L5 10l5.6-1.4z"/>
  {/if}
</svg>
