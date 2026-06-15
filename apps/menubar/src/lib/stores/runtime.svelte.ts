export interface RuntimeSnapshot {
  health: any | null;
  doctor: any | null;
  projectIdentity: any | null;
  focusFrame: any | null;
  trajectory: any | null;
  workpoint: any | null;
  workpointResume: any | null;
  session: any | null;
  workLoop: any | null;
  workLoopHealth: any | null;
  workLoopCheckpoints: any | null;
  memoryTelemetry: any | null;
  ontologyContractsVersion: string | null;
  ontologyContractsCount: number;
  recentEventCount: number;
  tokenBudget: any | null;
  cacheMetadata: any | null;
  predictionsRecent: any | null;
  predictionsStats: any | null;
  metacogStatus: any | null;
  metacogEvaluations: any | null;
  snapshotsRecent: any | null;
  lineageHead: any | null;
  releaseProof: any | null;
}

function createRuntimeStore() {
  let snapshot = $state<RuntimeSnapshot>({
    health: null,
    doctor: null,
    projectIdentity: null,
    focusFrame: null,
    trajectory: null,
    workpoint: null,
    workpointResume: null,
    session: null,
    workLoop: null,
    workLoopHealth: null,
    workLoopCheckpoints: null,
    memoryTelemetry: null,
    ontologyContractsVersion: null,
    ontologyContractsCount: 0,
    recentEventCount: 0,
    tokenBudget: null,
    cacheMetadata: null,
    predictionsRecent: null,
    predictionsStats: null,
    metacogStatus: null,
    metacogEvaluations: null,
    snapshotsRecent: null,
    lineageHead: null,
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
