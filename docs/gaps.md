# Focusa Spec Audit — COMPLETE
# Audited: 2026-04-03 (initial)
# Updated: 2026-04-11 (complete)

## IMPLEMENTED ✅

### CRITICAL (6/6)
1. §36.1 turn/append streaming — EXISTS in turn.rs
2. §36.2 Error signals — EXISTS, ingest-signal accepts extension format
3. §36.3 User input signals — EXISTS in turns.ts input handler
4. §34.2A Instance registration — EXISTS, dual format support
5. §33.3 ECS externalization — EXISTS in turns.ts tool_result handler
6. §33.4 Tool usage tracking — IMPLEMENTED: /v1/telemetry/tool-usage + tools GET

### HIGH (7/7)
7. §36.4 Session resume — IMPLEMENTED: /v1/session/resume + Action::ResumeSession
8. §36.5 Fork/tree — EXISTS: session_before_fork, session_fork, session_before_tree handlers
9. §33.7 State persistence — EXISTS: persistState() → appendEntry
10. §33.8 Session close — EXISTS: session_shutdown → /session/close
11. §35.5 Token counts — IMPLEMENTED: TurnComplete.tokens field with extension alias
12. §35.6 File tracking — EXISTS: modifiedFiles + file_churn signal
13. §35.7 Operator correction — EXISTS: turns.ts input handler with phrase detection

### MEDIUM (5/5)
14. §10.4 UI widgets — EXISTS: turn_end widget with all badges
15. §30 Metacognitive indicators — EXISTS: lastMetacogEvent in widget
16. §33.6 registerProvider — EXISTS: pi.registerProvider("focusa") in index.ts
17. §37.8 Model change tracking — EXISTS: model_select → ingest-signal
18. §38.3 Disable tools — EXISTS: graceful degradation on S.focusaAvailable

### LOW (6/6)
19. §37.4 Keyboard shortcuts — EXISTS: ctrl+shift+f (status), ctrl+shift+w (WBM)
20. §37.5 CLI flags — EXISTS: registerFlag('wbm'), registerFlag('no-focusa')
21. §37.6 Custom message renderer — EXISTS: registerMessageRenderer('focusa-state')
22. §35.8 Session name from frame — EXISTS: pi.setSessionName(active.title)
23. §34.2H Prompt template — IMPLEMENTED: .pi/prompts/focusa-context.md
24. §38.2 /wbm HTTP fallback — EXISTS: wbExec() with fallbackUrl/fallbackBody

## TEST COVERAGE

### CI Integration Tests (120+ tests)
- focusa_toggle_persistence_test.sh (SPEC-33.5)
- behavioral_alignment_test.sh (SPEC-53)
- trace_dimensions_test.sh (SPEC-56.1)
- checkpoint_trigger_test.sh (SPEC-56.2)
- channel_separation_test.sh (SPEC-54/54a)
- tool_contract_test.sh (SPEC-55)
- pi_extension_contract_test.sh (SPEC-52)
- golden_tasks_eval.sh (SPEC-57)
- menubar CI workflow

### API Contract Tests
- api_contract_probe.py (16+ checks)

## STATUS: COMPLETE ✅
