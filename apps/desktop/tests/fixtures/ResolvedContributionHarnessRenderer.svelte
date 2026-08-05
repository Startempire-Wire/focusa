<script lang="ts">
  import type { ResolvedContribution } from '../../src/lib/mission-canvas/types';
  let { contribution }: { contribution: ResolvedContribution } = $props();
</script>

<article data-trusted-renderer={contribution.renderer_binding_id}>{contribution.accessibility.label}</article>
