import CanonicalReferenceContribution from './contributions/CanonicalReferenceContribution.svelte';
import PiSessionContribution from './contributions/PiSessionContribution.svelte';
import ProjectOverviewContribution from './contributions/ProjectOverviewContribution.svelte';
import ProjectionControlsContribution from './contributions/ProjectionControlsContribution.svelte';
import PromptEditorContribution from './contributions/PromptEditorContribution.svelte';
import SteeringQueueContribution from './contributions/SteeringQueueContribution.svelte';
import FollowUpQueueContribution from './contributions/FollowUpQueueContribution.svelte';
import WorkSurfaceContribution from './contributions/WorkSurfaceContribution.svelte';
import BrowserArtifactContribution from './contributions/BrowserArtifactContribution.svelte';
import SessionInventoryContribution from './contributions/SessionInventoryContribution.svelte';
import WorkRailContribution from './contributions/WorkRailContribution.svelte';
import CanonicalEventHistoryContribution from './contributions/CanonicalEventHistoryContribution.svelte';
import { ContributionRendererRegistry, type TrustedContributionRenderer } from './contribution-renderers';

export const DEFAULT_CONTRIBUTION_RENDERERS = [
  {
    rendererBindingId: 'renderer:pi-session@v1',
    semanticBindingIds: ['semantic:pi-session'],
    contributionKinds: ['focused_work_surface'],
    component: PiSessionContribution
  },
  {
    // Canonical project overview: the focused Work Surface for the software
    // profile's overview activity (emitted by the daemon's builtin catalog).
    rendererBindingId: 'renderer:project-overview',
    semanticBindingIds: ['semantic:project-overview'],
    contributionKinds: ['focused_work_surface'],
    component: ProjectOverviewContribution
  },
  {
    // Canonical toolbar controls contribution (e.g. profile/activity selectors)
    // emitted by the daemon's builtin catalog.
    rendererBindingId: 'renderer:controls',
    semanticBindingIds: ['semantic:controls'],
    contributionKinds: ['toolbar_control'],
    component: ProjectionControlsContribution
  },
  {
    rendererBindingId: 'renderer:document@v1',
    contributionKinds: ['focused_work_surface'],
    component: DomainContentContribution
  },
  {
    rendererBindingId: 'renderer:research@v1',
    contributionKinds: ['focused_work_surface'],
    component: DomainContentContribution
  },
  {
    rendererBindingId: 'renderer:evidence@v1',
    contributionKinds: ['focused_work_surface'],
    component: DomainContentContribution
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
    rendererBindingId: 'renderer:artifact:browser_snapshot@v1',
    contributionKinds: ['focused_work_surface', 'generated_surface'],
    component: BrowserArtifactContribution
  },
  {
    rendererBindingId: 'renderer:silent-sessions@v1',
    semanticBindingIds: ['semantic:silent-sessions'],
    contributionKinds: ['inspector', 'focused_work_surface', 'generated_surface'],
    component: SessionInventoryContribution
  },
  {
    rendererBindingId: 'renderer:focusa-inspector@v1',
    semanticBindingIds: ['semantic:focusa-inspector'],
    contributionKinds: ['inspector'],
    component: CanonicalReferenceContribution
  },
  {
    rendererBindingId: 'renderer:work-rail@v1',
    semanticBindingIds: ['semantic:work-rail'],
    contributionKinds: ['work_rail'],
    component: WorkRailContribution
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
    rendererBindingId: 'renderer:history@v1',
    semanticBindingIds: ['semantic:history'],
    component: CanonicalEventHistoryContribution
  },
  {
    rendererBindingId: 'renderer:prompt-editor@v1',
    semanticBindingIds: ['semantic:prompt-editor'],
    contributionKinds: ['prompt_editor'],
    component: PromptEditorContribution
  }
] as const satisfies readonly TrustedContributionRenderer[];

export const DEFAULT_CONTRIBUTION_REGISTRY = new ContributionRendererRegistry(DEFAULT_CONTRIBUTION_RENDERERS);
