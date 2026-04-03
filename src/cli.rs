use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "claude-permit",
    about = "Manage Claude Code permission hygiene",
    version = env!("GIT_DESCRIBE"),
    after_help = "Logs are written to: ~/.local/share/claude-permit/logs/claude-permit.log"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Log a permission event from hook JSON (reads stdin)
    Log,

    /// Audit current permission rules and classify by risk
    Audit {
        /// Override settings.json path
        #[arg(long)]
        settings: Option<PathBuf>,

        /// Override settings.local.json path
        #[arg(long)]
        settings_local: Option<PathBuf>,

        /// Output format: table, json, markdown
        #[arg(long, default_value = "table")]
        format: String,

        /// Filter by risk tier: safe, moderate, dangerous (cannot be combined with --apply)
        #[arg(long, conflicts_with = "apply")]
        risk: Option<String>,

        /// Apply recommendations and write changes. Optionally specify actions:
        /// promote, remove, deny, dupe (default: all). Cannot be combined with --risk.
        #[arg(long, value_name = "ACTION", num_args = 0.., conflicts_with = "risk")]
        apply: Option<Vec<String>>,

        /// Rule patterns to filter output (exact, prefix, or substring match)
        #[arg(value_name = "PATTERN")]
        patterns: Vec<String>,
    },

    /// Suggest promotions based on usage patterns
    Suggest {
        /// Min observations to trigger suggestion
        #[arg(long, default_value = "3")]
        threshold: u32,

        /// Min distinct sessions
        #[arg(long, default_value = "2")]
        sessions: u32,

        /// Output format: table, json, markdown
        #[arg(long, default_value = "table")]
        format: String,

        /// Rule patterns to filter output (exact, prefix, or substring match)
        #[arg(value_name = "PATTERN")]
        patterns: Vec<String>,
    },

    /// Session summary of permission activity
    Report {
        /// Session ID (default: latest)
        #[arg(long)]
        session: Option<String>,

        /// Output format: table, json, markdown
        #[arg(long, default_value = "table")]
        format: String,
    },

    /// Prune old events from the database
    Clean {
        /// Delete events older than N days
        #[arg(long, default_value = "90")]
        older_than: u32,

        /// Show what would be deleted without deleting
        #[arg(long)]
        dry_run: bool,
    },

    /// Verify hook installation and DB connectivity
    Check,
}
