#!/usr/bin/env bash
# Existing CI dependency contract, shared by Pi and Rust HTTP consumer tests.
set -euo pipefail
cd "$(dirname "$0")/../.."
# The OVH source dispatcher may loan a dependency cache via this symlink.
# Detach only the invocation's link before npm manages its private directory.
if [[ -L apps/pi-extension/node_modules ]]; then
  unlink apps/pi-extension/node_modules
fi
npm ci --ignore-scripts --include=dev --prefix apps/pi-extension
(
  cd apps/pi-extension
  node ../../scripts/patch-pi-shrinkwrap-dependencies.mjs
  node scripts/patch-pi-shrinkwrap-dependencies.mjs
  node -e 'console.log("Pi test SDK: " + JSON.parse(require("node:fs").readFileSync("node_modules/@earendil-works/pi-coding-agent/package.json", "utf8")).version)'
)
