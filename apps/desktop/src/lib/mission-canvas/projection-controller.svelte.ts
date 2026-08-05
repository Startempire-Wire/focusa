import { validateMissionCanvasContract } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import type { ExactScope, ResolvedWorkspaceProjection } from './types';

export type ProjectionState =
  | { kind: 'unbound' }
  | { kind: 'loading'; scope: ExactScope }
  | { kind: 'refreshing'; scope: ExactScope; projection: ResolvedWorkspaceProjection }
  | { kind: 'ready'; scope: ExactScope; projection: ResolvedWorkspaceProjection }
  | { kind: 'stale'; scope: ExactScope; projection: ResolvedWorkspaceProjection; reason: string }
  | { kind: 'blocked'; scope?: ExactScope; reason: string }
  | { kind: 'error'; scope?: ExactScope; reason: string };

export type ProjectionLoader = (scope: ExactScope) => Promise<unknown>;

function sameScope(left: ExactScope, right: ExactScope): boolean {
  return left.project_root === right.project_root
    && left.continuity_id === right.continuity_id
    && left.attachment_id === right.attachment_id
    && left.session_id === right.session_id
    && (left.instance_id ?? null) === (right.instance_id ?? null)
    && (left.working_subpath_id ?? null) === (right.working_subpath_id ?? null);
}

export class MissionCanvasProjectionController {
  state = $state<ProjectionState>({ kind: 'unbound' });
  #requestGeneration = 0;

  constructor(private readonly loader: ProjectionLoader) {}

  async load(scope: ExactScope): Promise<void> {
    const generation = ++this.#requestGeneration;
    const prior = this.currentProjection();
    this.state = prior && sameScope(prior.scope, scope)
      ? { kind: 'refreshing', scope, projection: prior }
      : { kind: 'loading', scope };

    try {
      const value = await this.loader(scope);
      if (generation !== this.#requestGeneration) return;

      const validation = validateMissionCanvasContract('ResolvedWorkspaceProjection', value);
      if (!validation.valid) {
        const reason = validation.errors.join(',');
        this.state = prior
          ? { kind: 'stale', scope, projection: prior, reason }
          : { kind: 'error', scope, reason };
        return;
      }

      const projection = value as ResolvedWorkspaceProjection;
      if (!sameScope(scope, projection.scope)) {
        this.state = { kind: 'blocked', scope, reason: 'projection_scope_mismatch' };
        return;
      }
      if (prior && sameScope(prior.scope, scope) && projection.projection_revision < prior.projection_revision) {
        this.state = { kind: 'stale', scope, projection: prior, reason: 'projection_revision_regressed' };
        return;
      }

      this.state = { kind: 'ready', scope, projection };
    } catch (error) {
      if (generation !== this.#requestGeneration) return;
      const reason = error instanceof Error ? error.message : 'projection_load_failed';
      this.state = prior
        ? { kind: 'stale', scope, projection: prior, reason }
        : { kind: 'error', scope, reason };
    }
  }

  accept(scope: ExactScope, value: unknown): boolean {
    this.#requestGeneration += 1;
    const prior = this.currentProjection();
    const validation = validateMissionCanvasContract('ResolvedWorkspaceProjection', value);
    if (!validation.valid) {
      const reason = validation.errors.join(',');
      this.state = prior
        ? { kind: 'stale', scope, projection: prior, reason }
        : { kind: 'error', scope, reason };
      return false;
    }

    const projection = value as ResolvedWorkspaceProjection;
    if (!sameScope(scope, projection.scope)) {
      this.state = { kind: 'blocked', scope, reason: 'projection_scope_mismatch' };
      return false;
    }
    if (prior && sameScope(prior.scope, scope) && projection.projection_revision < prior.projection_revision) {
      this.state = { kind: 'stale', scope, projection: prior, reason: 'projection_revision_regressed' };
      return false;
    }
    this.state = { kind: 'ready', scope, projection };
    return true;
  }

  markStale(reason: string): void {
    const projection = this.currentProjection();
    if (projection) this.state = { kind: 'stale', scope: projection.scope, projection, reason };
  }

  clear(): void {
    this.#requestGeneration += 1;
    this.state = { kind: 'unbound' };
  }

  private currentProjection(): ResolvedWorkspaceProjection | undefined {
    return this.state.kind === 'ready' || this.state.kind === 'refreshing' || this.state.kind === 'stale'
      ? this.state.projection
      : undefined;
  }
}
