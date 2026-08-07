import type { Component } from 'svelte';
import type { MissionCanvasClient } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
import { validateMissionCanvasContract } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import type { ContributionKind, OperationBinding, ResolvedContribution, ResolvedWorkspaceProjection } from './types';

export interface ContributionRendererProps {
  contribution: ResolvedContribution;
  projection: ResolvedWorkspaceProjection;
  client?: MissionCanvasClient;
  onOperation?: (binding: OperationBinding) => void | Promise<void>;
}

export interface TrustedContributionRenderer {
  rendererBindingId: string;
  semanticBindingIds?: readonly string[];
  contributionKinds?: readonly ContributionKind[];
  component: Component<any>;
  componentProps?: Readonly<Record<string, unknown>>;
}

export interface ResolvedContributionRenderer {
  component: Component<any>;
  componentProps: Readonly<Record<string, unknown>>;
}

export type ContributionRendererFailureReason =
  | 'invalid_contribution'
  | 'unknown_renderer_binding'
  | 'semantic_binding_mismatch'
  | 'contribution_kind_mismatch';

/**
 * A bounded resolver diagnostic.  It intentionally contains binding identity
 * only; renderer internals, transport payloads, and canonical content never
 * become a client-local error surface.
 */
export interface ContributionRendererDiagnostic {
  reason: ContributionRendererFailureReason;
  contributionId?: string;
  rendererBindingId?: string;
  semanticBindingId?: string;
}

export type ContributionRendererResolution =
  | { status: 'resolved'; renderer: ResolvedContributionRenderer }
  | { status: 'blocked'; diagnostic: ContributionRendererDiagnostic };

const RESERVED_COMPONENT_PROPS = new Set(['contribution', 'projection', 'client', 'onOperation']);

const CONTRIBUTION_KINDS = new Set<ContributionKind>([
  'work_surface_strip',
  'focused_work_surface',
  'inspector',
  'inspector_section',
  'work_rail',
  'steering_queue',
  'follow_up_queue',
  'prompt_editor',
  'scope_bar',
  'activity_navigation',
  'toolbar_control',
  'contextual_action',
  'transient_notification',
  'generated_surface'
]);

/**
 * The registry is the trusted, executable side of generated renderer
 * metadata.  The projection supplies only opaque binding identities and
 * canonical data; it cannot provide a component, a fallback, or component
 * props.  Core remains the owner of eligibility and layout composition.
 */
export class ContributionRendererRegistry {
  readonly #entries: ReadonlyMap<string, TrustedContributionRenderer>;

  constructor(entries: readonly TrustedContributionRenderer[]) {
    if (!Array.isArray(entries)) {
      throw new Error('Contribution renderer registry requires an entry list.');
    }

    const indexed = new Map<string, TrustedContributionRenderer>();
    for (const entry of entries) {
      const normalized = normalizeTrustedRenderer(entry);
      if (indexed.has(normalized.rendererBindingId)) {
        throw new Error(`Invalid or duplicate renderer binding: ${normalized.rendererBindingId}`);
      }
      indexed.set(normalized.rendererBindingId, normalized);
    }
    this.#entries = indexed;
  }

  /**
   * Resolve an exact generated contribution to a trusted local component.
   * `undefined` is the compatibility result consumed by existing renderers;
   * callers that need a bounded reason should use resolveWithDiagnostic().
   */
  resolve(contribution: unknown): ResolvedContributionRenderer | undefined {
    return this.resolveContributionRenderer(contribution);
  }

  resolveContributionRenderer(contribution: unknown): ResolvedContributionRenderer | undefined {
    const result = this.resolveWithDiagnostic(contribution);
    return result.status === 'resolved' ? result.renderer : undefined;
  }

  /**
   * Resolve without losing the fail-closed reason.  This is diagnostic data,
   * not a second eligibility/layout decision and is safe to expose in the
   * bounded renderer-blocked state.
   */
  resolveWithDiagnostic(contribution: unknown): ContributionRendererResolution {
    const bindings = contributionBindings(contribution);
    if (!bindings) {
      return blocked('invalid_contribution');
    }

    const entry = this.#entries.get(bindings.rendererBindingId);
    if (!entry) {
      return blocked('unknown_renderer_binding', bindings);
    }
    if (entry.semanticBindingIds && !entry.semanticBindingIds.includes(bindings.semanticBindingId)) {
      return blocked('semantic_binding_mismatch', bindings);
    }
    if (entry.contributionKinds && !entry.contributionKinds.includes(bindings.kind as ContributionKind)) {
      return blocked('contribution_kind_mismatch', bindings);
    }
    if (!isResolvedContribution(contribution)) {
      return blocked('invalid_contribution', bindings);
    }

    return {
      status: 'resolved',
      renderer: Object.freeze({
        component: entry.component,
        componentProps: entry.componentProps ?? EMPTY_PROPS
      })
    };
  }

