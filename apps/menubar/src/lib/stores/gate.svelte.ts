// Gate store — Focus Gate candidates and signals.

export interface GateCandidate {
  id: string;
  kind: string;
  label: string;
  pressure: number;
  pinned: boolean;
  state: string; // "latent" | "surfaced" | "suppressed" | "resolved"
  created_at?: string;
  last_seen_at?: string;
  stale_advisory?: boolean;
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

const DEFAULT_RECENCY_WINDOW_MS = 7 * 24 * 60 * 60 * 1000;
const MIN_VISIBLE_PRESSURE = 0.05;
const MAX_VISIBLE_CANDIDATES = 20;

function seenAt(candidate: GateCandidate): number {
  const raw = candidate.last_seen_at || candidate.created_at || '';
  const time = raw ? Date.parse(raw) : Number.NaN;
  return Number.isFinite(time) ? time : 0;
}

function withStaleLabel(candidate: GateCandidate, now = Date.now()): GateCandidate {
  const ageMs = now - seenAt(candidate);
  return {
    ...candidate,
    stale_advisory: ageMs > DEFAULT_RECENCY_WINDOW_MS,
  };
}

function visibleCandidate(candidate: GateCandidate, now = Date.now()): boolean {
  const candidateWithAge = withStaleLabel(candidate, now);
  if (candidateWithAge.pinned || candidateWithAge.state === 'surfaced') return true;
  if (!candidateWithAge.stale_advisory) return true;
  return Number(candidateWithAge.pressure || 0) >= MIN_VISIBLE_PRESSURE;
}

function createGateStore() {
  let candidates = $state<GateCandidate[]>([]);
  let hiddenCandidateCount = $state(0);
  let signals = $state<GateSignal[]>([]);

  return {
    get candidates() { return candidates; },
    get hiddenCandidateCount() { return hiddenCandidateCount; },
    get signals() { return signals; },
    get surfacedCount() {
      return candidates.filter(c => c.state === 'surfaced').length;
    },

    update(gateData: any) {
      if (gateData) {
        const now = Date.now();
        const rawCandidates: GateCandidate[] = gateData.candidates ?? [];
        const visible = rawCandidates
          .filter((candidate) => visibleCandidate(candidate, now))
          .map((candidate) => withStaleLabel(candidate, now))
          .sort((a, b) => Number(b.pressure || 0) - Number(a.pressure || 0));
        candidates = visible.slice(0, MAX_VISIBLE_CANDIDATES);
        hiddenCandidateCount = Math.max(0, rawCandidates.length - candidates.length);
        signals = (gateData.signals ?? []).slice(-20);
      }
    },
  };
}

export const gateStore = createGateStore();
