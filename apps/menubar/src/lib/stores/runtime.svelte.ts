export interface RuntimeSnapshot {
  health: any | null;
  workpoint: any | null;
  workLoop: any | null;
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
    workpoint: null,
    workLoop: null,
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
