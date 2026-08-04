import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export const FOCUSA_DEEP_LINK_EVENT = 'focusa://deep-link-intent';

export type FocusaDeepLinkRoute = 'connect' | 'mission' | 'card' | 'workpoint';

export interface FocusaDeepLinkIntent {
  schema: 'focusa.deep_link_intent.v1';
  route: FocusaDeepLinkRoute;
  target_ref?: string;
  governed_connect_payload?: string;
}

export type MenubarTab = 'pair' | 'mission-canvas' | 'workpoint';

export function tabForFocusaDeepLink(intent: FocusaDeepLinkIntent): MenubarTab {
  switch (intent.route) {
    case 'connect':
      return 'pair';
    case 'workpoint':
      return 'workpoint';
    case 'mission':
    case 'card':
      return 'mission-canvas';
  }
}

/**
 * Listen before taking the cold-start queue so no activation can fall between
 * frontend registration and the Rust ready transition.
 */
export async function subscribeFocusaDeepLinks(
  onIntent: (intent: FocusaDeepLinkIntent) => void,
): Promise<UnlistenFn> {
  const unlisten = await listen<FocusaDeepLinkIntent>(FOCUSA_DEEP_LINK_EVENT, (event) => {
    onIntent(event.payload);
  });
  try {
    const pending = await invoke<FocusaDeepLinkIntent[]>('focusa_take_deep_link_intents');
    for (const intent of pending) onIntent(intent);
    return unlisten;
  } catch (error) {
    unlisten();
    throw error;
  }
}
