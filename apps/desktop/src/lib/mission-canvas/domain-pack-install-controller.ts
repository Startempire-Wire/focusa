import type { MissionCanvasClient, MissionCanvasOperationInput } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
import { sameWorkstreamKey, validateMissionCanvasContract } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import type { DomainPackInstallReceipt, WorkstreamAuthorityContext } from './types';
import { MissionCanvasTransportError } from './http-transport';

/**
 * Transport payload owned by the generated Mission Canvas operation.  The
 * domain-pack catalog is intentionally opaque to Desktop; eligibility and
 * registry semantics remain core-owned.
 */
export type DomainPackInstallPayload = Record<string, unknown>;

export interface DomainPackInstallGovernance {
  actorId: string;
  authorityRef: string;
}

export type DomainPackInstallState =
  | { kind: 'idle' }
  | { kind: 'awaiting_confirmation'; authority: WorkstreamAuthorityContext; pack: DomainPackInstallPayload; idempotencyKey: string; governance: DomainPackInstallGovernance }
  | { kind: 'installing'; authority: WorkstreamAuthorityContext; pack: DomainPackInstallPayload; idempotencyKey: string; governance: DomainPackInstallGovernance }
  | { kind: 'installed'; authority: WorkstreamAuthorityContext; receipt: DomainPackInstallReceipt }
  | { kind: 'stale'; authority: WorkstreamAuthorityContext; pack: DomainPackInstallPayload; idempotencyKey: string; governance: DomainPackInstallGovernance; reason: string }
  | { kind: 'blocked'; authority?: WorkstreamAuthorityContext; reason: string }
  | { kind: 'error'; authority?: WorkstreamAuthorityContext; reason: string };

/**
 * Desktop management-flow consumer for the generated operation.  Calling
 * `begin` never mutates; only `confirm` sends `domain_packInstall` with the
 * explicit generated confirmation and the exact Workstream authority.
 */
export class MissionCanvasDomainPackInstallController {
  state: DomainPackInstallState = { kind: 'idle' };
  #generation = 0;

  constructor(private readonly client: Pick<MissionCanvasClient, 'domain_packInstall'>) {}

  begin(
    authority: WorkstreamAuthorityContext,
    pack: DomainPackInstallPayload,
    governance: DomainPackInstallGovernance,
    idempotencyKey = crypto.randomUUID()
  ): boolean {
    const validation = validateMissionCanvasContract('WorkstreamAuthorityContext', authority);
    if (!validation.valid) {
      this.state = { kind: 'blocked', authority, reason: validation.errors.join(',') };
      return false;
    }
    if (!idempotencyKey.trim()) {
      this.state = { kind: 'blocked', authority, reason: 'idempotency_key_required' };
      return false;
    }
    if (!governance.actorId.trim() || !governance.authorityRef.trim()) {
      this.state = { kind: 'blocked', authority, reason: 'authority_required' };
      return false;
    }
    this.#generation += 1;
    this.state = {
      kind: 'awaiting_confirmation',
      authority,
      pack,
      idempotencyKey,
      governance
    };
    return true;
  }

  async confirm(): Promise<void> {
    if (this.state.kind !== 'awaiting_confirmation' && this.state.kind !== 'stale') return;
    const pending = this.state;
    const generation = ++this.#generation;
    this.state = {
      kind: 'installing',
      authority: pending.authority,
      pack: pending.pack,
      idempotencyKey: pending.idempotencyKey,
      governance: pending.governance
    };
    try {
      const value = await this.client.domain_packInstall({
        ...pending.authority,
        pack: pending.pack,
        idempotency_key: pending.idempotencyKey,
        confirmation: 'confirm',
        actor_id: pending.governance.actorId,
        authority_ref: pending.governance.authorityRef
      } as MissionCanvasOperationInput);
      if (generation !== this.#generation) return;
      const validation = validateMissionCanvasContract('DomainPackInstallReceipt', value);
      if (!validation.valid) {
        this.state = { kind: 'error', authority: pending.authority, reason: validation.errors.join(',') };
        return;
      }
      const receipt = value as DomainPackInstallReceipt;
      if (!sameWorkstreamKey(receipt.workstream, pending.authority.workstream)) {
        this.state = { kind: 'blocked', authority: pending.authority, reason: 'foreign_receipt_scope' };
        return;
      }
      this.state = { kind: 'installed', authority: pending.authority, receipt };
    } catch (error) {
      if (generation !== this.#generation) return;
      const reason = error instanceof Error ? error.message : 'domain_pack_install_failed';
      this.state = error instanceof MissionCanvasTransportError && error.status === 409
        ? { kind: 'stale', authority: pending.authority, pack: pending.pack, idempotencyKey: pending.idempotencyKey, governance: pending.governance, reason }
        : { kind: 'error', authority: pending.authority, reason };
    }
  }

  cancel(): void {
    this.#generation += 1;
    this.state = { kind: 'idle' };
  }

  clear(): void {
    this.cancel();
  }
}
