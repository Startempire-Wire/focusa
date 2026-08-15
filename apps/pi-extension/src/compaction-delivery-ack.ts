// Delivery-acknowledgment decision for the compaction resume packet.
// Dependency-free on purpose: this module must be importable by both bun
// tests and the node-based spec gate (node resolves .ts imports only when
// the module itself has no unresolved .js-extension imports).
//
// sendMessage() returns void, so receipt is only observable via the harness
// lifecycle: the queued nextTurn message is consumed when the next agent
// turn starts. That is only truthful for the session that queued the
// delivery, so the delivery key must end with the current session frame key.

export function compactionDeliveryAckEligible(
  deliveryKey: string | undefined,
  deliveryState: string | undefined,
  frameKey: string
): boolean {
  if (deliveryState !== "unknown_completion" && deliveryState !== "deferred_to_next_turn") {
    return false;
  }
  if (!deliveryKey) return false;
  const suffix = `:${frameKey || "session"}`;
  return deliveryKey.endsWith(suffix);
}
