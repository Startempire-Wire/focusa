import CanonicalReferenceContribution from './contributions/CanonicalReferenceContribution.svelte';
import PromptEditorContribution from './contributions/PromptEditorContribution.svelte';
import SteeringQueueContribution from './contributions/SteeringQueueContribution.svelte';
import FollowUpQueueContribution from './contributions/FollowUpQueueContribution.svelte';
import WorkSurfaceContribution from './contributions/WorkSurfaceContribution.svelte';
import { ContributionRendererRegistry, type TrustedContributionRenderer } from './contribution-renderers';

export const DEFAULT_CONTRIBUTION_RENDERERS = [
  {
    rendererBindingId: 'renderer:pi-session@v1',
    semanticBindingIds: ['semantic:pi-session'],
    contributionKinds: ['focused_work_surface'],
    component: WorkSurfaceContribution
  },
  {
    rendererBindingId: 'renderer:document@v1',
    contributionKinds: ['focused_work_surface'],
    component: WorkSurfaceContribution
  },
  {
    rendererBindingId: 'renderer:research@v1',
    contributionKinds: ['focused_work_surface'],
    component: WorkSurfaceContribution
  },
  {
    rendererBindingId: 'renderer:evidence@v1',
    contributionKinds: ['focused_work_surface'],
    component: WorkSurfaceContribution
  },
  {
    rendererBindingId: 'renderer:artifact:json@v1',
    contributionKinds: ['focused_work_surface', 'generated_surface'],
    component: WorkSurfaceContribution
  },
  {
    rendererBindingId: 'renderer:artifact:markdown@v1',
    contributionKinds: ['focused_work_surface', 'generated_surface'],
    component: WorkSurfaceContribution
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
    component: SteeringQueueContribution
  },
  {
    rendererBindingId: 'renderer:follow-up-queue@v1',
    semanticBindingIds: ['semantic:follow-up-queue'],
    contributionKinds: ['follow_up_queue'],
    component: FollowUpQueueContribution
  },
  {
    rendererBindingId: 'renderer:prompt-editor@v1',
    semanticBindingIds: ['semantic:prompt-editor'],
    contributionKinds: ['prompt_editor'],
    component: PromptEditorContribution
  }
] as const satisfies readonly TrustedContributionRenderer[];

export const DEFAULT_CONTRIBUTION_REGISTRY = new ContributionRendererRegistry(DEFAULT_CONTRIBUTION_RENDERERS);
