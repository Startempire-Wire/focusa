<script lang="ts">
  import { onMount } from 'svelte';
  type OrbState = 'idle' | 'loading' | 'working' | 'connecting' | 'complete' | 'stale' | 'blocked' | 'error';
  let { state: orbState = 'idle', size = 28, label = 'System state' }: { state?: OrbState; size?: number; label?: string } = $props();
  let canvas = $state<HTMLCanvasElement>();

  const colors: Record<OrbState, [string, string]> = {
    idle: ['#4f9cff', '#a98cff'], loading: ['#46dafb', '#4f9cff'], working: ['#4f9cff', '#a98cff'], connecting: ['#46dafb', '#73b7ff'], complete: ['#4bde91', '#73b7ff'], stale: ['#ffbd59', '#a98cff'], blocked: ['#ff6e7f', '#ffbd59'], error: ['#ff6e7f', '#a98cff']
  };

  onMount(() => {
    if (!canvas) return;
    const context = canvas.getContext('2d');
    if (!context) return;
    const dpr = Math.min(3, window.devicePixelRatio || 1);
    canvas.width = size * dpr;
    canvas.height = size * dpr;
    context.scale(dpr, dpr);
    const reduce = document.documentElement.dataset.motion === 'reduced' || (matchMedia('(prefers-reduced-motion: reduce)').matches && document.documentElement.dataset.motion !== 'full');
    let visible = true;
    let frame = 0;
    const observer = new IntersectionObserver(([entry]) => { visible = Boolean(entry?.isIntersecting); if (visible && !frame) frame = requestAnimationFrame(draw); });
    observer.observe(canvas);
    const draw = (now = 0) => {
      frame = 0;
      const [first, second] = colors[orbState];
      const active = !['idle', 'stale', 'blocked', 'error'].includes(orbState);
      const pulse = reduce ? .82 : .78 + Math.sin(now / (active ? 520 : 1250)) * (active ? .12 : .06);
      context.clearRect(0, 0, size, size);
      const gradient = context.createRadialGradient(size * .36, size * .3, size * .05, size / 2, size / 2, size * .48);
      gradient.addColorStop(0, first);
      gradient.addColorStop(.62, second);
      gradient.addColorStop(1, 'rgba(9,10,14,0)');
      context.globalAlpha = orbState === 'blocked' || orbState === 'error' ? .7 : pulse;
      context.fillStyle = gradient;
      context.beginPath();
      context.arc(size / 2, size / 2, size * .48, 0, Math.PI * 2);
      context.fill();
      context.globalAlpha = .72;
      context.strokeStyle = first;
      context.lineWidth = 1;
      context.beginPath();
      context.arc(size / 2, size / 2, size * (.29 + (reduce ? 0 : Math.sin(now / 840) * .025)), 0, Math.PI * 2);
      context.stroke();
      context.globalAlpha = 1;
      if (!reduce && visible && document.visibilityState === 'visible') frame = requestAnimationFrame(draw);
    };
    draw();
    const onVisibility = () => { if (document.visibilityState === 'visible' && visible && !frame) frame = requestAnimationFrame(draw); };
    document.addEventListener('visibilitychange', onVisibility);
    return () => { observer.disconnect(); document.removeEventListener('visibilitychange', onVisibility); if (frame) cancelAnimationFrame(frame); };
  });
</script>
<span class="orb" role="img" aria-label={`${label}: ${orbState}`} style={`width:${size}px;height:${size}px`}><canvas bind:this={canvas} style={`width:${size}px;height:${size}px`} aria-hidden="true"></canvas></span>
<style>.orb{display:block;flex:0 0 auto}canvas{display:block;filter:drop-shadow(0 0 9px color-mix(in srgb,var(--color-accent) 18%,transparent))}</style>
