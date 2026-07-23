#!/usr/bin/env python3
from pathlib import Path
S=(Path(__file__).resolve().parents[1]/'crates/focusa-core/src/silent_sessions/platform_backends.rs').read_text()
for x in ['HeuristicLow','HeuristicMedium','Verified','heuristic output cannot become a structured fact','GenericAdapterCapabilities','rpc','pty','TmuxMigrationBackend','imported_aliases','imported_log_refs','canonical_identity_owner','canonical_state_owner','model_owner','health_owner','HerdrBackend','capabilities_negotiated','daemon_canonical_authority','Unsupported','Experimental','Proven','process_tree','streams','controls','pause_declared','recovery','owner_execution','resources_declared','job_object','conpty','runtime_suite_ref','platform remains explicitly unsupported until runtime proof is green','Windows requires Job Object and ConPTY proof']:
 assert x in S,x
assert len(S.splitlines())<=500
print('Spec133 optional/platform backend truth static contract: PASS')
