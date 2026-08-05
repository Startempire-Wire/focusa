import TrustedCustomElementContribution from './TrustedCustomElementContribution.svelte';
import type { TrustedContributionRenderer } from './contribution-renderers';

export interface TrustedCustomElementBinding {
  rendererBindingId: string;
  semanticBindingIds?: readonly string[];
  elementName: string;
}

export function trustedCustomElementRenderer(binding: TrustedCustomElementBinding): TrustedContributionRenderer {
  if (!/^[a-z][a-z0-9]*(?:-[a-z0-9]+)+$/.test(binding.elementName)) {
    throw new Error(`Invalid custom element name for ${binding.rendererBindingId}`);
  }
  return {
    rendererBindingId: binding.rendererBindingId,
    semanticBindingIds: binding.semanticBindingIds,
    component: TrustedCustomElementContribution,
    componentProps: { elementName: binding.elementName }
  };
}
