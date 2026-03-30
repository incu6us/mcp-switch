use owo_colors::OwoColorize;
use serde_json::{Map, Value};
use std::path::Path;

pub fn print_list(servers: Option<&Map<String, Value>>, path: &Path) {
    println!("{}", format!("Config: {}", path.display()).dimmed());

    let servers = match servers {
        Some(s) if !s.is_empty() => s,
        _ => {
            println!("  No servers configured");
            return;
        }
    };

    let max_name = servers.keys().map(|k| k.len()).max().unwrap_or(0);

    for (name, cfg) in servers {
        let disabled = crate::config::is_disabled(cfg);
        let status = if disabled {
            "disabled".red().to_string()
        } else {
            "enabled".green().to_string()
        };
        let summary = crate::config::server_summary(cfg);

        println!(
            "  {:<width$}  {:>10}  {}",
            name.bold(),
            status,
            summary.dimmed(),
            width = max_name
        );
    }
}

pub fn print_list_json(servers: Option<&Map<String, Value>>) {
    let entries: Vec<Value> = match servers {
        Some(s) => s
            .iter()
            .map(|(name, cfg)| {
                serde_json::json!({
                    "name": name,
                    "disabled": crate::config::is_disabled(cfg),
                    "type": crate::config::server_type(cfg),
                    "summary": crate::config::server_summary(cfg),
                })
            })
            .collect(),
        None => vec![],
    };
    println!("{}", serde_json::to_string_pretty(&entries).unwrap());
}

pub fn print_status(servers: Option<&Map<String, Value>>, filter: Option<&str>, path: &Path) {
    println!("{}", format!("Config: {}", path.display()).dimmed());

    let servers = match servers {
        Some(s) if !s.is_empty() => s,
        _ => {
            println!("  No servers configured");
            return;
        }
    };

    let iter: Box<dyn Iterator<Item = (&String, &Value)>> = if let Some(name) = filter {
        Box::new(servers.iter().filter(move |(k, _)| k.as_str() == name))
    } else {
        Box::new(servers.iter())
    };

    for (name, cfg) in iter {
        let disabled = crate::config::is_disabled(cfg);
        let status = if disabled {
            "disabled".red().to_string()
        } else {
            "enabled".green().to_string()
        };

        println!("\n  {} [{}]", name.bold(), status);

        let obj = cfg.as_object().unwrap();
        for (key, val) in obj {
            if key == "disabled" {
                continue;
            }
            let display = match val {
                Value::String(s) => s.clone(),
                Value::Array(arr) => {
                    let items: Vec<String> = arr.iter().map(|v| format!("{}", v)).collect();
                    items.join(", ")
                }
                Value::Object(map) => {
                    let keys: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
                    format!("{{{}}}", keys.join(", "))
                }
                other => format!("{}", other),
            };
            println!("    {}: {}", key.dimmed(), display);
        }
    }
}

pub fn print_status_json(servers: Option<&Map<String, Value>>, filter: Option<&str>) {
    let entries: Vec<Value> = match servers {
        Some(s) => s
            .iter()
            .filter(|(k, _)| filter.is_none_or(|f| k.as_str() == f))
            .map(|(name, cfg)| {
                let mut entry = cfg.clone();
                entry
                    .as_object_mut()
                    .unwrap()
                    .insert("name".to_string(), Value::String(name.clone()));
                entry
            })
            .collect(),
        None => vec![],
    };
    println!("{}", serde_json::to_string_pretty(&entries).unwrap());
}

pub fn print_change(server: Option<&str>, all: bool, state: &str) {
    let colored_state = if state == "enabled" {
        state.green().to_string()
    } else {
        state.red().to_string()
    };

    if all {
        println!("All servers {}", colored_state);
    } else if let Some(name) = server {
        println!("{} {}", name.bold(), colored_state);
    }
}

pub fn print_change_json(server: Option<&str>, all: bool, state: &str) {
    let result = serde_json::json!({
        "server": server,
        "all": all,
        "state": state,
    });
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}

pub fn print_global_config_hint() {
    eprintln!(
        "{}",
        "Note: restart active Claude Code sessions to apply global config changes".yellow()
    );
}
