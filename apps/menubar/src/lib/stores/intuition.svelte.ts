// Intuition store — reactive state for intuition signals.

export interface IntuitionSignal {
  kind: string;
  frame_id?: string;
  timestamp: string;
  confidence: number;
}

function createIntuitionStore() {
  let signals = $state<IntuitionSignal[]>([]);
  let hasRecent = $state(false);

  return {
    get signals() { return signals; },
    get hasRecent() { return hasRecent; },

    update(data: any) {
      // Intuition signals from focus_gate signals list.
      const gateSignals = data.focus_gate?.signals ?? [];
      signals = gateSignals.slice(-20); // Keep last 20.
      hasRecent = signals.length > 0;
    },
  };
}

export const intuitionStore = createIntuitionStore();
