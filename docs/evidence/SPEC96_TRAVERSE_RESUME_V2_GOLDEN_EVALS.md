# Spec96 Traverse + Resume Packet v2 Golden Evals

`tests/spec96_traverse_resume_v2_golden_eval_test.sh` compares the current Workpoint Resume Packet v2 + `focusa_traverse` posture against an old transcript-tail baseline.

Coverage:

- narrow traversal slices and field projection instead of full context reads;
- tag verification for stale/invalid anchors;
- resume packet v2 provenance and `failure_class` taxonomy;
- tool-choice accuracy: Workpoint resume → trajectory view → traverse → doctor/recovery;
- drift reduction: transcript-tail authority rejection, project_root+continuity_id, scope mismatch;
- daemon unavailable, stale tag, scope mismatch, and cold-path timeout taxonomy.

The safe audit (`scripts/audit-focusa-tool-suite-safe.mjs`) now fails missing bounded traversal, transcript-tail canonical resume, missing v2 provenance, stale-tag verification gaps, and missing timeout taxonomy.
