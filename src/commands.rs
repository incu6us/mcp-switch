use anyhow::{bail, Result};
use serde_json::Value;
use std::path::Path;

use crate::config;
use crate::output;

pub fn cmd_on(
    path: &Path,
    server: Option<&str>,
    all: bool,
    dry_run: bool,
    json: bool,
) -> Result<()> {
    if !all && server.is_none() {
        bail!("specify a server name or use --all");
    }

    let mut cfg = config::load_config(path)?;

    if all {
        set_all(&mut cfg, false)?;
    } else {
        let name = server.unwrap();
        set_one(&mut cfg, name, false)?;
    }

    config::save_config(path, &cfg, dry_run)?;

    if !dry_run {
        if json {
            output::print_change_json(server, all, "enabled");
        } else {
            output::print_change(server, all, "enabled");
        }
    }

    Ok(())
}

pub fn cmd_off(
    path: &Path,
    server: Option<&str>,
    all: bool,
    dry_run: bool,
    json: bool,
) -> Result<()> {
    if !all && server.is_none() {
        bail!("specify a server name or use --all");
    }

    let mut cfg = config::load_config(path)?;

    if all {
        set_all(&mut cfg, true)?;
    } else {
        let name = server.unwrap();
        set_one(&mut cfg, name, true)?;
    }

    config::save_config(path, &cfg, dry_run)?;

    if !dry_run {
        if json {
            output::print_change_json(server, all, "disabled");
        } else {
            output::print_change(server, all, "disabled");
        }
    }

    Ok(())
}

pub fn cmd_list(path: &Path, json: bool) -> Result<()> {
    let cfg = config::load_config(path)?;
    let servers = config::get_mcp_servers(&cfg);

    if json {
        output::print_list_json(servers);
    } else {
        output::print_list(servers, path);
    }

    Ok(())
}

pub fn cmd_status(path: &Path, server: Option<&str>, json: bool) -> Result<()> {
    let cfg = config::load_config(path)?;
    let servers = config::get_mcp_servers(&cfg);

    if let Some(name) = server {
        if let Some(servers_map) = servers {
            config::validate_server_exists(servers_map, name)?;
        } else {
            bail!("server '{}' not found (no servers configured)", name);
        }
    }

    if json {
        output::print_status_json(servers, server);
    } else {
        output::print_status(servers, server, path);
    }

    Ok(())
}

fn set_one(cfg: &mut Value, name: &str, disabled: bool) -> Result<()> {
    let servers = config::get_mcp_servers_mut(cfg);
    config::validate_server_exists(servers, name)?;

    let entry = servers.get_mut(name).unwrap().as_object_mut().unwrap();
    if disabled {
        entry.insert("disabled".to_string(), Value::Bool(true));
    } else {
        entry.remove("disabled");
    }
    Ok(())
}

fn set_all(cfg: &mut Value, disabled: bool) -> Result<()> {
    let servers = config::get_mcp_servers_mut(cfg);
    let keys: Vec<String> = servers.keys().cloned().collect();

    if keys.is_empty() {
        bail!("no servers configured");
    }

    for key in keys {
        let entry = servers.get_mut(&key).unwrap().as_object_mut().unwrap();
        if disabled {
            entry.insert("disabled".to_string(), Value::Bool(true));
        } else {
            entry.remove("disabled");
        }
    }
    Ok(())
}
