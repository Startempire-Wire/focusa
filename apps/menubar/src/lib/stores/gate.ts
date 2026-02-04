// Gate store — reactive state for Focus Gate candidates.

export interface Candidate {
  id: string;
  kind: string;
  label: string;
  pressure: number;
  pinned: boolean;
  status: string;
}

function createGateStore() {
  let candidates = $state<Candidate[]>([]);

  return {
    get candidates() { return candidates; },
    get surfacedCount() {
      return candidates.filter(c => c.status === 'Surfaced').length;
    },
    get pinnedCount() {
      return candidates.filter(c => c.pinned).length;
    },

    update(gateData: any) {
      if (gateData?.candidates) {
        candidates = gateData.candidates;
      }
    },
  };
}

export const gateStore = createGateStore();
