import { describe, expect, it } from "vitest";
import { componentManifest } from "../src/index.js";

describe("Focusa Svelte Custom Element manifest", () => {
  it("registers every required initial trusted element exactly once", () => {
    expect(componentManifest).toHaveLength(31);
    expect(new Set(componentManifest.map(({ name }) => name)).size).toBe(31);
    expect(new Set(componentManifest.map(({ tag }) => tag)).size).toBe(31);
    for (const definition of componentManifest) {
      expect(definition.name).toMatch(/^Focusa/);
      expect(definition.tag).toMatch(/^focusa-/);
      expect(customElements.get(definition.tag)).toBeTruthy();
    }
  });

  it("exposes real responsive custom-element presentation", async () => {
    const element = document.createElement("focusa-recovery-card") as HTMLElement & {
      label: string;
      description: string;
      status: string;
      details: string;
    };
    element.label = "Recover work";
    element.description = "The action is not available.";
    element.status = "retry";
    element.details = "No action executed";
    document.body.append(element);
    await Promise.resolve();
    expect(element.shadowRoot?.querySelector('[role="alert"]')).toBeTruthy();
    expect(element.shadowRoot?.textContent).toContain("Recover work");
    expect(element.shadowRoot?.querySelector("[data-terminal-fallback]")).toBeTruthy();
    element.remove();
  });
});
