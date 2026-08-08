const ALLOWED_BROWSER_ARTIFACT_KEYS = new Set([
  'schema',
  'artifact_id',
  'artifact_kind',
  'title',
  'before_ref',
  'after_ref',
  'evidence_refs',
  'summary',
  'changes',
  'citations',
  'provenance',
  'project_root',
  'continuity_id',
  'session_origin',
  'attachment_id',
  'freshness',
  'authority',
  'vertical_dispatch',
  'render_safe',
  'redacted',
  'artifact_handle',
  'external_open_ref'
]);

const ALLOWED_BROWSER_ARTIFACT_PROVENANCE_KEYS = new Set([
  'source_kind',
  'harvested_at',
  'uiai_session_ref',
  'browser_context_ref',
  'operator_id'
]);

const ALLOWED_UIAI_SESSION_KEYS = new Set([
  'session_origin',
  'uiai_session_ref',
  'browser_context_id',
  'browser_context_ref',
  'browser_target_id',
  'browser_target_ref',
  'continuity_id',
  'attachment_id',
  'continuity_ref',
  'session_ref'
]);

interface BrowserArtifactCitation {
  citation_ref: string;
  source_origin: string;
  authoritative?: boolean;
}

interface BrowserArtifactProvenance {
  source_kind: string;
  harvested_at: string;
  uiai_session_ref?: string;
  browser_context_ref?: string;
  operator_id?: string;
}

export interface BrowserArtifactDescriptor {
  schema: 'focusa.workspace_artifact_descriptor.v1';
  artifact_id: string;
  artifact_kind: 'browser_snapshot';
  title: string;
  before_ref: string;
  after_ref: string;
  evidence_refs: string[];
  summary?: string;
  changes?: string[];
  citations?: BrowserArtifactCitation[];
  provenance?: BrowserArtifactProvenance;
  project_root: string;
  continuity_id: string;
  session_origin: string;
  attachment_id?: string;
  freshness: string;
  authority: string;
  vertical_dispatch?: string;
  render_safe: boolean;
  redacted?: boolean;
  artifact_handle?: string;
  external_open_ref?: string;
}

export interface UIAISessionRef {
  session_origin: string;
  uiai_session_ref?: string;
  browser_context_id?: string;
  browser_context_ref?: string;
  browser_target_id?: string;
  browser_target_ref?: string;
  continuity_id?: string;
  attachment_id?: string;
  continuity_ref?: string;
  session_ref?: string;
}

function asRecord(value: unknown): Record<string, unknown> | undefined {
  return typeof value === 'object' && value !== null ? value as Record<string, unknown> : undefined;
}

function isNonEmptyString(value: unknown): value is string {
  return typeof value === 'string' && value.trim().length > 0;
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => isNonEmptyString(item));
}

function hasOnlyKnownKeys(value: Record<string, unknown>, allowed: Set<string>): boolean {
  return Object.keys(value).every((key) => allowed.has(key));
}

function parseCitation(value: unknown): BrowserArtifactCitation | undefined {
  const candidate = asRecord(value);
  if (!candidate) return undefined;
  if (!hasOnlyKnownKeys(candidate, new Set(['citation_ref', 'source_origin', 'authoritative']))) return undefined;
  if (!isNonEmptyString(candidate.citation_ref) || !isNonEmptyString(candidate.source_origin)) return undefined;
  const result: BrowserArtifactCitation = {
    citation_ref: candidate.citation_ref.trim(),
    source_origin: candidate.source_origin.trim()
  };
  if (candidate.authoritative !== undefined && typeof candidate.authoritative !== 'boolean') return undefined;
  if (candidate.authoritative !== undefined) {
    result.authoritative = candidate.authoritative;
  }
  return result;
}

