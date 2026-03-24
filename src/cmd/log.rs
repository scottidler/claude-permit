use eyre::Result;
use std::io::Read;

use crate::db::EventStore;
use crate::hook::{HookPayload, normalize_tool_input};

/// Run the `log` subcommand: read hook JSON from stdin, write event to DB, output `{}`.
///
/// On any error, still outputs `{}` to stdout so the hook pipeline is never blocked.
pub fn run_log(store: &EventStore) -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let payload: HookPayload = serde_json::from_str(&input)?;

    let normalized = normalize_tool_input(&payload.tool_name, &payload.tool_input);
    let raw_input = serde_json::to_string(&payload.tool_input)?;
    let session_id = payload.session_id.as_deref().unwrap_or("unknown");
    let timestamp = chrono::Utc::now().to_rfc3339();

    store.insert_event(
        &timestamp,
        session_id,
        &payload.tool_name,
        &normalized,
        Some(&raw_input),
        None, // risk_tier computed in Phase 2
        Some(&input),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDb {
        store: EventStore,
        dir: tempfile::TempDir,
    }

    impl TestDb {
        fn new() -> Self {
            let dir = tempfile::TempDir::new().expect("temp dir");
            let store = EventStore::open(&dir.path().join("test.db")).expect("open");
            Self { store, dir }
        }

        fn path(&self) -> &std::path::Path {
            self.dir.path()
        }
    }

    #[test]
    fn log_inserts_event() {
        let t = TestDb::new();
        let _ = t.path(); // keep dir alive
        let json = r#"{"tool_name":"Bash","tool_input":{"command":"git status"},"session_id":"s1"}"#;

        let payload: HookPayload = serde_json::from_str(json).expect("parse");
        let normalized = normalize_tool_input(&payload.tool_name, &payload.tool_input);
        let raw_input = serde_json::to_string(&payload.tool_input).expect("serialize");
        let session_id = payload.session_id.as_deref().unwrap_or("unknown");
        let timestamp = chrono::Utc::now().to_rfc3339();

        t.store
            .insert_event(
                &timestamp,
                session_id,
                &payload.tool_name,
                &normalized,
                Some(&raw_input),
                None,
                Some(json),
            )
            .expect("insert");

        assert_eq!(t.store.count_events().expect("count"), 1);
    }
}
