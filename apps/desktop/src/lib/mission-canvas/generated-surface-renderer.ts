import GeneratedSurfaceContribution from './contributions/GeneratedSurfaceContribution.svelte';
import type { GeneratedSurfaceSnapshotResolver } from './generated-surface-types';
import type { TrustedContributionRenderer } from './contribution-renderers';
import type { ContributionKind } from './types';

export interface TrustedGeneratedSurfaceBinding {
  rendererBindingId: string;
  semanticBindingIds: readonly string[];
  contributionKinds?: readonly ContributionKind[];
  snapshotResolver: GeneratedSurfaceSnapshotResolver;
}

/**
 * Binds a canonical Renderer Registry identity to the maintained A2UI/Lit host.
 * Callers must supply exact registry and semantic IDs; kind-only fallback is forbidden.
 */
export function trustedGeneratedSurfaceRenderer(
  binding: TrustedGeneratedSurfaceBinding
): TrustedContributionRenderer {
  if (!binding.rendererBindingId || binding.semanticBindingIds.length === 0) {
    throw new Error('Generated surface binding requires exact renderer and semantic identities.');
  }
  return {
    rendererBindingId: binding.rendererBindingId,
    semanticBindingIds: binding.semanticBindingIds,
    contributionKinds: binding.contributionKinds ?? ['generated_surface'],
    component: GeneratedSurfaceContribution,
    componentProps: { snapshotResolver: binding.snapshotResolver }
  };
}
