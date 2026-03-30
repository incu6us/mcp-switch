use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "mcp-switch",
    version,
    about = "Enable/disable MCP servers for Claude Code"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[command(flatten)]
    pub scope: ScopeFlags,

    /// Preview changes without writing
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Args)]
#[group(multiple = false)]
pub struct ScopeFlags {
    /// Target project .mcp.json (default)
    #[arg(short, long)]
    pub project: bool,

    /// Target user ~/.claude/settings.json
    #[arg(short, long)]
    pub user: bool,

    /// Target a specific config file
    #[arg(short, long, value_name = "PATH")]
    pub file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Enable a server
    On {
        /// Server name (omit with --all)
        server: Option<String>,
        /// Enable all servers
        #[arg(long)]
        all: bool,
    },
    /// Disable a server
    Off {
        /// Server name (omit with --all)
        server: Option<String>,
        /// Disable all servers
        #[arg(long)]
        all: bool,
    },
    /// List all servers with status
    #[command(alias = "ls")]
    List,
    /// Show detailed server status
    Status {
        /// Server name (omit for all)
        server: Option<String>,
    },
    /// Manage profiles
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
}

#[derive(Subcommand)]
pub enum ProfileAction {
    /// Save current state as a named profile
    Save {
        /// Profile name
        name: String,
    },
    /// Apply a saved profile
    Apply {
        /// Profile name
        name: String,
    },
    /// List saved profiles
    List,
}
