<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { FitAddon } from '@xterm/addon-fit';
  import { Terminal } from '@xterm/xterm';
  import '@xterm/xterm/css/xterm.css';
  import type { AttachmentId, WorkSurfaceId } from '$lib/mission-canvas/types';
  import { hasExactPiAttachment, type PiAttachmentProjection, type PiNativeCommand, type PiTerminalGeometry } from './pi-attachment-contract';

  export interface PiTerminalOutput {
    attachment_id: AttachmentId;
    work_surface_id: WorkSurfaceId;
    generation: number;
    sequence: number;
    data: string;
  }

  export interface PiTerminalBridge {
    send(command: PiNativeCommand): Promise<void>;
    subscribeOutput(attachmentId: AttachmentId, listener: (output: PiTerminalOutput) => void): Promise<() => void> | (() => void);
  }

  let {
    attachment,
    bridge
  }: {
    attachment: PiAttachmentProjection;
    bridge: PiTerminalBridge;
  } = $props();

  let host: HTMLDivElement;
  let terminal = $state.raw<Terminal>();
  let fitAddon: FitAddon | undefined;
  let geometry = $state.raw<PiTerminalGeometry>();
  let terminalReady = $state(false);

  function measure(): PiTerminalGeometry | undefined {
    if (!terminal || !host) return undefined;
    return {
      columns: terminal.cols,
      rows: terminal.rows,
      pixelWidth: Math.round(host.clientWidth),
      pixelHeight: Math.round(host.clientHeight)
    };
  }

  async function send(command: PiNativeCommand): Promise<void> {
    try {
      await bridge.send(command);
    } catch {
      // The attachment projection remains the authority for surfaced runtime failure.
    }
  }

  onMount(() => {
    const styles = getComputedStyle(document.documentElement);
    fitAddon = new FitAddon();
    terminal = new Terminal({
      allowProposedApi: false,
      convertEol: false,
      cursorBlink: true,
      cursorStyle: 'block',
      fontFamily: "'Commit Mono', 'SFMono-Regular', Consolas, monospace",
      fontSize: 13,
      fontWeight: '400',
      lineHeight: 1.4,
      scrollback: 10_000,
      theme: {
        background: styles.getPropertyValue('--color-bg').trim(),
        foreground: styles.getPropertyValue('--color-text-soft').trim(),
        cursor: styles.getPropertyValue('--color-accent-bright').trim(),
        selectionBackground: styles.getPropertyValue('--color-elevated').trim()
      }
    });
    terminal.loadAddon(fitAddon);
    terminal.open(host);
    fitAddon.fit();
    geometry = measure();
    terminalReady = true;

    const input = terminal.onData((data) => {
      if (attachment.canWrite && hasExactPiAttachment(attachment)) {
        void send({ kind: 'input', attachment_id: attachment.identity.attachment_id, data });
      }
    });
    const resize = new ResizeObserver(() => {
      fitAddon?.fit();
      const next = measure();
      if (!next || !hasExactPiAttachment(attachment)) return;
      if (geometry && next.columns === geometry.columns && next.rows === geometry.rows && next.pixelWidth === geometry.pixelWidth && next.pixelHeight === geometry.pixelHeight) return;
      geometry = next;
      void send({ kind: 'resize', attachment_id: attachment.identity.attachment_id, geometry: next });
    });
    resize.observe(host);

    return () => {
      input.dispose();
      resize.disconnect();
      terminalReady = false;
      terminal?.dispose();
      terminal = undefined;
      fitAddon = undefined;
    };
  });

  $effect(() => {
    if (!terminalReady || !terminal || !hasExactPiAttachment(attachment)) return;
    const initialGeometry = untrack(() => geometry);
    if (!initialGeometry) return;
    const identity = attachment.identity;
    let unsubscribe: (() => void) | undefined;
    let cancelled = false;
    let generation: number | undefined;
    let lastSequence = -1;

    void Promise.resolve(bridge.subscribeOutput(identity.attachment_id, (output) => {
      if (output.attachment_id !== identity.attachment_id || output.work_surface_id !== identity.work_surface_id) return;
      if (generation === undefined) generation = output.generation;
      if (output.generation !== generation || output.sequence <= lastSequence) return;
      lastSequence = output.sequence;
      terminal?.write(output.data);
    })).then((stop) => {
      if (cancelled) stop();
      else unsubscribe = stop;
    });
    void send({ kind: 'attach', identity, geometry: initialGeometry });

    return () => {
      cancelled = true;
      unsubscribe?.();
      void send({ kind: 'detach', attachment_id: identity.attachment_id });
    };
  });
</script>

<div bind:this={host} class="pty-terminal" aria-label="Live Pi terminal" data-attachment-id={attachment.identity?.attachment_id}></div>

<style>
  .pty-terminal{width:100%;height:100%;min-width:0;min-height:0;overflow:hidden;border-radius:10px;background:var(--color-bg);text-align:left}
  .pty-terminal :global(.xterm){height:100%;padding:var(--space-3)}
  .pty-terminal :global(.xterm-viewport){scrollbar-width:thin;scrollbar-color:var(--color-border-strong) transparent}
</style>
