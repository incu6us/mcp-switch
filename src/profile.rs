use anyhow::{bail, Context, Result};
use owo_colors::OwoColorize;
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

use crate::config;

fn profiles_dir() -> Result<std::path::PathBuf> {
    let config_dir = dirs::config_dir().context("could not determine config directory")?;
    Ok(config_dir.join("mcp-switch").join("profiles"))
}

pub fn cmd_save(config_path: &Path, name: &str) -> Result<()> {
    let cfg = config::load_config(config_path)?;
    let servers = config::get_mcp_servers(&cfg);

    let state: Map<String, Value> = match servers {
        Some(s) => s
            .iter()
            .map(|(k, v)| {
                let disabled = config::is_disabled(v);
                (k.clone(), serde_json::json!({ "disabled": disabled }))
            })
            .collect(),
        None => Map::new(),
    };

    let profile = serde_json::json!({
        "source": config_path.to_string_lossy(),
        "servers": state,
    });

    let dir = profiles_dir()?;
    fs::create_dir_all(&dir)?;

    let path = dir.join(format!("{}.json", name));
    let json = serde_json::to_string_pretty(&profile)? + "\n";
    fs::write(&path, json)?;

    println!("Profile {} saved ({} servers)", name.bold(), state.len());
    Ok(())
}

pub fn cmd_apply(config_path: &Path, name: &str, dry_run: bool, json_output: bool) -> Result<()> {
    let dir = profiles_dir()?;
    let path = dir.join(format!("{}.json", name));

    if !path.exists() {
        bail!("profile '{}' not found", name);
    }

    let profile_content = fs::read_to_string(&path)?;
    let profile: Value = serde_json::from_str(&profile_content)?;

    let profile_servers = profile
        .get("servers")
        .and_then(|v| v.as_object())
        .context("invalid profile format")?;

    let mut cfg = config::load_config(config_path)?;
    let servers = config::get_mcp_servers_mut(&mut cfg);

    let mut applied = 0;
    let mut skipped = Vec::new();

    for (server_name, state) in profile_servers {
        if !servers.contains_key(server_name) {
            skipped.push(server_name.as_str());
            continue;
        }

        let disabled = state
            .get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let entry = servers
            .get_mut(server_name)
            .unwrap()
            .as_object_mut()
            .unwrap();

        if disabled {
            entry.insert("disabled".to_string(), Value::Bool(true));
        } else {
            entry.remove("disabled");
        }
        applied += 1;
    }

    config::save_config(config_path, &cfg, dry_run)?;

    if !dry_run {
        if json_output {
            let result = serde_json::json!({
                "profile": name,
                "applied": applied,
                "skipped": skipped,
            });
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        } else {
            println!("Profile {} applied ({} servers)", name.bold(), applied);
            if !skipped.is_empty() {
                eprintln!(
                    "{}",
                    format!("Skipped (not in config): {}", skipped.join(", ")).yellow()
                );
            }
        }
    }

    Ok(())
}

pub fn cmd_list(json_output: bool) -> Result<()> {
    let dir = profiles_dir()?;

    if !dir.exists() {
        if json_output {
            println!("[]");
        } else {
            println!("No profiles saved");
        }
        return Ok(());
    }

    let mut profiles: Vec<String> = Vec::new();

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                profiles.push(name.to_string());
            }
        }
    }

    profiles.sort();

    if json_output {
        println!("{}", serde_json::to_string_pretty(&profiles).unwrap());
    } else if profiles.is_empty() {
        println!("No profiles saved");
    } else {
        for name in &profiles {
            // Load profile to show server count
            let path = dir.join(format!("{}.json", name));
            let count = fs::read_to_string(&path)
                .ok()
                .and_then(|c| serde_json::from_str::<Value>(&c).ok())
                .and_then(|v| v.get("servers")?.as_object().map(|s| s.len()))
                .unwrap_or(0);

            println!("  {}  {}", name.bold(), format!("({} servers)", count).dimmed());
        }
    }

    Ok(())
}
