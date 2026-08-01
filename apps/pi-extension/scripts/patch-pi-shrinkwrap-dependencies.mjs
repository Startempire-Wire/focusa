import { cpSync, existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const safeBrace = join(root, "node_modules", "brace-expansion");
const bundledBrace = join(
  root,
  "node_modules",
  "@earendil-works",
  "pi-coding-agent",
  "node_modules",
  "brace-expansion"
);

const version = (dir) => JSON.parse(readFileSync(join(dir, "package.json"), "utf8")).version;

if (existsSync(bundledBrace)) {
  if (!existsSync(safeBrace) || version(safeBrace) !== "5.0.8") {
    throw new Error("safe brace-expansion@5.0.8 override is unavailable");
  }
  if (version(bundledBrace) !== "5.0.8") {
    // The upstream Pi package ships an npm-shrinkwrap with 5.0.7. Overlay the
    // root audited package without deleting the package tree, then verify it.
    cpSync(safeBrace, bundledBrace, { recursive: true, force: true });
  }
  if (version(bundledBrace) !== "5.0.8") {
    throw new Error("Pi shrinkwrap security overlay did not activate");
  }
  console.log("Pi shrinkwrap security overlay: brace-expansion@5.0.8");
}
