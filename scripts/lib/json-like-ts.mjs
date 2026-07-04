export function parseJsonLikeTsLiteral(source) {
  // Contract registries are JSON-shaped TypeScript literals. Normalize common
  // TS-only trailing commas so scripts can share one canonical parser.
  return JSON.parse(source.replace(/,\s*([}\]])/g, '$1'));
}
