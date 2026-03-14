<div align="center">
  <img src="public/logo.jpg" width="120" alt="AIO Coding Hub Logo" />

# AIO Coding Hub

**Local AI CLI Unified Gateway** — Route Claude Code / Codex / Gemini CLI through a single entry point

[![Release](https://img.shields.io/github/v/release/dyndynjyxa/aio-coding-hub?style=flat-square)](https://github.com/dyndynjyxa/aio-coding-hub/releases)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20macOS%20|%20Linux-lightgrey?style=flat-square)](#installation)

[简体中文](./README.md) | English

</div>

> **Credits** — Inspired by [cc-switch](https://github.com/farion1231/cc-switch), [claude-code-hub](https://github.com/ding113/claude-code-hub), and [code-switch-R](https://github.com/Rogers-F/code-switch-R).

---

## Why?

| Problem | How AIO Coding Hub Solves It |
|---------|------------------------------|
| Each CLI needs separate API config | **Unified gateway** — all CLIs route through `127.0.0.1` |
| Upstream goes down, requests fail | **Smart failover** — auto-switch providers with circuit breaker |
| No idea how many tokens or how much it costs | **Full observability** — trace, usage stats, cost estimation |
| Different projects need different Prompts / MCP configs | **Workspace isolation** — per-project CLI config, one-click switch |

---

## Screenshots

### Home — Heatmap, usage trends, active sessions, request logs

![Home](public/screenshots/home.png)

### Usage — Token stats, cache hit rate, latency, cost leaderboard

![Usage](public/screenshots/usage.png)

### Model Validation — Multi-dimensional channel verification

![Model Validation](public/screenshots/modelValidate.png)

---

## Features

### Gateway Proxy

- Single entry point for Claude Code / Codex / Gemini CLI
- Custom model name mapping
- Auto-fix for SSE / JSON responses

### Smart Routing & Resilience

- Multi-provider priority ordering + automatic failover
- Circuit breaker (configurable threshold & recovery time)
- Sticky session for consistent provider routing

### Usage & Observability

- Token usage analytics (by CLI / provider / model)
- Cost estimation + auto-synced model pricing
- Request trace & real-time console logs
- Heatmap & cache trend charts

### Workspace Management

- Per-project isolation for Prompts, MCP, and Skill configs
- Workspace compare, clone, switch & rollback
- Auto-sync configs to each CLI

### Skill Market

- Discover and install Skills from Git repositories
- Repository management, filtering, and sorting
- Batch management linked to workspaces

### CLI Management

- Direct editing of Claude Code settings
- CodeMirror editor for Codex config.toml
- Environment variable conflict detection
- Local session history browser (project → session → messages)

### Model Validation

- Multi-dimensional validation templates (token truncation, Extended Thinking, etc.)
- Cross-provider signature verification
- Batch validation + history

### More

- Auto-update, autostart, single instance
- Data import / export / reset
- WSL support

---

## Installation

### Download from Releases (Recommended)

Go to [Releases](https://github.com/dyndynjyxa/aio-coding-hub/releases) and download for your platform:

| Platform | Package |
|----------|---------|
| **Windows** | `.exe` (NSIS) or `.msi` |
| **macOS** | `.dmg` |
| **Linux** | `.deb` / `.AppImage` |

<details>
<summary>macOS security note</summary>

If you see "can't be opened / unverified developer":

```bash
sudo xattr -cr /Applications/"AIO Coding Hub.app"
```

</details>

### Build from Source

<details>
<summary>Prerequisites</summary>

**General:** Node.js 18+, pnpm, Rust 1.90+

**Windows:** [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (select "Desktop development with C++")

**macOS:** `xcode-select --install`

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

</details>

```bash
git clone https://github.com/dyndynjyxa/aio-coding-hub.git
cd aio-coding-hub
pnpm install

# Development
pnpm tauri:dev

# Build (current platform)
pnpm tauri:build

# Platform-specific
pnpm tauri:build:mac:arm64       # macOS Apple Silicon
pnpm tauri:build:mac:x64         # macOS Intel
pnpm tauri:build:mac:universal   # macOS Universal
pnpm tauri:build:win:x64         # Windows x64
pnpm tauri:build:win:arm64       # Windows ARM64
pnpm tauri:build:linux:x64       # Linux x64
```

---

## Quick Start

```
1. Providers page → Add upstream (official API / self-hosted proxy / company gateway)
2. Home page → Toggle "Proxy" switch for target CLI
3. Run CLI in terminal → View trace & stats in Console / Usage page
```

Verify the gateway is running:

```bash
curl http://127.0.0.1:37123/health
# {"status":"ok"}
```

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| **Frontend** | React 19 · TypeScript · Tailwind CSS · Vite |
| **State** | TanStack Query · React Hooks |
| **Desktop** | Tauri 2 |
| **Backend** | Rust · Axum (HTTP Gateway) |
| **Database** | SQLite (rusqlite) |
| **Testing** | Vitest · Testing Library · MSW · Cargo Test |

---

## Quality Assurance

```bash
pnpm check:precommit       # Quick pre-commit (frontend + Rust check)
pnpm check:precommit:full  # Full check (formatting + clippy)
pnpm check:prepush         # Coverage + backend tests + clippy
pnpm test:unit              # Frontend unit tests
pnpm tauri:test             # Backend tests
```

---

## Not Designed For

- Public deployment / remote access / multi-tenant
- Enterprise RBAC

> This is a **local desktop tool + local gateway**. All data stays on your machine.

---

## Contributing

Issues and PRs welcome! We follow [Conventional Commits](https://www.conventionalcommits.org/).

```bash
feat(ui): add usage heatmap
fix(gateway): handle timeout correctly
docs: update installation guide
```

---

## License

[MIT License](LICENSE)

---

[![Stargazers over time](https://starchart.cc/dyndynjyxa/aio-coding-hub.svg?variant=adaptive)](https://starchart.cc/dyndynjyxa/aio-coding-hub)
