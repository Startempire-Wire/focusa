// Intuition types — mirrors focusa-core intuition module

export interface IntuitionSignal {
  kind: IntuitionSignalKind;
  frame_id: string | null;
  message: string;
  confidence: number;
  observed_at: string;
}

export type IntuitionSignalKind =
  | 'ErrorRepetition'
  | 'Inactivity'
  | 'DeepNesting'
  | 'LongRunning'
  | 'TopicDrift'
  | 'HighConfidence';
