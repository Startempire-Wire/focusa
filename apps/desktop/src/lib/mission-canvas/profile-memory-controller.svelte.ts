import type { ExactScope, ProfileLayoutMemory } from './types';

export interface ProfileMemoryBinding {
  scope: ExactScope;
  profileId: string;
  activityModeId: string;
  viewportClass: ProfileLayoutMemory['viewport_class'];
}

export interface ProfileMemoryTransport {
  get(binding: ProfileMemoryBinding): Promise<ProfileLayoutMemory>;
  update(memory: ProfileLayoutMemory): Promise<ProfileLayoutMemory>;
}

export type ProfileMemoryState =
  | { kind: 'unbound' }
  | { kind: 'loading'; binding: ProfileMemoryBinding }
  | { kind: 'ready'; binding: ProfileMemoryBinding; memory: ProfileLayoutMemory }
  | { kind: 'saving'; binding: ProfileMemoryBinding; memory: ProfileLayoutMemory }
  | { kind: 'conflict'; binding: ProfileMemoryBinding; memory: ProfileLayoutMemory; reason: string }
  | { kind: 'error'; binding?: ProfileMemoryBinding; reason: string };

function sameScope(left: ExactScope, right: ExactScope): boolean {
  return left.project_root === right.project_root
    && left.continuity_id === right.continuity_id
    && left.attachment_id === right.attachment_id
    && left.session_id === right.session_id
    && (left.instance_id ?? null) === (right.instance_id ?? null)
    && (left.working_subpath_id ?? null) === (right.working_subpath_id ?? null);
}

function matches(memory: ProfileLayoutMemory, binding: ProfileMemoryBinding): boolean {
  return sameScope(memory.scope, binding.scope)
    && memory.profile_id === binding.profileId
    && memory.activity_mode_id === binding.activityModeId
    && memory.viewport_class === binding.viewportClass;
}

export class MissionCanvasProfileMemoryController {
  state = $state<ProfileMemoryState>({ kind: 'unbound' });
  #generation = 0;

  constructor(private readonly transport: ProfileMemoryTransport) {}

  async load(binding: ProfileMemoryBinding): Promise<void> {
    const generation = ++this.#generation;
    this.state = { kind: 'loading', binding };
    try {
      const memory = await this.transport.get(binding);
      if (generation !== this.#generation) return;
      this.state = matches(memory, binding)
        ? { kind: 'ready', binding, memory }
        : { kind: 'error', binding, reason: 'foreign_profile_memory' };
    } catch (error) {
      if (generation === this.#generation) this.state = { kind: 'error', binding, reason: message(error) };
    }
  }

  async update(memory: ProfileLayoutMemory): Promise<void> {
    if (this.state.kind !== 'ready' && this.state.kind !== 'conflict') return;
    const { binding, memory: prior } = this.state;
    if (!matches(memory, binding)) {
      this.state = { kind: 'conflict', binding, memory: prior, reason: 'foreign_profile_memory' };
      return;
    }

    const generation = ++this.#generation;
    this.state = { kind: 'saving', binding, memory };
    try {
      const next = await this.transport.update(memory);
      if (generation !== this.#generation) return;
      if (!matches(next, binding)) {
        this.state = { kind: 'conflict', binding, memory, reason: 'foreign_profile_memory' };
      } else if (next.memory_revision < prior.memory_revision) {
        this.state = { kind: 'conflict', binding, memory, reason: 'profile_memory_revision_regressed' };
      } else {
        this.state = { kind: 'ready', binding, memory: next };
      }
    } catch (error) {
      if (generation === this.#generation) this.state = { kind: 'conflict', binding, memory, reason: message(error) };
    }
  }

  clear(): void {
    this.#generation += 1;
    this.state = { kind: 'unbound' };
  }
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : 'profile_memory_operation_failed';
}
