# Focusa Desktop shell provenance

The T070 shell was authored in the Focusa repository after reviewing the product-neutral desktop patterns on `WPUIAI/uiai-engine` branch `feat/cockpit-ota-updates` at local reference commit `cc87d75947c347bc14fd6e77ea60da448648ac35`.

Reused patterns are limited to generic SvelteKit static-adapter, Vite development-server, responsive application-frame, navigation-manifest, and Tauri 2 shell conventions. No UIAI Engine domain model, route, credential, updater, deep-link, browser-session, or canonical state implementation was copied into Focusa Desktop.

Focusa Desktop owns its product identity and workspace vocabulary. UIAI Engine remains the exclusive browser execution and evaluation authority used to prove this shared browser/Tauri application.
