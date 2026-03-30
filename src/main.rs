mod cli;
mod commands;
mod config;
mod output;
mod profile;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, ProfileAction};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = config::resolve_target(&cli.scope)?;

    let is_mutation = matches!(
        cli.command,
        Command::On { .. }
            | Command::Off { .. }
            | Command::Profile {
                action: ProfileAction::Apply { .. }
            }
    );

    match cli.command {
        Command::On { server, all } => {
            commands::cmd_on(&path, server.as_deref(), all, cli.dry_run, cli.json)?;
        }
        Command::Off { server, all } => {
            commands::cmd_off(&path, server.as_deref(), all, cli.dry_run, cli.json)?;
        }
        Command::List => {
            commands::cmd_list(&path, cli.json)?;
        }
        Command::Status { server } => {
            commands::cmd_status(&path, server.as_deref(), cli.json)?;
        }
        Command::Profile { action } => match action {
            ProfileAction::Save { name } => {
                profile::cmd_save(&path, &name)?;
            }
            ProfileAction::Apply { name } => {
                profile::cmd_apply(&path, &name, cli.dry_run, cli.json)?;
            }
            ProfileAction::List => {
                profile::cmd_list(cli.json)?;
            }
        },
    }

    if is_mutation && !cli.dry_run && config::is_global_config(&path) {
        output::print_global_config_hint();
    }

    Ok(())
}
