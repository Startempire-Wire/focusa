import TrustedCustomElementContribution from './TrustedCustomElementContribution.svelte';
import type { TrustedContributionRenderer } from './contribution-renderers';
import type { ContributionKind } from './types';

export interface TrustedCustomElementBinding {
  rendererBindingId: string;
  semanticBindingIds?: readonly string[];
  contributionKinds?: readonly ContributionKind[];
  elementName: string;
}

export function trustedCustomElementRenderer(binding: TrustedCustomElementBinding): TrustedContributionRenderer {
  if (!/^[a-z][a-z0-9]*(?:-[a-z0-9]+)+$/.test(binding.elementName)) {
    throw new Error(`Invalid custom element name for ${binding.rendererBindingId}`);
  }
  return {
    rendererBindingId: binding.rendererBindingId,
    semanticBindingIds: binding.semanticBindingIds,
    contributionKinds: binding.contributionKinds,
    component: TrustedCustomElementContribution,
    componentProps: { elementName: binding.elementName }
  };
}