function normalizeProvenance(value: unknown): BrowserArtifactProvenance | undefined {
  const provenance = asRecord(value);
  if (!provenance) return undefined;
  if (!hasOnlyKnownKeys(provenance, ALLOWED_BROWSER_ARTIFACT_PROVENANCE_KEYS)) return undefined;
  if (!isNonEmptyString(provenance.source_kind) || !isNonEmptyString(provenance.harvested_at)) return undefined;
  const normalized: BrowserArtifactProvenance = {
    source_kind: provenance.source_kind,
    harvested_at: provenance.harvested_at
  };
  if (isNonEmptyString(provenance.uiai_session_ref)) {
    normalized.uiai_session_ref = provenance.uiai_session_ref;
  }
  if (isNonEmptyString(provenance.browser_context_ref)) {
    normalized.browser_context_ref = provenance.browser_context_ref;
  }
  if (isNonEmptyString(provenance.operator_id)) {
    normalized.operator_id = provenance.operator_id;
  }
  return normalized;
}

function extractCitations(value: unknown): BrowserArtifactCitation[] {
  if (!Array.isArray(value)) return [];
  const citations = [] as BrowserArtifactCitation[];
  for (const item of value) {
    const citation = parseCitation(item);
    if (citation) citations.push(citation);
  }
  return citations;
}

export const UIAISessionRef = {
  validate(value: unknown): value is UIAISessionRef {
    const record = asRecord(value);
    if (!record) return false;
    if (!hasOnlyKnownKeys(record, ALLOWED_UIAI_SESSION_KEYS)) return false;
    if (!isNonEmptyString(record.session_origin)) return false;
    if (record.uiai_session_ref !== undefined && !isNonEmptyString(record.uiai_session_ref)) return false;
    if (record.browser_context_id !== undefined && !isNonEmptyString(record.browser_context_id)) return false;
    if (record.browser_context_ref !== undefined && !isNonEmptyString(record.browser_context_ref)) return false;
    if (record.browser_target_id !== undefined && !isNonEmptyString(record.browser_target_id)) return false;
    if (record.browser_target_ref !== undefined && !isNonEmptyString(record.browser_target_ref)) return false;
    if (record.continuity_id !== undefined && !isNonEmptyString(record.continuity_id)) return false;
    if (record.attachment_id !== undefined && !isNonEmptyString(record.attachment_id)) return false;
    if (record.continuity_ref !== undefined && !isNonEmptyString(record.continuity_ref)) return false;
    if (record.session_ref !== undefined && !isNonEmptyString(record.session_ref)) return false;
    return true;
  }
};

export const BrowserArtifactRef = {
  validate(value: unknown): value is BrowserArtifactDescriptor {
    const record = asRecord(value);
    if (!record) return false;
    if (!hasOnlyKnownKeys(record, ALLOWED_BROWSER_ARTIFACT_KEYS)) return false;
    if (record.schema !== 'focusa.workspace_artifact_descriptor.v1') return false;
    if (!isNonEmptyString(record.artifact_id)
      || !isNonEmptyString(record.artifact_kind)
      || !isNonEmptyString(record.title)
      || !isNonEmptyString(record.before_ref)
      || !isNonEmptyString(record.after_ref)
      || !isNonEmptyString(record.project_root)
      || !isNonEmptyString(record.continuity_id)
      || !isNonEmptyString(record.session_origin)
      || !isNonEmptyString(record.freshness)
      || !isNonEmptyString(record.authority)
      || typeof record.render_safe !== 'boolean') {
      return false;
    }
    if (record.artifact_kind !== 'browser_snapshot') return false;
    if (!isStringArray(record.evidence_refs)) return false;
    if (record.attachment_id !== undefined && !isNonEmptyString(record.attachment_id)) return false;
    if (record.vertical_dispatch !== undefined && !isNonEmptyString(record.vertical_dispatch)) return false;
    if (record.summary !== undefined && typeof record.summary !== 'string') return false;
    if (record.redacted !== undefined && typeof record.redacted !== 'boolean') return false;
    if (record.artifact_handle !== undefined && !isNonEmptyString(record.artifact_handle)) return false;
    if (record.external_open_ref !== undefined && !isNonEmptyString(record.external_open_ref)) return false;

    if (record.provenance !== undefined) {
      const provenance = normalizeProvenance(record.provenance);
      if (!provenance) return false;
      record.provenance = provenance;
    }

    if (record.citations !== undefined && !isStringArray(record.citations)) {
      const citations = extractCitations(record.citations);
      if (citations.length !== (record.citations as unknown[]).length) return false;
    }

    if (record.changes !== undefined && !isStringArray(record.changes)) return false;
    return true;
  }
};

