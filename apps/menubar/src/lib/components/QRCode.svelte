<!--
  QRCode.svelte — renders a QR code as inline SVG (focusa-ui0y.9).

  - Pure inline SVG, no external assets
  - Uses the `qrcode` npm package (~20KB, no native deps)
  - Mobile-friendly: at least 200×200px, 4-module quiet zone
  - Auto-refreshes when the `payload` changes

  Props:
    payload:   string to encode (e.g., `pair_url` from /v1/device/pair/start)
    size:      pixel size of the rendered SVG (default 240)
    level:     error-correction level: L | M | Q | H (default M)
    dark:      dark color (default currentColor for theme)
    light:     light color (default transparent)
-->
<script lang="ts">
  import QRCodeLib from 'qrcode';
  import { onMount } from 'svelte';

  let { payload, size = 240, level = 'M', dark = 'currentColor', light = 'transparent' }: {
    payload: string;
    size?: number;
    level?: 'L' | 'M' | 'Q' | 'H';
    dark?: string;
    light?: string;
  } = $props();

  let svg = $state<string>('');
  let error = $state<string | null>(null);

  async function render() {
    if (!payload) {
      svg = '';
      return;
    }
    try {
      svg = await QRCodeLib.toString(payload, {
        type: 'svg',
        margin: 4,                    // 4-module quiet zone
        width: size,
        errorCorrectionLevel: level,
        color: { dark, light },
      });
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      svg = '';
    }
  }

  $effect(() => {
    // re-render when payload/size/level change
    void payload; void size; void level; void dark; void light;
    void render();
  });

  onMount(render);
</script>

{#if error}
  <div class="qr-error" role="alert" aria-live="polite">QR error: {error}</div>
{:else if svg}
  <div class="qr" style:width="{size}px" style:height="{size}px" aria-label="QR code">
    {@html svg}
  </div>
{/if}

<style>
  .qr {
    display: inline-block;
    line-height: 0;
  }
  .qr-error {
    color: #e84c4c;
    font-size: 12px;
    padding: 8px;
  }
  /* Make currentColor work for the SVG paths */
  :global(.qr svg) {
    width: 100%;
    height: 100%;
  }
</style>
