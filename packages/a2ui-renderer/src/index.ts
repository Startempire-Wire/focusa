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

export const FOCUSA_A2UI_PROTOCOL = "v0.9" as const;
export const FOCUSA_A2UI_CATALOG_ID = basicCatalog.id;

export interface RendererLimits {
  maxMessages: number;
  maxSerializedBytes: number;
}

export interface FocusaA2uiRendererOptions {
  onAction?: (action: A2uiClientAction) => void;
  limits?: Partial<RendererLimits>;
}

const DEFAULT_LIMITS: RendererLimits = {
  maxMessages: 256,
  maxSerializedBytes: 1_000_000,
};

/** Permanent Focusa generated-UI runtime: A2UI v0.9.1 web_core + Lit. */
export class FocusaA2uiRenderer {
  readonly processor: MessageProcessor<LitComponentApi>;
  readonly limits: RendererLimits;
  readonly #mounted = new Set<A2uiSurface>();

  constructor(options: FocusaA2uiRendererOptions = {}) {
    this.limits = { ...DEFAULT_LIMITS, ...options.limits };
    this.processor = new MessageProcessor(
      [basicCatalog],
      options.onAction,
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
    const element = document.createElement("a2ui-surface") as A2uiSurface;
    element.surface = surface;
    container.replaceChildren(element);
    this.#mounted.add(element);
    return element;
  }

  surfaceIds(): string[] {
    return [...this.processor.model.surfacesMap.keys()].sort();
  }

  clientCapabilities(): ReturnType<MessageProcessor<LitComponentApi>["getClientCapabilities"]> {
    return this.processor.getClientCapabilities({ includeInlineCatalogs: false });
  }

  dispose(): void {
    for (const element of this.#mounted) element.remove();
    this.#mounted.clear();
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
    for (const message of messages) {
      if (message.version !== FOCUSA_A2UI_PROTOCOL) {
        throw new Error(`Unsupported A2UI protocol version: ${String(message.version)}`);
      }
    }
    this.processor.processMessages(messages);
  }
}

export type { A2uiClientAction, A2uiMessage } from "@a2ui/web_core/v0_9";
export { A2uiSurface, basicCatalog } from "@a2ui/lit/v0_9";
