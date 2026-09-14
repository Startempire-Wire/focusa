#!/usr/bin/env node
// Tool taxonomy audit (#258 slice 1): enumerate the Pi extension's
// focusa_* tools, group by family, and flag semantic-duplicate candidates
// (same family + near-identical action verbs in the name).
import { readFileSync } from "node:fs";

const path = "/root/.pi/agent/extensions/focusa/src/tools.ts";
const src = readFileSync(path, "utf8");
const names = [...src.matchAll(/name: "focusa_([a-z0-9_]+)"/g)].map((m) => m[1]);
const families = {};
for (const name of names) {
  const [family, ...rest] = name.split("_");
  (families[family] ||= []).push(rest.join("_"));
}
console.log(`total tools: ${names.length}`);
console.log(`families: ${Object.keys(families).length}`);
for (const [family, members] of Object.entries(families).sort((a, b) => b[1].length - a[1].length)) {
  console.log(`${String(members.length).padStart(3)}  ${family}: ${members.join(", ")}`);
}
// semantic-duplicate candidates: same family, names differing only by
// a suffix like _status/_state/_view/_list/_read/_get/_current.
const candidates = [];
for (const [family, members] of Object.entries(families)) {
  const groups = {};
  for (const member of members) {
    const base = member.replace(/_(status|state|view|list|read|get|current|check|query)$/, "");
    (groups[base] ||= []).push(member);
  }
  for (const [base, variants] of Object.entries(groups)) {
    if (variants.length > 1) {
      candidates.push(`focusa_${family}: ${base} → ${variants.join(" | ")}`);
    }
  }
}
console.log("");
console.log(`semantic-duplicate candidates: ${candidates.length}`);
for (const candidate of candidates) console.log(`  ${candidate}`);
