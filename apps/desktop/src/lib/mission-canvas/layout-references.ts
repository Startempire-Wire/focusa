import type { LayoutNode } from './types';

export interface LayoutIntegrityIssue {
  code: 'empty_container' | 'invalid_active_tab' | 'duplicate_contribution' | 'duplicate_node' | 'invalid_grid_columns';
  nodeId: string;
  contributionId?: string;
}

export function validateLayoutIntegrity(node: LayoutNode): readonly LayoutIntegrityIssue[] {
  const issues: LayoutIntegrityIssue[] = [];
  const nodeIds = new Set<string>();
  const contributionIds = new Set<string>();

  function addContribution(nodeId: string, contributionId: string): void {
    if (contributionIds.has(contributionId)) {
      issues.push({ code: 'duplicate_contribution', nodeId, contributionId });
    } else {
      contributionIds.add(contributionId);
    }
  }

  function visit(current: LayoutNode): void {
    if (nodeIds.has(current.node_id)) issues.push({ code: 'duplicate_node', nodeId: current.node_id });
    else nodeIds.add(current.node_id);

    switch (current.kind) {
      case 'single':
        addContribution(current.node_id, current.contribution_id);
        break;
      case 'split':
      case 'stack':
        if (current.children.length === 0) issues.push({ code: 'empty_container', nodeId: current.node_id });
        for (const child of current.children) visit(child);
        break;
      case 'grid':
        if (current.children.length === 0) issues.push({ code: 'empty_container', nodeId: current.node_id });
        if (current.columns < 1) issues.push({ code: 'invalid_grid_columns', nodeId: current.node_id });
        for (const child of current.children) visit(child);
        break;
      case 'tabs':
        if (current.contribution_ids.length === 0) issues.push({ code: 'empty_container', nodeId: current.node_id });
        if (!current.contribution_ids.includes(current.active_contribution_id)) {
          issues.push({ code: 'invalid_active_tab', nodeId: current.node_id, contributionId: current.active_contribution_id });
        }
        for (const contributionId of current.contribution_ids) addContribution(current.node_id, contributionId);
        break;
      case 'inspector':
        if (current.inspector_contribution_ids.length === 0) issues.push({ code: 'empty_container', nodeId: current.node_id });
        visit(current.primary);
        for (const contributionId of current.inspector_contribution_ids) addContribution(current.node_id, contributionId);
        break;
    }
  }

  visit(node);
  return issues;
}

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
