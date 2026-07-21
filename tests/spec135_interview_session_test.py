#!/usr/bin/env python3
import json
import pathlib

R = pathlib.Path(__file__).resolve().parents[1]
C = R / "docs/contracts/spec135/generated-contract-v1"


def j(p):
    return json.loads((C / p).read_text())


def main():
    t = (R / "crates/focusa-core/src/types.rs").read_text()
    r = (R / "crates/focusa-core/src/reducer.rs").read_text()
    a = (R / "crates/focusa-api/src/routes/interview_sessions.rs").read_text()
    u = (R / "packages/a2ui-renderer/proof/interview-resume.ts").read_text()
    for x in [
        "ProjectInterviewSessionRecord",
        "ProjectInterviewBranchRecord",
        "ProjectInterviewQuestionRecord",
        "ProjectInterviewAnswerRecord",
    ]:
        assert x in t
    for x in [
        "resume pointers",
        "project_interview_sessions.push",
        "approved Role Profile",
    ]:
        assert x in r
    for x in [
        "Open",
        "QueueQuestion",
        "RecordAnswer",
        "Pause",
        "Close",
        "Reopen",
        "DeferBranch",
    ]:
        assert x in a
    ops = {x["operation_id"]: x for x in j("operation-registry.json")["operations"]}
    assert ops["focusa.interview.session.list"]["scope"]["required_keys"] == [
        "project_root",
        "continuity_id",
        "attachment_id",
    ]
    assert (
        ops["focusa.interview.session.mutate"]["materialization_mode"]
        == "canonical_event"
    )
    for s in [
        "focusa.project_interview_session_list.request.v1",
        "focusa.project_interview_session_list.v1",
        "focusa.project_interview_session_mutation.request.v1",
        "focusa.project_interview_session_mutation_result.v1",
    ]:
        assert (C / "json-schema" / f"{s}.json").exists()
    assert (
        "FocusaA2uiRenderer" in u
        and "Run Resume Proof" in u
        and "playwright" not in u.lower()
    )
    assert j("spec135-ri3-interview-resume-proof.json")["status"] == "passed"
    assert j("uiai-eval.ri3-interview-resume.result.json")["status"] == "passed"
    print("Spec 135 RI3 durable Interview contracts/UI proof: PASS")


if __name__ == "__main__":
    main()
