import { validateMissionCanvasContract } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import { authorityFromProjection, sameWorkstreamAuthority as sameScope } from './exact-scope';
import type { ResolvedWorkspaceProjection, WorkstreamAuthorityContext } from './types';

export type ProjectionState =
  | { kind: 'unbound' }
  | { kind: 'loading'; scope: WorkstreamAuthorityContext }
  | { kind: 'refreshing'; scope: WorkstreamAuthorityContext; projection: ResolvedWorkspaceProjection }
  | { kind: 'ready'; scope: WorkstreamAuthorityContext; projection: ResolvedWorkspaceProjection }
  | { kind: 'stale'; scope: WorkstreamAuthorityContext; projection: ResolvedWorkspaceProjection; reason: string }
  | { kind: 'blocked'; scope?: WorkstreamAuthorityContext; reason: string }
  | { kind: 'error'; scope?: WorkstreamAuthorityContext; reason: string };

export type ProjectionLoader = (scope: WorkstreamAuthorityContext) => Promise<unknown>;

export class MissionCanvasProjectionController {
  state = $state<ProjectionState>({ kind: 'unbound' });
  #requestGeneration = 0;

  constructor(private readonly loader: ProjectionLoader) {}

  async load(scope: WorkstreamAuthorityContext): Promise<void> {
    const generation = ++this.#requestGeneration;
    const prior = this.currentProjection();
    this.state = prior && sameScope(authorityFromProjection(prior), scope)
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
      if (!sameScope(scope, authorityFromProjection(projection))) {
        this.state = { kind: 'blocked', scope, reason: 'projection_scope_mismatch' };
        return;
      }
      if (prior && sameScope(authorityFromProjection(prior), scope) && projection.projection_revision < prior.projection_revision) {
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

  accept(scope: WorkstreamAuthorityContext, value: unknown): boolean {
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
    if (!sameScope(scope, authorityFromProjection(projection))) {
      this.state = { kind: 'blocked', scope, reason: 'projection_scope_mismatch' };
      return false;
    }
    if (prior && sameScope(authorityFromProjection(prior), scope) && projection.projection_revision < prior.projection_revision) {
      this.state = { kind: 'stale', scope, projection: prior, reason: 'projection_revision_regressed' };
      return false;
    }
    this.state = { kind: 'ready', scope, projection };
    return true;
  }

  markStale(reason: string): void {
    const projection = this.currentProjection();
    if (projection && (this.state.kind === 'ready' || this.state.kind === 'refreshing' || this.state.kind === 'stale')) {
      this.state = { kind: 'stale', scope: this.state.scope, projection, reason };
    }
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
