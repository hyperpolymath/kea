// SPDX-License-Identifier: AGPL-3.0-or-later

//! Kea-Bivouac — The Command Authority (CLI).
//!
//! This is the primary binary for the Bivouac controller. It provides 
//! the administrative interface for managing nomadic deployments and 
//! executing response playbooks within the Kea ecosystem.

use std::path::PathBuf;
use anyhow::Context;
use clap::{Parser, Subcommand};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use kea_bivouac::{
    playbook::{self, PlaybookExecutor},
    Config,
};

/// CLI SCHEMA: Defines the global options and subcommands.
#[derive(Parser, Debug)]
#[command(name = "bivouac")]
struct Cli {
    #[arg(short, long, default_value = "bivouac.toml")]
    config: PathBuf,
    #[arg(short, long)]
    verbose: bool,
    #[arg(long)]
    dry_run: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// EXECUTE: Manually runs a playbook by name or path.
    Execute { playbook: String },
    /// TRIGGER: Runs a playbook associated with a specific event type.
    Trigger { trigger_type: String },
    /// VALIDATE: Performs a dry-run check of a playbook's syntax and actions.
    Validate { playbook: PathBuf },
    /// INITIALIZE: Generates a default bivouac.toml configuration.
    Init { #[arg(short, long)] force: bool },
    // ... [Other commands: Config, List, Version]
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // PARSING: Ingest user arguments.
    let cli = Cli::parse();

    // OBSERVABILITY: Setup tracing based on verbosity level.
    let filter = if cli.verbose { EnvFilter::new("info") } else { EnvFilter::new("warn") };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // DISPATCH: Execute the requested administrative action.
    match cli.command {
        Commands::Init { force } => init_config(&cli.config, force).await,
        Commands::Execute { playbook } => execute_playbook(&cli.config, &playbook, cli.dry_run).await,
        Commands::Trigger { trigger_type } => trigger_playbook(&cli.config, &trigger_type, cli.dry_run).await,
        // ... [Remaining handlers]
        _ => Ok(())
    }
}
