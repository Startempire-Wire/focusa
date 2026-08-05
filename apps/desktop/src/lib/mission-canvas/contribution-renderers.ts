import type { Component } from 'svelte';
import type { ResolvedContribution } from './types';

export interface ContributionRendererProps {
  contribution: ResolvedContribution;
}

export interface TrustedContributionRenderer {
  rendererBindingId: string;
  semanticBindingIds?: readonly string[];
  component: Component<ContributionRendererProps>;
}

export class ContributionRendererRegistry {
  readonly #entries: ReadonlyMap<string, TrustedContributionRenderer>;

  constructor(entries: readonly TrustedContributionRenderer[]) {
    const indexed = new Map<string, TrustedContributionRenderer>();
    for (const entry of entries) {
      if (!entry.rendererBindingId || indexed.has(entry.rendererBindingId)) {
        throw new Error(`Invalid or duplicate renderer binding: ${entry.rendererBindingId}`);
      }
      indexed.set(entry.rendererBindingId, entry);
    }
    this.#entries = indexed;
  }

  resolve(contribution: ResolvedContribution): Component<ContributionRendererProps> | undefined {
    const entry = this.#entries.get(contribution.renderer_binding_id);
    if (!entry) return undefined;
    if (entry.semanticBindingIds && !entry.semanticBindingIds.includes(contribution.semantic_binding_id)) {
      return undefined;
    }
    return entry.component;
  }

  has(rendererBindingId: string): boolean {
    return this.#entries.has(rendererBindingId);
  }
}
