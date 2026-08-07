import { authorityFromEvent, authorityFromProjection, sameWorkstreamAuthority } from './exact-scope';
import type {
  ProjectionLifecycleEvent,
  ResolvedWorkspaceProjection,
  WorkstreamAuthorityContext
} from './types';

export type CapabilityNotification = Readonly<{
  kind: 'capability_lost';
  affectedContributionIds: readonly string[];
  message: string;
}>;

export type CapabilityRefreshResult = Readonly<{
  projection: ResolvedWorkspaceProjection;
  notification?: CapabilityNotification;
  restoredContributionIds: readonly string[];
}>;

export class CapabilityLossController {
  #generation = 0;

  constructor(
    private readonly projectionRefresh: () => Promise<ResolvedWorkspaceProjection>
  ) {}

  async handle(
    events: readonly ProjectionLifecycleEvent[],
    current: ResolvedWorkspaceProjection,
    authority: WorkstreamAuthorityContext
  ): Promise<CapabilityRefreshResult | undefined> {
    const capabilityEvents = events.filter((event) => event.event_kind === 'capability_changed');
    if (capabilityEvents.length === 0) return undefined;
    if (!sameWorkstreamAuthority(authorityFromProjection(current), authority)) return undefined;
    if (capabilityEvents.some((event) => !sameWorkstreamAuthority(authorityFromEvent(event), authority))) return undefined;
    if (capabilityEvents.some((event) =>
      event.projection_revision < current.projection_revision
      || event.layout_revision < current.layout_revision
    )) return undefined;

    const generation = ++this.#generation;
    const next = await this.projectionRefresh();
    if (generation !== this.#generation) return undefined;
    if (!sameWorkstreamAuthority(authorityFromProjection(next), authority)) return undefined;
    if (next.projection_revision < current.projection_revision || next.layout_revision < current.layout_revision) return undefined;

    const priorIds = new Set(current.eligible_contributions.map((item) => item.contribution_id));
    const nextIds = new Set(next.eligible_contributions.map((item) => item.contribution_id));
    const lost = [...priorIds].filter((id) => !nextIds.has(id)).sort();
    const restored = [...nextIds].filter((id) => !priorIds.has(id)).sort();
    return Object.freeze({
      projection: next,
      restoredContributionIds: Object.freeze(restored),
      notification: lost.length > 0
        ? Object.freeze({
            kind: 'capability_lost' as const,
            affectedContributionIds: Object.freeze(lost),
            message: lost.length === 1
              ? 'A capability became unavailable; the Canvas was recomposed.'
              : `${lost.length} capabilities became unavailable; the Canvas was recomposed.`
          })
        : undefined
    });
  }

  cancel(): void {
    this.#generation += 1;
  }
}
