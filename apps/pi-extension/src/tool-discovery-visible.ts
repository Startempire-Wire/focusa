export function modelVisibleDiscoveryPayload(
  label: string,
  payload: unknown,
  store: (kind: string, content: string) => string,
  maxChars = 12_000
): string {
  const serialized = JSON.stringify(payload, null, 2);
  if (serialized.length <= maxChars) return `${label}\n${serialized}`;
  const handleId = store("text", serialized);
  return `${label}\n${serialized.slice(0, maxChars)}…\n[HANDLE:text:${handleId}]\nUse /focusa-rehydrate ${handleId} to retrieve the complete bounded payload.`;
}
