use anyhow::{bail, Context, Result};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::ScopeFlags;

/// Returns true if the given path is a user-level (global) config file.
pub fn is_global_config(path: &Path) -> bool {
    let Some(home) = dirs::home_dir() else {
        return false;
    };
    for candidate in USER_CONFIG_CANDIDATES {
        if path == home.join(candidate) {
            return true;
        }
    }
    false
}

/// User-level config candidates, checked in priority order.
/// `~/.claude.json` is the primary global config where Claude Code stores MCP servers.
/// `~/.claude/settings.json` is an alternative location.
const USER_CONFIG_CANDIDATES: &[&str] = &[".claude.json", ".claude/settings.json"];

/// Resolve the user-level Claude Code config path.
///
/// Checks `~/.claude.json` first (primary global config), then
/// `~/.claude/settings.json`. On Windows also checks `%APPDATA%\claude\`.
pub fn resolve_user_config() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;

    for candidate in USER_CONFIG_CANDIDATES {
        let path = home.join(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    // Windows fallback: %APPDATA%\claude\settings.json
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            let alt = PathBuf::from(appdata).join("claude").join("settings.json");
            if alt.exists() {
                return Ok(alt);
            }
        }
    }

    // Nothing exists yet — default to ~/.claude.json
    Ok(home.join(".claude.json"))
}

/// Project-level config file candidates, checked in priority order.
const PROJECT_CONFIG_CANDIDATES: &[&str] = &[".mcp.json", ".claude/settings.json"];

pub fn resolve_target(flags: &ScopeFlags) -> Result<PathBuf> {
    if let Some(ref path) = flags.file {
        return Ok(path.clone());
    }

    if flags.user {
        return resolve_user_config();
    }

    // Default: project-level config — walk up from cwd
    // Check both .mcp.json and .claude/settings.json at each level
    // Stop at home directory to avoid matching user-level configs
    let cwd = std::env::current_dir().context("could not determine current directory")?;
    let home = dirs::home_dir();
    let mut dir = cwd.as_path();
    loop {
        // Stop before checking the home directory — configs there are user-level
        if home.as_deref() == Some(dir) {
            break;
        }
        for candidate_name in PROJECT_CONFIG_CANDIDATES {
            let candidate = dir.join(candidate_name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    // Not found — default to .mcp.json in cwd (will be created on write)
    Ok(cwd.join(".mcp.json"))
}

pub fn load_config(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(serde_json::json!({"mcpServers": {}}));
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let value: Value = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse JSON in {}", path.display()))?;

    Ok(value)
}

pub fn save_config(path: &Path, value: &Value, dry_run: bool) -> Result<()> {
    let json = serde_json::to_string_pretty(value)? + "\n";

    if dry_run {
        eprintln!("Would write to {}:\n{}", path.display(), json);
        return Ok(());
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }

    // Atomic write: write to .tmp then rename
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, &json)
        .with_context(|| format!("failed to write {}", tmp_path.display()))?;
    fs::rename(&tmp_path, path)
        .with_context(|| format!("failed to rename {} to {}", tmp_path.display(), path.display()))?;

    Ok(())
}

pub fn get_mcp_servers(config: &Value) -> Option<&Map<String, Value>> {
    config.get("mcpServers")?.as_object()
}

pub fn get_mcp_servers_mut(config: &mut Value) -> &mut Map<String, Value> {
    if config.get("mcpServers").is_none() {
        config
            .as_object_mut()
            .unwrap()
            .insert("mcpServers".to_string(), Value::Object(Map::new()));
    }
    config
        .get_mut("mcpServers")
        .unwrap()
        .as_object_mut()
        .unwrap()
}

pub fn is_disabled(server_config: &Value) -> bool {
    server_config
        .get("disabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

pub fn server_type(server_config: &Value) -> &str {
    server_config
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("stdio")
}

pub fn server_summary(server_config: &Value) -> String {
    let stype = server_type(server_config);
    match stype {
        "http" | "sse" => {
            let url = server_config
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            format!("{}: {}", stype, url)
        }
        _ => {
            let cmd = server_config
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let args = server_config
                .get("args")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();
            if args.is_empty() {
                format!("stdio: {}", cmd)
            } else {
                format!("stdio: {} {}", cmd, args)
            }
        }
    }
}

pub fn validate_server_exists(servers: &Map<String, Value>, name: &str) -> Result<()> {
    if !servers.contains_key(name) {
        let available: Vec<&str> = servers.keys().map(|s| s.as_str()).collect();
        if available.is_empty() {
            bail!("server '{}' not found (no servers configured)", name);
        } else {
            bail!(
                "server '{}' not found. Available: {}",
                name,
                available.join(", ")
            );
        }
    }
    Ok(())
}
