#!/usr/bin/env node
// Skill ownership audit (#259 slice 1): every SKILL.md must appear in the
// manifest with an assigned owner, and every referenced runbook must exist.
import { readFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const repoRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const manifestPath = join(repoRoot, ".pi/skills/skills-manifest.v1.json");
const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
const skills = manifest.skills;
console.log(`manifest skills: ${skills.length}`);

let failures = 0;
for (const entry of skills) {
  const skillMd = join(repoRoot, entry.skill_md);
  if (!existsSync(skillMd)) {
    console.log(`MISSING skill_md: ${entry.skill}`);
    failures++;
  }
  if (entry.runbook && !existsSync(join(repoRoot, entry.runbook))) {
    console.log(`MISSING runbook: ${entry.runbook} (${entry.skill})`);
    failures++;
  }
  if (!entry.owner_ref || entry.owner_ref === "unassigned") {
    console.log(`UNASSIGNED owner: ${entry.skill}`);
    failures++;
  }
}
console.log(failures === 0 ? "SKILL-OWNERSHIP-GREEN" : `${failures} failures`);
process.exit(failures === 0 ? 0 : 1);
