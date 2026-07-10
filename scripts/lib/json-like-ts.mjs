export function parseJsonLikeTsLiteral(source) {
  // Contract registries are JSON-shaped TypeScript literals. Normalize common
  // TS-only object keys and trailing commas so scripts can share one canonical parser.
  const jsonLike = source
    .replace(/([,{]\s*)([A-Za-z_$][A-Za-z0-9_$]*)(\s*:)/g, '$1"$2"$3')
    .replace(/,\s*([}\]])/g, '$1');
  return JSON.parse(jsonLike);
}
