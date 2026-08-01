import createClient, { type ClientOptions } from "openapi-fetch";
import type { paths } from "./schema.js";
import type {
  SemanticPairActionRequest,
  SemanticPairScope,
} from "./semantic-pair.js";

export type FocusaSpec135Client = ReturnType<typeof createClient<paths>>;

/** Create a typed Focusa client from the generated OpenAPI 3.0.3 contract. */
export function createFocusaSpec135Client(options: ClientOptions): FocusaSpec135Client {
  return createClient<paths>(options);
}

export interface SemanticPairTransport {
  status(scope: SemanticPairScope): Promise<unknown>;
  operations(scope: SemanticPairScope): Promise<unknown>;
  portfolio(scope: SemanticPairScope, pairId?: string): Promise<unknown>;
  invoke<T = unknown>(request: SemanticPairActionRequest): Promise<T>;
}

/** Typed binding for Spec144 routes not yet represented by the Spec135 OpenAPI snapshot. */
export function createSemanticPairClient(
  baseUrl: string,
  fetcher: typeof fetch = fetch,
): SemanticPairTransport {
  const query = (scope: SemanticPairScope) => new URLSearchParams({
    project_root: scope.project_root,
    continuity_id: scope.continuity_id,
  }).toString();
  const json = async <T>(path: string, init?: RequestInit): Promise<T> => {
    const response = await fetcher(`${baseUrl.replace(/\/$/, "")}${path}`, init);
    if (!response.ok) throw new Error(`Semantic Pair request failed: ${response.status}`);
    return response.json() as Promise<T>;
  };
  return {
    status: (scope) => json(`/v1/semantic-integrity/status?${query(scope)}`),
    operations: (scope) => json(`/v1/semantic-integrity/operations?${query(scope)}&limit=100`),
    portfolio: (scope, pairId) => json(`/v1/semantic-integrity/operations/semantic_pair.get`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({ operation_id: "semantic_pair.get", scope, pair_id: pairId }),
    }),
    invoke: <T>(request: SemanticPairActionRequest) => json<T>(
      `/v1/semantic-integrity/operations/${encodeURIComponent(request.operation_id)}`,
      { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(request) },
    ),
  };
}
