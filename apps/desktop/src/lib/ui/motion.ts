import { quartOut } from 'svelte/easing';
import type { TransitionConfig } from 'svelte/transition';

export type MotionMode = 'system' | 'full' | 'reduced';
const STORAGE_KEY = 'focusa.desktop.motion_preference.v1';

export function scene(_node: Element, { duration = 220, y = 5 }: { duration?: number; y?: number } = {}): TransitionConfig {
  const explicitReduced = typeof document !== 'undefined' && document.documentElement.dataset.motion === 'reduced';
  const systemReduced = typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches && document.documentElement.dataset.motion !== 'full';
  const reduced = explicitReduced || systemReduced;
  return {
    duration: reduced ? 110 : duration,
    easing: quartOut,
    css: (t) => `opacity:${t};transform:translate3d(0,${reduced ? 0 : (1 - t) * y}px,0)`
  };
}

export function readMotionPreference(): MotionMode {
  if (typeof window === 'undefined') return 'system';
  const stored = window.localStorage.getItem(STORAGE_KEY);
  return stored === 'full' || stored === 'reduced' ? stored : 'system';
}

export function pop(_node: Element, { duration = 190, y = 4, scale = .985 }: { duration?: number; y?: number; scale?: number } = {}): TransitionConfig {
  const reduced = typeof document !== 'undefined' && (document.documentElement.dataset.motion === 'reduced' || (matchMedia('(prefers-reduced-motion: reduce)').matches && document.documentElement.dataset.motion !== 'full'));
  return {
    duration: reduced ? 100 : duration,
    easing: quartOut,
    css: (t) => `opacity:${t};transform:translate3d(0,${reduced ? 0 : (1 - t) * y}px,0) scale(${reduced ? 1 : scale + (1 - scale) * t})`
  };
}

export function installMotionPreference(): () => void {
  if (typeof window === 'undefined') return () => {};
  document.documentElement.dataset.motion = readMotionPreference();
  return () => {};
}

export function setMotionPreference(mode: MotionMode): void {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(STORAGE_KEY, mode);
  document.documentElement.dataset.motion = mode;
}
