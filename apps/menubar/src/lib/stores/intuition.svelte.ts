// Intuition store — recent intuition signals from gate.

export interface IntuitionSignal {
  kind: string;
  frame_id?: string;
  timestamp: string;
  confidence: number;
}

function createIntuitionStore() {
  let signals = $state<IntuitionSignal[]>([]);

  return {
    get signals() { return signals; },
    get hasRecent() { return signals.length > 0; },
    get count() { return signals.length; },

    update(data: any) {
      const gateSignals = data.focus_gate?.signals ?? [];
      signals = gateSignals.slice(-20);
    },
  };
}

export const intuitionStore = createIntuitionStore();
