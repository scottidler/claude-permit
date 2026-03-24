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

        /// Filter by risk tier: safe, moderate, dangerous
        #[arg(long)]
        risk: Option<String>,
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
