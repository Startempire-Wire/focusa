use serde::Deserialize;

#[test]
fn probe_old_snapshot_parse() {
    let raw = match std::fs::read_to_string("/tmp/backup-state.json") {
        Ok(raw) => raw,
        Err(_) => return, // probe file absent in CI — skip silently
    };
    let mut de = serde_json::Deserializer::from_str(&raw);
    let mut track = serde_path_to_error::Track::new();
    let path_de = serde_path_to_error::Deserializer::new(&mut de, &mut track);
    match <focusa_core::types::FocusaState as Deserialize>::deserialize(path_de) {
        Ok(_) => println!("PARSE-OK"),
        Err(e) => {
            panic!("PATH: {} ERR: {}", track.path(), e);
        }
    }
}
