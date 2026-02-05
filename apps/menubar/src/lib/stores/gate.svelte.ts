// Gate store — Focus Gate candidates and signals.

export interface GateCandidate {
  id: string;
  kind: string;
  label: string;
  pressure: number;
  pinned: boolean;
  state: string; // "latent" | "surfaced" | "suppressed" | "resolved"
}

export interface GateSignal {
  id: string;
  ts: string;
  origin: string;
  kind: string;
  frame_context?: string;
  summary: string;
  tags: string[];
}

function createGateStore() {
  let candidates = $state<GateCandidate[]>([]);
  let signals = $state<GateSignal[]>([]);

  return {
    get candidates() { return candidates; },
    get signals() { return signals; },
    get surfacedCount() {
      return candidates.filter(c => c.state === 'surfaced').length;
    },

    update(gateData: any) {
      if (gateData) {
        candidates = gateData.candidates ?? [];
        signals = (gateData.signals ?? []).slice(-20);
      }
    },
  };
}

export const gateStore = createGateStore();
