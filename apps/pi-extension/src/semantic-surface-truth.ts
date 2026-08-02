export type SemanticSurfaceTruth = {
  state: string;
  operationCount: number;
  mutationCount: number;
  supportedCount: number;
  schemaOnlyCount: number;
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
  visibleLimit = 6
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
  const gaps = operations.filter(
    (item) =>
      item && typeof item === "object" && (item as Record<string, unknown>).availability !== "supported"
  );
  const operationLines = gaps.slice(0, Math.max(0, visibleLimit)).map((item) => {
    const operation = item as Record<string, unknown>;
    return `  ${bounded(operation.operation_id || "unknown", 48)} · ${bounded(operation.kind || "read", 12)} · ${bounded(operation.availability || "unknown", 24)}`;
  });
  if (gaps.length > visibleLimit) {
    operationLines.push(
      `  … ${gaps.length - visibleLimit} more gaps · focusa semantic-integrity registry for full truth`
    );
  }
  return {
    state: degraded ? "degraded" : bounded(status.state || "unknown", 40),
    operationCount,
    mutationCount,
    supportedCount,
    schemaOnlyCount,
    operationLines,
  };
}
