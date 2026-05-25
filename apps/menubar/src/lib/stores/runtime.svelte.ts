export interface RuntimeSnapshot {
  health: any | null;
  doctor: any | null;
  projectIdentity: any | null;
  trajectory: any | null;
  workpoint: any | null;
  workpointResume: any | null;
  workLoop: any | null;
  workLoopHealth: any | null;
  memoryTelemetry: any | null;
  ontologyContractsVersion: string | null;
  ontologyContractsCount: number;
  recentEventCount: number;
  tokenBudget: any | null;
  cacheMetadata: any | null;
  releaseProof: any | null;
}

function createRuntimeStore() {
  let snapshot = $state<RuntimeSnapshot>({
    health: null,
    doctor: null,
    projectIdentity: null,
    trajectory: null,
    workpoint: null,
    workpointResume: null,
    workLoop: null,
    workLoopHealth: null,
    memoryTelemetry: null,
    ontologyContractsVersion: null,
    ontologyContractsCount: 0,
    recentEventCount: 0,
    tokenBudget: null,
    cacheMetadata: null,
    releaseProof: null,
  });
  let errorMsg = $state<string | null>(null);

  return {
    get snapshot() { return snapshot; },
    get errorMsg() { return errorMsg; },

    update(parts: Partial<RuntimeSnapshot>) {
      snapshot = { ...snapshot, ...parts };
      errorMsg = null;
    },

    setError(msg: string) {
      errorMsg = msg;
    },
  };
}

export const runtimeStore = createRuntimeStore();
