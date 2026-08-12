<script lang="ts">
  import type { MissionCanvasClient } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
  import PromptEditorContribution from '$lib/mission-canvas/contributions/PromptEditorContribution.svelte';
  import type { AttachmentKey, ResolvedContribution, ResolvedWorkspaceProjection, WorkstreamAuthorityContext } from '$lib/mission-canvas/types';

  let {
    authority,
    client
  }: {
    authority: WorkstreamAuthorityContext;
    client?: MissionCanvasClient;
  } = $props();

  // The recipient is resolved from the authority's WorkSurfaceId.
  // PromptEditorContribution routes drafts to projection.focused_work_surface_id.
  // Follows Spec 135J: recipient must be exact, never a surface/CWD guess.
  const resolvedRecipient = $derived(authority.work_surface_id ?? authority.attachment?.workspace_binding_id ?? null);

  // Minimal projection envelope constructed from authority.
  const projection = $derived({
    workstream: authority.workstream,
    continuity_id: authority.continuity_id,
    attachment: authority.attachment ?? null,
    workspace_profile_id: authority.workspace_profile_id ?? 'software',
    activity_mode_id: authority.activity_mode_id ?? 'overview',
    projection_revision: 0,
    layout_revision: 0,
    eligible_contributions: [] as ResolvedContribution[],
    operation_bindings: [] as any[],
    activity_mode_revision: 0,
    candidate_contribution_ids: [] as string[],
    canonical_read_model_revision: 0,
    durable_event_cursor: '',
    evidence_refs: [] as string[],
    focused_semantic_target: null as string | null,
    focused_work_surface_id: resolvedRecipient,
    layout_tree: null as any,
    omission_diagnostics: [] as any[],
    projection_digest: '',
    receipt_refs: [] as string[],
    resolved_at: new Date().toISOString(),
    runtime_object: authority.runtime_object ?? null,
    schema: 'focusa.resolved_workspace_projection.v1' as const,
    work_surface_id: authority.work_surface_id ?? null
  }) satisfies ResolvedWorkspaceProjection;

  // Synthetic contribution for the global prompt editor lane.
  const syntheticContribution = $derived({
    contribution_id: 'contribution:prompt-editor',
    renderer_binding_id: 'renderer:prompt-editor@v1',
    kind: 'prompt_editor' as const,
    accessibility: {
      label: 'Prompt Editor',
      description: 'Compose and route prompts through the canonical Workstream',
      role: 'region' as const
    },
    freshness: { status: 'current' as const, observed_at: new Date().toISOString() },
    data_ref: { kind: 'prompt_editor' as const, ref: 'surface:prompt-editor', revision: 1, freshness: 'current' as const },
    operation_ids: [] as string[],
    candidate_contribution_ids: [] as string[],
    data: null
  }) satisfies ResolvedContribution;
</script>

<PromptEditorContribution
  contribution={syntheticContribution}
  {projection}
  {client}
/>
