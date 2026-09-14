// Decision/constraint validation helpers extracted from tools.ts (#600).
// Kept dependency-free so tests can transpile this module in isolation
// (background-job-tools.test.mjs pattern) without loading the full tool graph.
const TASK_PATTERNS =
  /\b(Fix all|Implement|Add|Create|Update|Remove|Check|Verify|Test|Build(?! agents?\b)|Deploy|NEXT:|Signal:)\b/i;
const DEBUG_PATTERNS =
  /(\bDEBUG\b|\bTODO\b|\bstack trace\b|\berror\b|\bfailed\b|\bcrash\b|\bbroken\b|\bbug\b|\bat line\b|\bTraceback\b)/i;
export const SELF_REF_PATTERNS =
  /\b(I think|I tried|I'm working|I'm doing|working on|trying to|in this session|while I was|I was just)\b/i;
const MULTI_SENTENCE = /\.\s+\w/;

export function validateDecision(decision: string): { valid: boolean; reason?: string } {
  // §AsccSections: decisions = crystallized choices that guide future action.
  // Keep the public validator aligned with pushDelta's canonical Focus State limit.
  if (decision.length > 160) {
    return {
      valid: false,
      reason:
        "Too verbose — distill to ONE crystallized sentence (max 160 chars). Use scratchpad for elaboration.",
    };
  }
  const taskMatch = TASK_PATTERNS.exec(decision);
  if (taskMatch) {
    return {
      valid: false,
      reason: `Sounds like a task list (matched: ${taskMatch[0]}) — decisions capture ARCHITECTURAL CHOICES, not implementation plans. Write task in scratchpad. Distill the decision.`,
    };
  }
  if (DEBUG_PATTERNS.test(decision)) {
    return {
      valid: false,
      reason:
        "Sounds like debugging metadata — decisions are stable choices, not investigation notes. Move to scratchpad.",
    };
  }
  if (SELF_REF_PATTERNS.test(decision)) {
    return {
      valid: false,
      reason:
        "Sounds like stream-of-consciousness — decisions should be objective architectural statements. Distill from scratchpad notes.",
    };
  }
  if (MULTI_SENTENCE.test(decision)) {
    return {
      valid: false,
      reason:
        "Multiple sentences — decisions should be ONE crystallized sentence. Per §AsccSections (<=160 chars).",
    };
  }
  return { valid: true };
}

export function validateConstraint(constraint: string, source?: string): { valid: boolean; reason?: string } {
  // §AsccSections: constraints = DISCOVERED REQUIREMENTS (not self-imposed tasks)
  // Constraint is a hard boundary from environment/architecture, not "I should do X".
  // Operator directives are discovered requirements even when phrased with "must/must not".
  const operatorDirective =
    /operator directive/i.test(source || "") || /^operator directive\b/i.test(constraint);
  if (constraint.length > 200) {
    return { valid: false, reason: "Too verbose — distill to one sentence (max 200 chars)." };
  }
  if (!operatorDirective && TASK_PATTERNS.test(constraint)) {
    return {
      valid: false,
      reason:
        "Sounds like a self-imposed task — constraints are DISCOVERED REQUIREMENTS from environment/architecture. Not 'I will do X'.",
    };
  }
  if (!operatorDirective && /\b(will|should|must|need to|going to)\b/i.test(constraint)) {
    return {
      valid: false,
      reason:
        "Sounds like self-imposed obligation — constraints are discovered requirements from environment, not agent commitments. Use scratchpad.",
    };
  }
  return { valid: true };
}
