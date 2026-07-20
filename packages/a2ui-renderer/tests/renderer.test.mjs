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
    "https://a2ui.org/specification/v0_9/basic_catalog.json",
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

  assert.equal(
    renderer.clientCapabilities()["v0.9"].supportedCatalogIds[0],
    FOCUSA_A2UI_CATALOG_ID,
  );
  renderer.dispose();
  assert.equal(document.body.children.length, 0);
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
