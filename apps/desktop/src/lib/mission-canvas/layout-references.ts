import type { LayoutNode } from './types';

export function collectLayoutContributionIds(node: LayoutNode, target = new Set<string>()): ReadonlySet<string> {
  switch (node.kind) {
    case 'single':
      target.add(node.contribution_id);
      break;
    case 'split':
    case 'stack':
    case 'grid':
      for (const child of node.children) collectLayoutContributionIds(child, target);
      break;
    case 'tabs':
      for (const contributionId of node.contribution_ids) target.add(contributionId);
      break;
    case 'inspector':
      collectLayoutContributionIds(node.primary, target);
      for (const contributionId of node.inspector_contribution_ids) target.add(contributionId);
      break;
  }
  return target;
}
