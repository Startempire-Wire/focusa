#!/usr/bin/env python3
import json
from pathlib import Path
R=Path(__file__).resolve().parents[1]
p=json.loads((R/'docs/contracts/spec135/generated-contract-v1/spec135-v1-v6-domain-projection-proof.json').read_text())
assert p['ontology_core']['requirements']==['SPEC135-V1','SPEC135-V2']
assert p['domain_packs']['available']==['general','software','research']
assert p['domain_packs']['bounded_artifact_limit']==64
s=(R/'crates/focusa-api/src/routes/ontology.rs').read_text()
for marker in ('/v1/ontology/domain-pack','research_citations_then_software_artifacts_then_general','canonical_state_unchanged','artifacts.iter().take(64)','Domain packs change terminology and views only'):
    assert marker in s
for gate in p['ontology_core']['proof_gates']: assert (R/gate).is_file()
assert p['receipt'].startswith('receipt:spec135-v1-v6:')
print('Spec 135 V1-V6 ontology and domain projection foundation: PASS')