export const ArtifactRenderer = {
  render(descriptor: BrowserArtifactDescriptor): string[] {
    const citations = descriptor.citations?.length
      ? descriptor.citations
          .map((citation) => `${citation.citation_ref}@${citation.source_origin}`)
          .join(', ')
      : 'none';
    const changes = descriptor.changes?.length
      ? descriptor.changes.map((change) => `- ${change}`)
      : ['- no structured changes'];
    const provenance = descriptor.provenance ? `${descriptor.provenance.source_kind}@${descriptor.provenance.harvested_at}` : 'unknown';
    const sessionContext = browserSessionContextFromDescriptor(descriptor);

    const lines = [
      `## Browser snapshot artifact · ${descriptor.title}`,
      `Artifact: ${descriptor.artifact_id}`,
      `Kind: ${descriptor.artifact_kind}`,
      `Session origin: ${descriptor.session_origin}`,
      `UIAI session ref: ${sessionContext.uiai_session_ref || sessionContext.session_origin}`,
      `Browser context: ${sessionContext.browser_context_ref || sessionContext.browser_context_id || 'not-available'}`,
      `Project: ${descriptor.project_root}`,
      `Continuity: ${descriptor.continuity_id}`,
      `Evidence refs: ${descriptor.evidence_refs.length ? descriptor.evidence_refs.join(', ') : 'none'}`,
      `Before ref: ${descriptor.before_ref}`,
      `After ref: ${descriptor.after_ref}`,
      `Freshness: ${descriptor.freshness}`,
      `Authority: ${descriptor.authority}`,
      `Provenance: ${provenance}`
    ];

    lines.push(`Citations: ${citations}`);
    lines.push(`Render-safe: ${descriptor.render_safe}`);
    lines.push(...changes);
    lines.push('Execution control: Desktop is metadata-only; browser actions remain in UIAI Engine.');

    if (!descriptor.render_safe) {
      lines.push('RENDER_BLOCKED: render_safe is false; fallback required');
    }

    return lines;
  }
};

export function parseBrowserArtifactDescriptor(raw: unknown): BrowserArtifactDescriptor | null {
  if (!isNonEmptyString(raw)) return null;
  try {
    const parsed = JSON.parse(raw) as unknown;
    return BrowserArtifactRef.validate(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

export function browserSessionContextFromDescriptor(descriptor: BrowserArtifactDescriptor): UIAISessionRef {
  const context = asRecord(descriptor.provenance) ?? {};
  const candidate = {
    session_origin: descriptor.session_origin,
    uiai_session_ref: isNonEmptyString(context.uiai_session_ref) ? context.uiai_session_ref : descriptor.session_origin,
    browser_context_ref: isNonEmptyString(context.browser_context_ref) ? context.browser_context_ref : undefined,
    browser_target_ref: descriptor.external_open_ref,
    continuity_id: descriptor.continuity_id,
    attachment_id: descriptor.attachment_id
  } satisfies Record<string, unknown>;

  if (UIAISessionRef.validate(candidate)) {
    return {
      session_origin: descriptor.session_origin,
      uiai_session_ref: isNonEmptyString(candidate.uiai_session_ref) ? candidate.uiai_session_ref : descriptor.session_origin,
      browser_context_ref: isNonEmptyString(candidate.browser_context_ref) ? candidate.browser_context_ref : undefined,
      browser_target_ref: isNonEmptyString(candidate.browser_target_ref) ? candidate.browser_target_ref : undefined,
      continuity_id: isNonEmptyString(candidate.continuity_id) ? candidate.continuity_id : undefined,
      attachment_id: isNonEmptyString(candidate.attachment_id) ? candidate.attachment_id : undefined
    };
  }

  return {
    session_origin: descriptor.session_origin
  };
}

export function sessionContextLine(descriptor: BrowserArtifactDescriptor): string {
  const context = browserSessionContextFromDescriptor(descriptor);
  const session = context.browser_context_ref || context.browser_context_id || 'not-available';
  return `Browser context id: ${session}`;
}
