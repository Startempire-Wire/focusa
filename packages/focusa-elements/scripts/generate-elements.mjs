import { readFile, mkdir, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const manifest = JSON.parse(await readFile(resolve(root, "src/component-manifest.json"), "utf8"));
const output = resolve(root, "src/generated");
await rm(output, { recursive: true, force: true });
await mkdir(output, { recursive: true });

const imports = [];
for (const component of manifest) {
  const source = `<script lang="ts">\n  import TrustedComponent from "../TrustedComponent.svelte";\n  let { label = "${component.name.replace(/^Focusa/, "").replace(/([a-z])([A-Z])/g, "$1 $2")}", description = "", status = "ready", progress = 0, primaryActionLabel = "Continue", actionAvailable = false, disabled = false, busy = false, details = "", invokeAction = undefined } = $props();\n</script>\n<svelte:options customElement="${component.tag}" />\n<TrustedComponent componentName="${component.name}" kind="${component.kind}" {label} {description} {status} {progress} {primaryActionLabel} {actionAvailable} {disabled} {busy} {details} {invokeAction} />\n`;
  await writeFile(resolve(output, `${component.name}.svelte`), source);
  imports.push(`import "./${component.name}.svelte";`);
}
await writeFile(resolve(output, "index.ts"), `${imports.join("\n")}\n`);
await writeFile(
  resolve(output, "manifest.ts"),
  `export const componentManifest = ${JSON.stringify(manifest, null, 2)} as const;\n`,
);
console.log(`generated ${manifest.length} trusted Focusa Svelte Custom Elements`);
