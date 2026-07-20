<script lang="ts">
  import { createEventDispatcher } from "svelte";

  let {
    componentName,
    kind = "card",
    label = "Focusa",
    description = "",
    status = "ready",
    progress = 0,
    primaryActionLabel = "Continue",
    actionAvailable = false,
    disabled = false,
    busy = false,
    details = "",
    invokeAction,
  }: {
    componentName: string;
    kind?: string;
    label?: string;
    description?: string;
    status?: string;
    progress?: number;
    primaryActionLabel?: string;
    actionAvailable?: boolean;
    disabled?: boolean;
    busy?: boolean;
    details?: string;
    invokeAction?: () => void;
  } = $props();

  const dispatch = createEventDispatcher<{ "focusa-action": { componentName: string } }>();
  const activate = () => {
    invokeAction?.();
    dispatch("focusa-action", { componentName });
  };
  const boundedProgress = $derived(Math.max(0, Math.min(100, Number(progress) || 0)));
</script>

<article
  class:warning={kind === "warning"}
  class:recovery={kind === "recovery"}
  class:shell={kind === "shell"}
  data-focusa-component={componentName}
  data-projection="desktop tablet mobile terminal"
  role={kind === "recovery" ? "alert" : undefined}
  aria-busy={busy}
>
  <header>
    <strong>{label}</strong>
    <span class="status" aria-live="polite">{busy ? "Saving…" : status}</span>
  </header>

  {#if description}<p>{description}</p>{/if}

  {#if kind === "progress"}
    <div
      class="progress"
      role="progressbar"
      aria-label={label}
      aria-valuemin="0"
      aria-valuemax="100"
      aria-valuenow={boundedProgress}
    ><span style:width={`${boundedProgress}%`}></span></div>
  {/if}

  {#if kind === "input"}
    <label>
      <span>Your response</span>
      <textarea aria-label={label} disabled={disabled} rows="3"></textarea>
    </label>
  {/if}

  {#if actionAvailable}
    <button class="primary" type="button" onclick={activate} disabled={disabled || busy}>
      {primaryActionLabel}
    </button>
  {/if}

  {#if details}
    <details><summary>Advanced details</summary><pre>{details}</pre></details>
  {/if}

  <span class="terminal" data-terminal-fallback>{label}: {status}</span>
</article>

<style>
  :host { display: block; color: var(--focusa-fg, #172033); font: 500 0.95rem/1.45 system-ui, sans-serif; }
  article { container-type: inline-size; display: grid; gap: .75rem; padding: 1rem; border: 1px solid var(--focusa-border, #c8d0dd); border-radius: .75rem; background: var(--focusa-surface, #fff); }
  article.warning { border-inline-start: .35rem solid #9b6500; }
  article.recovery { border-inline-start: .35rem solid #a32929; }
  article.shell { min-height: 8rem; }
  header { display: flex; align-items: baseline; justify-content: space-between; gap: 1rem; }
  .status { color: var(--focusa-muted, #536075); font-size: .8rem; }
  p { margin: 0; }
  label { display: grid; gap: .35rem; }
  textarea { min-height: 4.5rem; padding: .6rem; border: 1px solid #7a879b; border-radius: .4rem; font: inherit; }
  .primary { justify-self: start; min-height: 2.75rem; padding: .65rem 1rem; border: 0; border-radius: .45rem; color: #fff; background: #174ea6; font: inherit; font-weight: 700; cursor: pointer; }
  .primary:focus-visible, textarea:focus-visible, summary:focus-visible { outline: 3px solid #f4b400; outline-offset: 3px; }
  .primary:disabled { cursor: not-allowed; opacity: .6; }
  .progress { height: .7rem; overflow: hidden; border-radius: 999px; background: #d7deea; }
  .progress span { display: block; height: 100%; background: #176b45; transition: width 160ms ease; }
  pre { overflow: auto; white-space: pre-wrap; }
  .terminal { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
  @container (max-width: 30rem) { header { align-items: start; flex-direction: column; gap: .25rem; } .primary { width: 100%; } }
  @media (prefers-reduced-motion: reduce) { *, .progress span { scroll-behavior: auto !important; transition: none !important; animation: none !important; } }
  @media (prefers-contrast: more) { article { border-width: 2px; } }
</style>
