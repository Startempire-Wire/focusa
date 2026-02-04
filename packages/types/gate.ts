// Gate types — mirrors focusa-core gate module

export interface GateCandidate {
  id: string;
  kind: string;
  label: string;
  pressure: number;
  pinned: boolean;
  status: 'Pending' | 'Surfaced' | 'Suppressed' | 'Resolved';
  source_frame_id: string | null;
  created_at: string;
  suppressed_until: string | null;
}

export interface GateSignal {
  kind: string;
  frame_id: string | null;
  payload: string;
  timestamp: string;
  confidence: number;
}
