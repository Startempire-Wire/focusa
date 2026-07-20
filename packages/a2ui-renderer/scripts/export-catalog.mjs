import { mkdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { JSDOM } from "jsdom";

const dom = new JSDOM("<!doctype html><html><body></body></html>");
for (const [name, value] of Object.entries({
  window: dom.window,
  document: dom.window.document,
  HTMLElement: dom.window.HTMLElement,
  customElements: dom.window.customElements,
  Element: dom.window.Element,
  Node: dom.window.Node,
  Event: dom.window.Event,
  MutationObserver: dom.window.MutationObserver,
})) {
  Object.defineProperty(globalThis, name, { configurable: true, value });
}

const { FocusaA2uiRenderer } = await import("../dist/index.js");
const renderer = new FocusaA2uiRenderer();
const capabilities = renderer.processor.getClientCapabilities({ includeInlineCatalogs: true });
const output = resolve(
  process.argv[2] ?? "../../docs/contracts/spec135/generated-contract-v1/a2ui-catalog.json",
);
const document = {
  schema: "focusa.a2ui_catalog.v1",
  protocol_version: "v0.9",
  package_lock: {
    "@a2ui/web_core": "0.9.1",
    "@a2ui/lit": "0.9.1",
    "@focusa/elements": "0.9.120-dev",
    lit: "3.3.1",
    svelte: "5.55.9",
  },
  renderer: "lit",
  capabilities,
};
await mkdir(dirname(output), { recursive: true });
await writeFile(output, `${JSON.stringify(document, null, 2)}\n`);
renderer.dispose();
console.log(output);