  has(rendererBindingId: unknown): boolean {
    return typeof rendererBindingId === 'string'
      && rendererBindingId.trim() === rendererBindingId
      && rendererBindingId.length > 0
      && this.#entries.has(rendererBindingId);
  }
}

/**
 * Named call seam used by the Desktop layout renderer and the executable
 * Mission Canvas call graph.  The registry remains the only owner of trusted
 * component resolution; this helper does not inspect layout or infer state.
 */
export function resolveContributionRenderer(
  registry: ContributionRendererRegistry,
  contribution: unknown
): ResolvedContributionRenderer | undefined {
  return registry.resolveContributionRenderer(contribution);
}

const EMPTY_PROPS: Readonly<Record<string, unknown>> = Object.freeze({});

function normalizeTrustedRenderer(entry: TrustedContributionRenderer): TrustedContributionRenderer {
  if (!entry || typeof entry !== 'object') {
    throw new Error('Invalid contribution renderer entry.');
  }
  const rendererBindingId = exactId(entry.rendererBindingId, 'renderer binding');
  if (!isTrustedComponent(entry.component)) {
    throw new Error(`Invalid trusted renderer component: ${rendererBindingId}`);
  }

  const semanticBindingIds = normalizeIdList(entry.semanticBindingIds, 'semantic binding', rendererBindingId);
  const contributionKinds = normalizeKindList(entry.contributionKinds, rendererBindingId);
  const componentProps = normalizeComponentProps(entry.componentProps, rendererBindingId);

  return Object.freeze({
    rendererBindingId,
    ...(semanticBindingIds ? { semanticBindingIds } : {}),
    ...(contributionKinds ? { contributionKinds } : {}),
    component: entry.component,
    componentProps
  });
}

function normalizeIdList(
  values: readonly string[] | undefined,
  label: string,
  rendererBindingId: string
): readonly string[] | undefined {
  if (values === undefined) return undefined;
  if (!Array.isArray(values) || values.length === 0) {
    throw new Error(`Invalid ${label} list for ${rendererBindingId}`);
  }
  const normalized = values.map((value) => exactId(value, label));
  if (new Set(normalized).size !== normalized.length) {
    throw new Error(`Duplicate ${label} for ${rendererBindingId}`);
  }
  return Object.freeze(normalized);
}

function normalizeKindList(
  values: readonly ContributionKind[] | undefined,
  rendererBindingId: string
): readonly ContributionKind[] | undefined {
  if (values === undefined) return undefined;
  if (!Array.isArray(values) || values.length === 0 || values.some((value) => !CONTRIBUTION_KINDS.has(value))) {
    throw new Error(`Invalid contribution kind list for ${rendererBindingId}`);
  }
  if (new Set(values).size !== values.length) {
    throw new Error(`Duplicate contribution kind for ${rendererBindingId}`);
  }
  return Object.freeze([...values]);
}

function normalizeComponentProps(
  value: Readonly<Record<string, unknown>> | undefined,
  rendererBindingId: string
): Readonly<Record<string, unknown>> {
  if (value === undefined) return EMPTY_PROPS;
  if (!isRecord(value)) {
    throw new Error(`Invalid component props for ${rendererBindingId}`);
  }
  for (const key of Object.keys(value)) {
    if (RESERVED_COMPONENT_PROPS.has(key)) {
      throw new Error(`Renderer props cannot override ${key}: ${rendererBindingId}`);
    }
  }
  return cloneAndFreeze(value, new WeakSet<object>()) as Readonly<Record<string, unknown>>;
}

