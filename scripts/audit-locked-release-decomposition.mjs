#!/usr/bin/env node
import { readFileSync } from "node:fs";
import process from "node:process";

const root = process.cwd();
const readJson = (path) => JSON.parse(readFileSync(`${root}/${path}`, "utf8"));
const requirementIds = (path) => {
  const source = readFileSync(`${root}/${path}`, "utf8");
  try {
    return JSON.parse(source).requirements.map((row) => row.requirement_id);
  } catch {
    return [
      ...source.matchAll(/^\s*-?\s*requirement_id:\s*['\"]?([^'\"\s]+)['\"]?\s*$/gm),
    ].map((match) => match[1]);
  }
};
const manifest = readJson("release-proof/audit/next-locked-release-scope.json");
const audit = readJson("release-proof/audit/next-locked-release-decomposition.json");
const failures = [];

function sameSet(label, expected, actual) {
  const left = new Set(expected);
  const right = new Set(actual);
  const missing = [...left].filter((value) => !right.has(value));
  const extra = [...right].filter((value) => !left.has(value));
  if (missing.length || extra.length) failures.push(`${label}: missing=${missing.join(",")} extra=${extra.join(",")}`);
}

if (manifest.schema !== "focusa.locked_release_scope.v2") failures.push("locked manifest schema is not v2");
if (audit.schema !== "focusa.locked_release_decomposition.v2") failures.push("decomposition schema is not v2");
if (manifest.pre_decomposition_open_issue_count !== 38) failures.push("original locked manifest count is not 38");
if (manifest.lock_revision !== 3 || manifest.scope_state !== "locked" || !manifest.relocked_at) {
  failures.push("release was not durably relocked at revision 3");
}
if (manifest.current_explicit_issue_count !== 41) failures.push("current explicit issue count is not 41");
if (manifest.current_locked_bead_member_count !== 237) failures.push("current locked Bead member count is not 237");
sameSet(
  "original locked issues",
  manifest.pre_decomposition_open_issue_ids,
  audit.original_locked_issues.map((entry) => entry.locked_issue_id)
);
for (const entry of audit.original_locked_issues) {
  if (!entry.decomposition_leaf_refs?.length) failures.push(`locked issue has no decomposition: ${entry.locked_issue_id}`);
}
const manifestAdditions = manifest.operator_authorized_post_lock_additions ?? [];
const decompositionAdditions = audit.operator_authorized_post_lock_additions ?? [];
if (manifestAdditions.length !== 3 || decompositionAdditions.length !== 3) {
  failures.push("operator-authorized post-lock addition count is not 3");
}
sameSet(
  "operator-authorized post-lock additions",
  manifestAdditions.map((entry) => entry.issue_id),
  decompositionAdditions.map((entry) => entry.issue_id)
);
for (const entry of decompositionAdditions) {
  if (entry.kind !== "bug" || !entry.lane || !entry.parent || !entry.terminal_gate || !entry.acceptance_ref) {
    failures.push(`post-lock addition contract incomplete: ${entry.issue_id}`);
  }
}
for (const spec of ["137", "138", "140"]) {
  const ledgerIds = requirementIds(`docs/contracts/spec${spec}-complete-feature-ledger.v1.yaml`);
  sameSet(
    `Spec ${spec} requirement mappings`,
    ledgerIds,
    audit.specs[spec].requirement_mappings.map((row) => row.requirement_id)
  );
  for (const row of audit.specs[spec].requirement_mappings) {
    if (!row.bead_ref) failures.push(`Spec ${spec} requirement lacks Bead: ${row.requirement_id}`);
  }
}
for (const spec of ["137a", "138a"]) {
  if (!audit.specs[spec].leaf_refs?.length) failures.push(`Spec ${spec} has no leaf Beads`);
  if (!audit.specs[spec].section_mappings?.length) failures.push(`Spec ${spec} has no section mappings`);
}
if (audit.unmapped_locked_issue_ids.length) failures.push("unmapped locked issue ids remain");
if (audit.unmapped_requirement_ids.length) failures.push("unmapped requirement ids remain");

const beadProofPath = process.argv[2];
if (beadProofPath) {
  const proof = JSON.parse(readFileSync(beadProofPath, "utf8"));
  const present = new Set(proof.map((issue) => issue.id));
  const refs = new Set([audit.acceptance_gate]);
  for (const entry of audit.original_locked_issues) entry.decomposition_leaf_refs.forEach((ref) => refs.add(ref));
  for (const entry of decompositionAdditions) refs.add(entry.issue_id);
  for (const spec of Object.values(audit.specs)) {
    refs.add(spec.parent);
    spec.leaf_refs?.forEach((ref) => refs.add(ref));
    spec.requirement_mappings?.forEach((row) => refs.add(row.bead_ref));
    spec.section_mappings?.forEach((row) => refs.add(row.bead_ref));
  }
  for (const ref of refs) if (!present.has(ref)) failures.push(`mapped Bead does not exist: ${ref}`);
}

if (failures.length) {
  console.error(JSON.stringify({ schema: "focusa.locked_release_decomposition_audit.v2", status: "failed", failures }, null, 2));
  process.exit(1);
}
console.log(JSON.stringify({
  schema: "focusa.locked_release_decomposition_audit.v2",
  status: "verified",
  original_locked_issues: 38,
  operator_authorized_post_lock_additions: decompositionAdditions.length,
  current_explicit_issues: manifest.current_explicit_issue_count,
  current_locked_beads: manifest.current_locked_bead_member_count,
  spec137_requirements: audit.specs["137"].requirement_mappings.length,
  spec138_requirements: audit.specs["138"].requirement_mappings.length,
  spec140_requirements: audit.specs["140"].requirement_mappings.length,
  addendum_leaf_beads: audit.specs["137a"].leaf_refs.length + audit.specs["138a"].leaf_refs.length,
  unmapped: 0
}, null, 2));
