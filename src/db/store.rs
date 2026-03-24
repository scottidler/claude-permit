use eyre::{Context, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Manages the SQLite event database.
pub struct EventStore {
    conn: Connection,
}

impl EventStore {
    /// Open (or create) the database at the given path, with WAL mode and schema init.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create database directory")?;
        }

        let conn = Connection::open(path).context("Failed to open SQLite database")?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .context("Failed to set WAL mode")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp   TEXT NOT NULL,
                session_id  TEXT NOT NULL,
                tool_name   TEXT NOT NULL,
                tool_input  TEXT NOT NULL,
                raw_input   TEXT,
                risk_tier   TEXT,
                raw_json    TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id);
            CREATE INDEX IF NOT EXISTS idx_events_tool ON events(tool_name, tool_input);
            CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);",
        )
        .context("Failed to initialize database schema")?;

        Ok(Self { conn })
    }

    /// Default database path: ~/.local/share/claude-permit/events.db
    pub fn default_path() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir().ok_or_else(|| eyre::eyre!("Could not determine local data directory"))?;
        Ok(data_dir.join("claude-permit").join("events.db"))
    }

    /// Insert a new event.
    pub fn insert_event(
        &self,
        timestamp: &str,
        session_id: &str,
        tool_name: &str,
        tool_input: &str,
        raw_input: Option<&str>,
        risk_tier: Option<&str>,
        raw_json: Option<&str>,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO events (timestamp, session_id, tool_name, tool_input, raw_input, risk_tier, raw_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    timestamp, session_id, tool_name, tool_input, raw_input, risk_tier, raw_json
                ],
            )
            .context("Failed to insert event")?;
        Ok(())
    }

    /// Count total events in the database.
    pub fn count_events(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .context("Failed to count events")?;
        Ok(count)
    }

    /// Check if the database is writable by performing a test write and rollback.
    pub fn is_writable(&self) -> bool {
        self.conn.execute_batch("BEGIN; ROLLBACK;").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct TestDb {
        store: EventStore,
        // Kept alive so the temp dir isn't deleted while tests run.
        // Accessed via path() in tests that need the directory.
        dir: TempDir,
    }

    impl TestDb {
        fn new() -> Self {
            let dir = TempDir::new().expect("create temp dir");
            let db_path = dir.path().join("test.db");
            let store = EventStore::open(&db_path).expect("open store");
            Self { store, dir }
        }

        fn path(&self) -> &Path {
            self.dir.path()
        }
    }

    #[test]
    fn open_creates_db_and_tables() {
        let t = TestDb::new();
        assert!(t.store.is_writable());
        assert_eq!(t.store.count_events().expect("count"), 0);
    }

    #[test]
    fn insert_and_count() {
        let t = TestDb::new();
        t.store
            .insert_event(
                "2026-03-24T12:00:00Z",
                "session-1",
                "Bash",
                "git status",
                Some(r#"{"command":"git status"}"#),
                Some("safe"),
                None,
            )
            .expect("insert");
        assert_eq!(t.store.count_events().expect("count"), 1);

        t.store
            .insert_event(
                "2026-03-24T12:01:00Z",
                "session-1",
                "Edit",
                "/tmp/foo.rs",
                None,
                Some("moderate"),
                None,
            )
            .expect("insert");
        assert_eq!(t.store.count_events().expect("count"), 2);
    }

    #[test]
    fn open_idempotent() {
        let t = TestDb::new();
        let db_path = t.path().join("reopen.db");
        let store1 = EventStore::open(&db_path).expect("open 1");
        store1
            .insert_event("2026-03-24T12:00:00Z", "s1", "Bash", "ls", None, None, None)
            .expect("insert");
        drop(store1);

        // Re-opening should not lose data
        let store2 = EventStore::open(&db_path).expect("open 2");
        assert_eq!(store2.count_events().expect("count"), 1);
    }

    #[test]
    fn creates_parent_directories() {
        let t = TestDb::new();
        let db_path = t.path().join("nested").join("dirs").join("test.db");
        let store = EventStore::open(&db_path).expect("open");
        assert!(store.is_writable());
    }
}
