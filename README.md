# mcp-switch

[![CI](https://github.com/incu6us/mcp-switch/actions/workflows/ci.yml/badge.svg)](https://github.com/incu6us/mcp-switch/actions/workflows/ci.yml)
[![Release](https://github.com/incu6us/mcp-switch/actions/workflows/release.yml/badge.svg)](https://github.com/incu6us/mcp-switch/actions/workflows/release.yml)
[![GitHub release](https://img.shields.io/github/v/release/incu6us/mcp-switch)](https://github.com/incu6us/mcp-switch/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Homebrew](https://img.shields.io/badge/homebrew-tap-orange)](https://github.com/incu6us/homebrew-tap)

A fast CLI tool to enable and disable [MCP](https://modelcontextprotocol.io/) server integrations for [Claude Code](https://docs.anthropic.com/en/docs/claude-code) from outside the editor.

## Motivation

Claude Code supports MCP servers for extending its capabilities — connecting to databases, APIs, issue trackers, and more. These servers are configured in JSON files (`.mcp.json` per project, `~/.claude.json` globally), and each can be toggled via a `"disabled"` flag.

But there's no built-in way to quickly switch servers on and off from the terminal. If you work across multiple projects with different MCP setups, or need to temporarily disable noisy integrations, you're stuck hand-editing JSON.

**mcp-switch** solves this with a single command:

```bash
mcp-switch off atlassian    # done
```

## Installation

### Homebrew

```bash
brew install incu6us/homebrew-tap/mcp-switch
```

### Download binary

Pre-built binaries for Linux, macOS, and Windows are available on the [Releases](https://github.com/incu6us/mcp-switch/releases) page.

### With cargo

```bash
cargo install mcp-switch
```

### From source

```bash
git clone https://github.com/incu6us/mcp-switch.git
cd mcp-switch
cargo install --path .
```

## Usage

### List servers

```bash
mcp-switch list
```

```
Config: /home/user/project/.mcp.json
  atlassian  enabled   stdio: npx @anthropic/mcp-atlassian
  loki       disabled  stdio: node loki-server.js
  greptile   enabled   http: https://api.greptile.com/mcp
```

### Enable / disable

```bash
mcp-switch on loki
mcp-switch off atlassian
```

### Bulk operations

```bash
mcp-switch off --all
mcp-switch on --all
```

### Detailed status

```bash
mcp-switch status greptile
```

```
Config: /home/user/project/.mcp.json

  greptile [enabled]
    type: http
    url: https://api.greptile.com/mcp
```

### Profiles

Save the current enabled/disabled state as a named profile and restore it later:

```bash
mcp-switch profile save dev       # save current state
mcp-switch off --all              # disable everything
mcp-switch profile apply dev      # restore saved state
mcp-switch profile list           # list saved profiles
```

Profiles are stored in `~/.config/mcp-switch/profiles/`.

## Config scope

By default, mcp-switch targets the project-level config (searching upward from the current directory, stopping at the home directory to avoid matching user-level configs). Use flags to target other locations:

| Flag | Target |
|---|---|
| `-p`, `--project` | Project config (default) |
| `-u`, `--user` | User-level global config |
| `-f`, `--file <path>` | Any specific config file |

### Config file locations

**Project-level** (checked in order, walking up from cwd):

| Priority | Path |
|---|---|
| 1 | `.mcp.json` |
| 2 | `.claude/settings.json` |

**User-level** (`--user` flag), checked in order:

| Priority | macOS / Linux | Windows | Notes |
|---|---|---|---|
| 1 | `~/.claude.json` | `%USERPROFILE%\.claude.json` | Primary global config |
| 2 | `~/.claude/settings.json` | `%USERPROFILE%\.claude\settings.json` | Alternative location |
| 3 | — | `%APPDATA%\claude\settings.json` | Windows fallback |

The project-level search walks up from the current directory but stops at the home directory — configs there (`~/.claude.json`, `~/.claude/settings.json`) are user-level and only matched with `--user`.

## Additional flags

| Flag | Description |
|---|---|
| `--dry-run` | Preview what would change without writing to disk |
| `--json` | Output in JSON for scripting and automation |

### Examples

```bash
# Preview changes
mcp-switch off atlassian --dry-run

# Script-friendly output
mcp-switch list --json

# Manage user-level servers
mcp-switch list --user
mcp-switch off my-global-server --user

# Point to a specific config file
mcp-switch list --file /path/to/settings.json
```

## How it works

MCP server configs live in standard JSON files. Each server entry can have a `"disabled": true` field:

```json
{
  "mcpServers": {
    "my-server": {
      "command": "node",
      "args": ["server.js"],
      "disabled": true
    }
  }
}
```

mcp-switch reads the config, flips the `disabled` flag (or removes it entirely when enabling, since the default state is enabled), and writes it back. Writes are atomic (write to temp file, then rename) to prevent corruption.

> **Note:** Changes to project-level `.mcp.json` are picked up on the next tool call. Changes to the global `~/.claude.json` require restarting the Claude Code session to take effect.

## License

MIT
