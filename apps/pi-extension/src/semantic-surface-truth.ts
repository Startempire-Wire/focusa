export type SemanticSurfaceTruth = {
  state: string;
  operationCount: number;
  mutationCount: number;
  supportedCount: number;
  schemaOnlyCount: number;
  displayedOperationCount: number;
  remainingOperationCount: number;
  operationLines: string[];
};

function bounded(value: unknown, max: number): string {
  const text = String(value || "")
    .replace(/\s+/g, " ")
    .trim();
  return text.length > max ? `${text.slice(0, max - 1)}…` : text;
}

function finiteCount(value: unknown, fallback: number): number {
  const count = Number(value);
  return Number.isFinite(count) && count >= 0 ? count : fallback;
}

export function semanticSurfaceTruth(
  statusResult: unknown,
  registryResult: unknown,
  visibleLimit = 4
): SemanticSurfaceTruth {
  const status =
    statusResult && typeof statusResult === "object" ? (statusResult as Record<string, unknown>) : {};
  const data = status.data && typeof status.data === "object" ? (status.data as Record<string, unknown>) : {};
  const registry =
    registryResult && typeof registryResult === "object" ? (registryResult as Record<string, unknown>) : {};
  const operations = Array.isArray(registry.items) ? registry.items : [];
  const supportedFromRegistry = operations.filter(
    (item) =>
      item && typeof item === "object" && (item as Record<string, unknown>).availability === "supported"
  ).length;
  const schemaOnlyFromRegistry = operations.filter(
    (item) =>
      item && typeof item === "object" && (item as Record<string, unknown>).availability === "schema_only"
  ).length;
  const supportedCount = finiteCount(data.supported_operations, supportedFromRegistry);
  const schemaOnlyCount = finiteCount(data.schema_only_operations, schemaOnlyFromRegistry);
  const operationCount = finiteCount(data.registered_operations, operations.length);
  const mutationCount = operations.filter(
    (item) => item && typeof item === "object" && (item as Record<string, unknown>).kind === "mutation"
  ).length;
  const degraded = status.degraded === true || registry.degraded === true || schemaOnlyCount > 0;
  const visibleOperations = operations.filter((item) => item && typeof item === "object");
  const boundedVisibleLimit = Number.isFinite(visibleLimit) ? Math.max(0, Math.floor(visibleLimit)) : 4;
  const displayedOperationCount = Math.min(visibleOperations.length, boundedVisibleLimit);
  const remainingOperationCount = Math.max(0, visibleOperations.length - displayedOperationCount);
  const operationLines = visibleOperations.slice(0, displayedOperationCount).map((item) => {
    const operation = item as Record<string, unknown>;
    return `  ${bounded(operation.operation_id || "unknown", 48)} · ${bounded(operation.kind || "read", 12)} · ${bounded(operation.availability || "unknown", 24)}`;
  });
  if (remainingOperationCount > 0) {
    operationLines.push(
      `  … ${remainingOperationCount} remaining · showing ${displayedOperationCount}/${visibleOperations.length} · focusa semantic-integrity registry`
    );
  }
  return {
    state: degraded ? "degraded" : bounded(status.state || "unknown", 40),
    operationCount,
    mutationCount,
    supportedCount,
    schemaOnlyCount,
    displayedOperationCount,
    remainingOperationCount,
    operationLines,
  };
}
