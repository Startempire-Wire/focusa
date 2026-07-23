import {
  NATIVE_ANCHOR_MAX_BYTES,
  semanticPersistenceDigest,
} from "../apps/pi-extension/src/persistence.ts";
import {
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "fs";
import { tmpdir } from "os";
import { join } from "path";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const TOTAL_EVENTS = 1_000_000;
const TOTAL_CYCLES = 10_000;
const EVENTS_PER_CYCLE = TOTAL_EVENTS / TOTAL_CYCLES;
const PHYSICAL_SEGMENTS = 10;
const CYCLES_PER_SEGMENT = TOTAL_CYCLES / PHYSICAL_SEGMENTS;
const root = mkdtempSync(join(tmpdir(), "focusa-spec130-million-soak-"));

let nativeAppends = 0;
let unchangedStateSuppressions = 0;
let maxAnchorBytes = 0;
let lastDigest = "";
const segmentPaths: string[] = [];
const replayWorkingSetBytes: number[] = [];
const replayRssBytes: number[] = [];

try {
  for (let segment = 0; segment < PHYSICAL_SEGMENTS; segment += 1) {
    const continuityId = `soak-continuity-${segment + 1}`;
    const segmentAnchors: string[] = [];

    for (let localCycle = 0; localCycle < CYCLES_PER_SEGMENT; localCycle += 1) {
      const cycle = segment * CYCLES_PER_SEGMENT + localCycle;
      const semanticState = {
        mission: "Keep million-event recovery bounded",
        continuity_id: continuityId,
        workpoint: {
          id: "workpoint-spec130-soak",
          revision: cycle + 1,
          blocker_refs: ["blocker:soak-authority"],
          evidence_refs: ["evidence:soak-integrity"],
          receipt_refs: ["receipt:soak-transition"],
        },
        next_action: "advance bounded rollover cycle",
      };
      const expectedDigest = semanticPersistenceDigest(semanticState);

      for (let observation = 0; observation < EVENTS_PER_CYCLE; observation += 1) {
        const observedState = {
          ...semanticState,
          timestamp: cycle * EVENTS_PER_CYCLE + observation,
          updated_at: `volatile-${cycle}-${observation}`,
          last_seen_turn: `turn-${cycle}-${observation}`,
          turnCount: observation,
        };
        const digest = semanticPersistenceDigest(observedState);
        assert(digest === expectedDigest, "volatile observation changed semantic digest");
        if (digest === lastDigest) {
          unchangedStateSuppressions += 1;
          continue;
        }

        const anchor = {
          schema: "focusa.compaction_persistence_anchor.v1",
          semanticDigest: digest,
          continuity_id: continuityId,
          workpoint_id: semanticState.workpoint.id,
          workpoint_revision: semanticState.workpoint.revision,
          blocker_refs: semanticState.workpoint.blocker_refs,
          evidence_refs: semanticState.workpoint.evidence_refs,
          receipt_refs: semanticState.workpoint.receipt_refs,
        };
        const encoded = JSON.stringify(anchor);
        const anchorBytes = Buffer.byteLength(encoded, "utf8");
        assert(anchorBytes <= NATIVE_ANCHOR_MAX_BYTES, "soak anchor exceeded native hard cap");
        maxAnchorBytes = Math.max(maxAnchorBytes, anchorBytes);
        segmentAnchors.push(encoded);
        nativeAppends += 1;
        lastDigest = digest;
      }
    }

    assert(
      segmentAnchors.length === CYCLES_PER_SEGMENT,
      `segment ${segment + 1} did not retain exactly one changed-state anchor per cycle`
    );
    const segmentPath = join(root, `segment-${String(segment + 1).padStart(2, "0")}.jsonl`);
    writeFileSync(segmentPath, `${segmentAnchors.join("\n")}\n`, { mode: 0o600, flag: "wx" });
    segmentPaths.push(segmentPath);
  }

  assert(nativeAppends === TOTAL_CYCLES, "changed semantic cycles were lost or duplicated");
  assert(
    unchangedStateSuppressions === TOTAL_EVENTS - TOTAL_CYCLES,
    "unchanged semantic observations were not fully suppressed"
  );
  assert(segmentPaths.length === PHYSICAL_SEGMENTS, "physical segment rotation count mismatch");

  for (const [index, segmentPath] of segmentPaths.entries()) {
    const lines = readFileSync(segmentPath, "utf8").trim().split("\n");
    const latest = JSON.parse(lines.at(-1) || "null");
    assert(latest?.continuity_id === `soak-continuity-${index + 1}`, "segment continuity was lost");
    assert(latest?.blocker_refs?.[0] === "blocker:soak-authority", "blocker ref was lost");
    assert(latest?.evidence_refs?.[0] === "evidence:soak-integrity", "evidence ref was lost");
    assert(latest?.receipt_refs?.[0] === "receipt:soak-transition", "receipt ref was lost");
    replayWorkingSetBytes.push(Buffer.byteLength(JSON.stringify(latest), "utf8"));
    globalThis.gc?.();
    replayRssBytes.push(process.memoryUsage().rss);
    assert(statSync(segmentPath).size > 0, "empty physical segment");
  }

  const xMean = (PHYSICAL_SEGMENTS - 1) / 2;
  const yMean = replayWorkingSetBytes.reduce((sum, value) => sum + value, 0) / PHYSICAL_SEGMENTS;
  const numerator = replayWorkingSetBytes.reduce(
    (sum, value, index) => sum + (index - xMean) * (value - yMean),
    0
  );
  const denominator = replayWorkingSetBytes.reduce(
    (sum, _value, index) => sum + (index - xMean) ** 2,
    0
  );
  const replaySlopeBytesPerSegment = denominator === 0 ? 0 : numerator / denominator;
  const rssRangeBytes = Math.max(...replayRssBytes) - Math.min(...replayRssBytes);
  assert(
    Math.abs(replaySlopeBytesPerSegment) <= 4,
    `bounded replay working-set slope is not flat: ${replaySlopeBytesPerSegment}`
  );
  assert(rssRangeBytes <= 64 * 1024 * 1024, `bounded replay RSS range exceeded 64 MiB: ${rssRangeBytes}`);

  console.log(
    JSON.stringify(
      {
        status: "passed",
        semantic_events: TOTAL_EVENTS,
        cycles: TOTAL_CYCLES,
        physical_segments: PHYSICAL_SEGMENTS,
        native_appends: nativeAppends,
        unchanged_state_suppressions: unchangedStateSuppressions,
        max_anchor_bytes: maxAnchorBytes,
        replay_slope_bytes_per_segment: replaySlopeBytesPerSegment,
        replay_rss_range_bytes: rssRangeBytes,
        required_ref_loss: 0,
      },
      null,
      2
    )
  );
} finally {
  rmSync(root, { recursive: true, force: true });
}
