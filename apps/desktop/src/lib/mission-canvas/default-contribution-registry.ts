import CanonicalReferenceContribution from './contributions/CanonicalReferenceContribution.svelte';
import PromptEditorContribution from './contributions/PromptEditorContribution.svelte';
import { ContributionRendererRegistry, type TrustedContributionRenderer } from './contribution-renderers';

export const DEFAULT_CONTRIBUTION_RENDERERS = [
  {
    rendererBindingId: 'renderer:pi-session@v1',
    semanticBindingIds: ['semantic:pi-session'],
    contributionKinds: ['focused_work_surface'],
    component: CanonicalReferenceContribution
  },
  {
    rendererBindingId: 'renderer:document@v1',
    contributionKinds: ['focused_work_surface'],
    component: CanonicalReferenceContribution
  },
  {
    rendererBindingId: 'renderer:research@v1',
    contributionKinds: ['focused_work_surface'],
    component: CanonicalReferenceContribution
  },
  {
    rendererBindingId: 'renderer:evidence@v1',
    contributionKinds: ['focused_work_surface'],
    component: CanonicalReferenceContribution
  },
  {
    rendererBindingId: 'renderer:artifact:json@v1',
    contributionKinds: ['focused_work_surface', 'generated_surface'],
    component: CanonicalReferenceContribution
  },
  {
    rendererBindingId: 'renderer:artifact:markdown@v1',
    contributionKinds: ['focused_work_surface', 'generated_surface'],
    component: CanonicalReferenceContribution
  },
  {
    rendererBindingId: 'renderer:focusa-inspector@v1',
    semanticBindingIds: ['semantic:focusa-inspector'],
    contributionKinds: ['inspector'],
    component: CanonicalReferenceContribution
  },
  {
    rendererBindingId: 'renderer:steering-queue@v1',
    semanticBindingIds: ['semantic:steering-queue'],
    contributionKinds: ['steering_queue'],
    component: CanonicalReferenceContribution
  },
  {
    rendererBindingId: 'renderer:prompt-editor@v1',
    semanticBindingIds: ['semantic:prompt-editor'],
    contributionKinds: ['prompt_editor'],
    component: PromptEditorContribution
  }
] as const satisfies readonly TrustedContributionRenderer[];

export const DEFAULT_CONTRIBUTION_REGISTRY = new ContributionRendererRegistry(DEFAULT_CONTRIBUTION_RENDERERS);
