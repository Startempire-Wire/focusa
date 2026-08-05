<script lang="ts">
  import type { ResolvedContribution } from './types';

  let {
    contribution,
    elementName
  }: {
    contribution: ResolvedContribution;
    elementName: string;
  } = $props();

  const trustedElementName = $derived(
    /^[a-z][a-z0-9]*(?:-[a-z0-9]+)+$/.test(elementName) ? elementName : undefined
  );

  type ContributionElement = HTMLElement & { contribution?: ResolvedContribution };

  function attachContribution(node: ContributionElement, value: ResolvedContribution) {
    node.contribution = value;
    return {
      update(next: ResolvedContribution) {
        node.contribution = next;
      },
      destroy() {
        delete node.contribution;
      }
    };
  }
</script>

{#if trustedElementName}
  <svelte:element
    this={trustedElementName}
    use:attachContribution={contribution}
    data-contribution-id={contribution.contribution_id}
    data-renderer-binding-id={contribution.renderer_binding_id}
  ></svelte:element>
{:else}
  <section class="invalid-element" role="alert" data-renderer-binding-id={contribution.renderer_binding_id}>
    <strong>{contribution.accessibility.label}</strong>
    <span>Trusted renderer element unavailable.</span>
  </section>
{/if}

<style>
  .invalid-element{display:grid;gap:var(--space-2);padding:var(--layout-card-padding);border:1px solid var(--color-warning);border-radius:var(--radius-card);background:var(--color-raised);color:var(--color-text)}
  .invalid-element span{color:var(--color-text-secondary)}
</style>
