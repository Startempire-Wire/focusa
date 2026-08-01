import "@focusa/elements";
import type { A2uiClientAction, A2uiMessage } from "@a2ui/web_core/v0_9";
import { FocusaA2uiRenderer } from "./index.js";

export class FocusaGeneratedSurfaceElement extends HTMLElement {
  #renderer?: FocusaA2uiRenderer;
  #allowedActions = new Set<string>();
  #surfaceId = "";
  #mounted = false;

  set allowedActions(value: Iterable<string>) {
    this.#allowedActions = new Set(value);
    this.#renderer = undefined;
    this.#mounted = false;
  }

  set snapshot(messages: A2uiMessage[]) {
    this.#ensureRenderer();
    this.#renderer!.processSnapshot(messages);
    this.#mountFromMessages(messages);
  }

  set delta(messages: A2uiMessage[]) {
    this.#ensureRenderer();
    this.#renderer!.processDelta(messages);
    if (!this.#mounted) this.#mountFromMessages(messages);
  }

  connectedCallback(): void {
    this.setAttribute("role", "region");
    if (!this.hasAttribute("aria-label")) this.setAttribute("aria-label", "Generated Focusa surface");
  }

  #ensureRenderer(): void {
    if (this.#renderer) return;
    this.#renderer = new FocusaA2uiRenderer({
      allowedActionNames: this.#allowedActions,
      onAction: (action: A2uiClientAction) => {
        this.dispatchEvent(
          new CustomEvent("focusa-operation", {
            bubbles: true,
            composed: true,
            detail: action,
          })
        );
      },
    });
  }

  #mountFromMessages(messages: A2uiMessage[]): void {
    const create = messages.find((message) => "createSurface" in message) as
      | { createSurface?: { surfaceId?: string } }
      | undefined;
    this.#surfaceId = create?.createSurface?.surfaceId || this.#surfaceId;
    if (!this.#surfaceId) return;
    this.#renderer!.mount(this, this.#surfaceId);
    this.#mounted = true;
  }
}

if (!customElements.get("focusa-generated-surface")) {
  customElements.define("focusa-generated-surface", FocusaGeneratedSurfaceElement);
}
