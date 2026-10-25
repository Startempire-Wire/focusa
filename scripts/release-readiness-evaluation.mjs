#!/usr/bin/env node
// Compatibility entrypoint only. This diagnostic cannot grant release readiness.
import { main } from './release-diagnostics-advisory.mjs';
await main({ legacy: true });
