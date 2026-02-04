// Gate store — Focus Gate candidates and signals.

export interface GateCandidate {
  id: string;
  kind: string;
  label: string;
  pressure: number;
  pinned: boolean;
  status: string;
}

export interface GateSignal {
  kind: string;
  frame_id?: string;
  timestamp: string;
  confidence: number;
}

function createGateStore() {
  let candidates = $state<GateCandidate[]>([]);
  let signals = $state<GateSignal[]>([]);

  return {
    get candidates() { return candidates; },
    get signals() { return signals; },
    get surfacedCount() {
      return candidates.filter(c => c.status === 'Surfaced').length;
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
