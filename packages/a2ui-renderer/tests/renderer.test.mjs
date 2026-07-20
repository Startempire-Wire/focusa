import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { JSDOM } from "jsdom";

const dom = new JSDOM("<!doctype html><html><body></body></html>", {
  url: "http://localhost/",
});
for (const [name, value] of Object.entries({
  window: dom.window,
  document: dom.window.document,
  HTMLElement: dom.window.HTMLElement,
  customElements: dom.window.customElements,
  Element: dom.window.Element,
  Node: dom.window.Node,
  Text: dom.window.Text,
  Comment: dom.window.Comment,
  Document: dom.window.Document,
  ShadowRoot: dom.window.ShadowRoot,
  CustomEvent: dom.window.CustomEvent,
  Event: dom.window.Event,
  MutationObserver: dom.window.MutationObserver,
  requestAnimationFrame: (callback) => setTimeout(callback, 0),
  cancelAnimationFrame: (id) => clearTimeout(id),
})) {
  Object.defineProperty(globalThis, name, { configurable: true, value });
}

const { FocusaA2uiRenderer, FOCUSA_A2UI_CATALOG_ID } = await import("../dist/index.js");
const snapshot = JSON.parse(
  await readFile(new URL("../fixtures/mission-snapshot.json", import.meta.url), "utf8"),
);
const delta = JSON.parse(
  await readFile(new URL("../fixtures/mission-delta.json", import.meta.url), "utf8"),
);

async function settle(element) {
  await element.updateComplete;
  await new Promise((resolve) => setTimeout(resolve, 0));
  const child = element.shadowRoot?.querySelector("a2ui-basic-text");
  if (child?.updateComplete) await child.updateComplete;
  return child;
}

test("A2UI v0.9.1 snapshot and delta render deterministically through Lit", async () => {
  assert.equal(
    FOCUSA_A2UI_CATALOG_ID,
    "https://focusa.dev/a2ui/v0_9/catalog.json",
  );
  const renderer = new FocusaA2uiRenderer();
  renderer.processSnapshot(snapshot);
  assert.deepEqual(renderer.surfaceIds(), ["mission-canvas"]);

  const element = renderer.mount(document.body, "mission-canvas");
  let child = await settle(element);
  assert.match(child?.shadowRoot?.textContent ?? "", /Mission ready/);

  renderer.processDelta(delta);
  child = await settle(element);
  assert.match(child?.shadowRoot?.textContent ?? "", /Mission resumed/);

  assert.ok(
    renderer.clientCapabilities()["v0.9"].supportedCatalogIds.includes(FOCUSA_A2UI_CATALOG_ID),
  );
  renderer.dispose();
  assert.equal(document.body.children.length, 0);
});

test("unknown components and actions fail closed with explicit recovery", async () => {
  let executed = 0;
  const renderer = new FocusaA2uiRenderer({ onAction: () => { executed += 1; } });
  renderer.processSnapshot([
    {
      version: "v0.9",
      createSurface: { surfaceId: "trusted", catalogId: FOCUSA_A2UI_CATALOG_ID },
    },
    {
      version: "v0.9",
      updateComponents: {
        surfaceId: "trusted",
        components: [{
          id: "root",
          component: "FocusaPrimaryAction",
          label: "Continue",
          action: { event: { name: "context.commit", context: {} } },
        }],
      },
    },
  ]);
  const surface = renderer.mount(document.body, "trusted");
  await surface.updateComplete;
  const adapter = surface.shadowRoot?.querySelector("focusa-primary-action-a2ui");
  await adapter?.updateComplete;
  const trusted = adapter?.shadowRoot?.querySelector("focusa-primary-action");
  await new Promise((resolve) => setTimeout(resolve, 0));
  trusted?.shadowRoot?.querySelector("button")?.click();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(executed, 0);
  assert.ok(document.body.querySelector("focusa-recovery-card"));

  renderer.processSnapshot([
    {
      version: "v0.9",
      createSurface: { surfaceId: "unknown", catalogId: FOCUSA_A2UI_CATALOG_ID },
    },
    {
      version: "v0.9",
      updateComponents: {
        surfaceId: "unknown",
        components: [{ id: "root", component: "UntrustedWidget" }],
      },
    },
  ]);
  const fallback = renderer.mount(document.body, "unknown");
  await fallback.updateComplete;
  assert.ok(fallback.shadowRoot?.querySelector("focusa-recovery-card-a2ui"));
  renderer.dispose();
});

test("renderer rejects wrong protocol and unbounded payloads", () => {
  const renderer = new FocusaA2uiRenderer({ limits: { maxMessages: 1 } });
  assert.throws(() => renderer.processDelta([]), /message count/);
  assert.throws(
    () => renderer.processDelta([{ version: "v0.8", deleteSurface: { surfaceId: "x" } }]),
    /Unsupported A2UI protocol version/,
  );
  assert.throws(() => renderer.processSnapshot(snapshot), /message count/);
  renderer.dispose();
});
