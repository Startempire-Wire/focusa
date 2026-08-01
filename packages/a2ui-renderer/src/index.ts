import {
  MessageProcessor,
  type A2uiClientAction,
  type A2uiMessage,
} from "@a2ui/web_core/v0_9";
import {
  A2uiSurface,
  basicCatalog,
  type LitComponentApi,
} from "@a2ui/lit/v0_9";
import {
  FOCUSA_CATALOG_ID,
  focusaCatalog,
} from "./focusa-catalog.js";
export {
  semanticPairActions,
  semanticPairSurfaceModel,
  type SemanticPairSurfaceMode,
  type SemanticPairSurfaceModel,
} from "./semantic-pair-surface.js";

export const FOCUSA_A2UI_PROTOCOL = "v0.9" as const;
export const FOCUSA_A2UI_CATALOG_ID = FOCUSA_CATALOG_ID;

export interface RendererLimits {
  maxMessages: number;
  maxSerializedBytes: number;
}

export interface FocusaA2uiRendererOptions {
  onAction?: (action: A2uiClientAction) => void | Promise<void>;
  allowedActionNames?: ReadonlySet<string>;
  limits?: Partial<RendererLimits>;
}

const DEFAULT_LIMITS: RendererLimits = {
  maxMessages: 256,
  maxSerializedBytes: 1_000_000,
};

interface MountedSurface {
  container: Element;
  element: A2uiSurface;
}

/** Permanent Focusa generated-UI runtime: A2UI v0.9.1 web_core + Lit. */
export class FocusaA2uiRenderer {
  readonly processor: MessageProcessor<LitComponentApi>;
  readonly limits: RendererLimits;
  readonly #mounted = new Map<string, MountedSurface>();
  readonly #focusaSurfaces = new Set<string>();
  readonly #knownComponents = new Set(focusaCatalog.components.keys());
  readonly #onAction?: (action: A2uiClientAction) => void | Promise<void>;
  readonly #allowedActionNames: ReadonlySet<string>;

  constructor(options: FocusaA2uiRendererOptions = {}) {
    this.limits = { ...DEFAULT_LIMITS, ...options.limits };
    this.#onAction = options.onAction;
    this.#allowedActionNames = options.allowedActionNames ?? new Set();
    this.processor = new MessageProcessor(
      [basicCatalog, focusaCatalog],
      (action) => this.dispatchAction(action),
    );
  }

  processSnapshot(messages: A2uiMessage[]): void {
    if (!messages.some((message) => "createSurface" in message)) {
      throw new Error("A2UI snapshot must create at least one surface");
    }
    this.#process(messages);
  }

  processDelta(messages: A2uiMessage[]): void {
    this.#process(messages);
  }

  mount(container: Element, surfaceId: string): A2uiSurface {
    const surface = this.processor.model.getSurface(surfaceId);
    if (!surface) throw new Error(`Unknown A2UI surface: ${surfaceId}`);
    const prior = this.#mounted.get(surfaceId);
    prior?.element.remove();
    const element = document.createElement("a2ui-surface") as A2uiSurface;
    element.surface = surface;
    container.replaceChildren(element);
    this.#mounted.set(surfaceId, { container, element });
    return element;
  }

  async dispatchAction(action: A2uiClientAction): Promise<void> {
    const name = action.name;
    if (this.#allowedActionNames.has(name) && this.#onAction) {
      await this.#onAction(action);
      return;
    }
    this.#renderUnsupported(
      action.surfaceId,
      `Action ${name} is unavailable or outside the generated Operation Registry binding.`,
    );
  }

  surfaceIds(): string[] {
    return [...this.processor.model.surfacesMap.keys()].sort();
  }

  clientCapabilities(): ReturnType<MessageProcessor<LitComponentApi>["getClientCapabilities"]> {
    return this.processor.getClientCapabilities({ includeInlineCatalogs: false });
  }

  dispose(): void {
    for (const { container } of this.#mounted.values()) container.replaceChildren();
    this.#mounted.clear();
    this.#focusaSurfaces.clear();
    this.processor.model.dispose();
  }

  #process(messages: A2uiMessage[]): void {
    if (messages.length === 0 || messages.length > this.limits.maxMessages) {
      throw new Error(`A2UI message count must be 1-${this.limits.maxMessages}`);
    }
    const bytes = new TextEncoder().encode(JSON.stringify(messages)).byteLength;
    if (bytes > this.limits.maxSerializedBytes) {
      throw new Error(`A2UI payload exceeds ${this.limits.maxSerializedBytes} bytes`);
    }
    const normalized: A2uiMessage[] = [];
    for (const message of messages) {
      if (message.version !== FOCUSA_A2UI_PROTOCOL) {
        throw new Error(`Unsupported A2UI protocol version: ${String(message.version)}`);
      }
      if ("createSurface" in message && message.createSurface.catalogId === FOCUSA_CATALOG_ID) {
        this.#focusaSurfaces.add(message.createSurface.surfaceId);
      }
      if ("deleteSurface" in message) this.#focusaSurfaces.delete(message.deleteSurface.surfaceId);
      normalized.push(this.#withRecoveryFallback(message));
    }
    this.processor.processMessages(normalized);
  }

  #withRecoveryFallback(message: A2uiMessage): A2uiMessage {
    if (!("updateComponents" in message)) return message;
    const update = message.updateComponents;
    if (!this.#focusaSurfaces.has(update.surfaceId)) return message;
    const components = update.components.map((component) => {
      const candidate = component as unknown as Record<string, unknown>;
      const componentName = String(candidate.component ?? "");
      if (this.#knownComponents.has(componentName)) return component;
      return {
        id: String(candidate.id ?? "unsupported"),
        component: "FocusaRecoveryCard",
        label: "Unsupported generated component",
        description: `${componentName || "Unknown component"} is not in the trusted Focusa catalog.`,
        status: "recovery",
        details: "No action was executed. Refresh the surface or use the recovery action.",
      };
    });
    return { ...message, updateComponents: { ...update, components } } as A2uiMessage;
  }

  #renderUnsupported(surfaceId: string, description: string): void {
    const mounted = this.#mounted.get(surfaceId);
    if (!mounted) return;
    const recovery = document.createElement("focusa-recovery-card") as HTMLElement & {
      label: string;
      description: string;
      status: string;
      details: string;
    };
    recovery.label = "Unsupported action";
    recovery.description = description;
    recovery.status = "recovery";
    recovery.details = "No action was executed. Refresh permissions or regenerate the surface.";
    mounted.container.append(recovery);
  }
}

export type { A2uiClientAction, A2uiMessage } from "@a2ui/web_core/v0_9";
export { A2uiSurface, basicCatalog } from "@a2ui/lit/v0_9";
export {
  FOCUSA_CATALOG_ID,
  FocusaComponentSchema,
  focusaCatalog,
  focusaComponentNames,
} from "./focusa-catalog.js";