function cloneAndFreeze(value: unknown, seen: WeakSet<object>): unknown {
  if (Array.isArray(value)) {
    if (seen.has(value)) throw new Error('Cyclic renderer component props.');
    seen.add(value);
    const copy = value.map((item) => cloneAndFreeze(item, seen));
    seen.delete(value);
    return Object.freeze(copy);
  }
  if (isRecord(value)) {
    if (seen.has(value)) throw new Error('Cyclic renderer component props.');
    seen.add(value);
    const copy: Record<string, unknown> = {};
    for (const [key, item] of Object.entries(value)) copy[key] = cloneAndFreeze(item, seen);
    seen.delete(value);
    return Object.freeze(copy);
  }
  if (value !== null && typeof value === 'object') {
    throw new Error('Renderer component props must be plain values.');
  }
  return value;
}

function contributionBindings(value: unknown): {
  contributionId?: string;
  rendererBindingId: string;
  semanticBindingId: string;
  kind: string;
} | undefined {
  if (!isRecord(value)) return undefined;
  const rendererBindingId = exactIdOrUndefined(value.renderer_binding_id);
  const semanticBindingId = exactIdOrUndefined(value.semantic_binding_id);
  const kind = typeof value.kind === 'string' ? value.kind : undefined;
  if (!rendererBindingId || !semanticBindingId || !kind) return undefined;
  return {
    contributionId: exactIdOrUndefined(value.contribution_id),
    rendererBindingId,
    semanticBindingId,
    kind
  };
}

function isResolvedContribution(value: unknown): value is ResolvedContribution {
  if (!isRecord(value)) return false;
  if (!validateMissionCanvasContract('ResolvedContribution', value).valid) return false;
  if (!exactIdOrUndefined(value.contribution_id)
    || !exactIdOrUndefined(value.semantic_binding_id)
    || !exactIdOrUndefined(value.renderer_binding_id)
    || typeof value.kind !== 'string'
    || !CONTRIBUTION_KINDS.has(value.kind as ContributionKind)
    || !Number.isInteger(value.contribution_revision)
    || (value.contribution_revision as number) < 0
    || !Array.isArray(value.operation_ids)
    || value.operation_ids.some((operation) => !exactIdOrUndefined(operation))) {
    return false;
  }

  return validNestedContract('CanonicalRef', value.data_ref, (dataRef) => Boolean(
    exactIdOrUndefined(dataRef.kind)
    && exactIdOrUndefined(dataRef.ref)
    && validRevision(dataRef.revision)
  ))
    && validNestedContract('AuthorityDescriptor', value.authority)
    && validNestedContract('FreshnessDescriptor', value.freshness)
    && validNestedContract('GeometryPreference', value.resolved_geometry)
    && validNestedContract('AccessibilityDescriptor', value.accessibility, (accessibility) => Boolean(
      exactIdOrUndefined(accessibility.label)
      && exactIdOrUndefined(accessibility.landmark_role)
      && exactIdOrUndefined(accessibility.focus_semantic_id)
    ))
    && (value.evidence_refs === undefined
      || (Array.isArray(value.evidence_refs) && value.evidence_refs.every((ref) => exactIdOrUndefined(ref))));
}

function validNestedContract(
  schema: string,
  value: unknown,
  extra?: (record: Record<string, unknown>) => boolean
): boolean {
  return isRecord(value)
    && validateMissionCanvasContract(schema, value).valid
    && (extra ? extra(value) : true);
}

function validRevision(value: unknown): boolean {
  return (typeof value === 'number' && Number.isFinite(value))
    || (typeof value === 'string' && value.trim().length > 0);
}

function blocked(
  reason: ContributionRendererFailureReason,
  bindings?: {
    contributionId?: string;
    rendererBindingId?: string;
    semanticBindingId?: string;
  }
): ContributionRendererResolution {
  return {
    status: 'blocked',
    diagnostic: Object.freeze({
      reason,
      ...(bindings?.contributionId ? { contributionId: bindings.contributionId } : {}),
      ...(bindings?.rendererBindingId ? { rendererBindingId: bindings.rendererBindingId } : {}),
      ...(bindings?.semanticBindingId ? { semanticBindingId: bindings.semanticBindingId } : {})
    })
  };
}

function exactId(value: unknown, label: string): string {
  const id = exactIdOrUndefined(value);
  if (!id) throw new Error(`Invalid ${label}.`);
  return id;
}

function exactIdOrUndefined(value: unknown): string | undefined {
  return typeof value === 'string' && value.length > 0 && value.trim() === value
    ? value
    : undefined;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function isTrustedComponent(value: unknown): value is Component<any> {
  return typeof value === 'function';
}
