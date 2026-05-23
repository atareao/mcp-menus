<div align="center">

# MCP Menu Server

[![License](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-orange?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-0.9-6b5ce7?style=flat-square)](https://modelcontextprotocol.io)
[![SQLite](https://img.shields.io/badge/SQLite-003b57?style=flat-square&logo=sqlite&logoColor=white)](https://www.sqlite.org/)

A Model Context Protocol (MCP) server for managing daily menus, backed by SQLite.

[Features](#features) • [Getting Started](#getting-started) • [Available Tools](#available-tools) • [Configuration](#configuration) • [Docker](#docker)

</div>

## Overview

MCP Menu is a lightweight server that assigns recipes from [mcp-recipes](https://github.com/atareao/mcp-recipes) to daily lunch and dinner slots. Each day stores a date, lunch recipe (ID + name), and dinner recipe (ID + name) in a local SQLite database.

The server runs as a single binary with zero external dependencies beyond the SQLite file, and supports both **stdio** and **HTTP** transports.

## Features

- **4 MCP Tools** — assign, query, and clear daily menus
- **Date Range Queries** — list menus for any period (defaults to next 7 days)
- **Upsert Semantics** — set or update individual meals without overwriting the other
- **Dual Transport** — stdio (default for MCP clients) or HTTP (for web/API access)
- **Zero Config** — runs out of the box with a local SQLite file

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (Edition 2024)
- [Cargo](https://doc.rust-lang.org/cargo/)

### Build and Run

```bash
# Build
cargo build --release

# Run with stdio transport (default)
cargo run

# Run with HTTP transport
MCP_TRANSPORT=http cargo run
```

### Quick Test

Once running with HTTP transport, the server listens on `127.0.0.1:3020`:

```bash
# Initialize session
SESSION=$(curl -s -i -X POST http://127.0.0.1:3020/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}' | \
  grep -i "^mcp-session-id:" | tr -d '\r' | cut -d' ' -f2)

# Assign a recipe to today's lunch
curl -X POST http://127.0.0.1:3020/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "Mcp-Session-Id: $SESSION" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"set_menu","arguments":{"date":"2026-05-23","comida_id":"abc-123","comida_name":"Paella","cena_id":"def-456","cena_name":"Tortilla"}}}'
```

## Available Tools

| Tool | Parameters | Description |
|---|---|---|
| `set_menu` | `date` (req), `comida_id`, `comida_name`, `cena_id`, `cena_name` | Assign recipes to a date's lunch and/or dinner. All fields except `date` are optional — omitted fields preserve existing values. |
| `get_menu` | `date` (req) | Get the full menu entry for a specific date. |
| `list_menus` | `start_date`, `end_date` | List menu entries in a date range. Defaults to today through the next 7 days. |
| `clear_day` | `date` (req) | Delete the menu entry for a specific date. |

## Configuration

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `MENUS_DB_PATH` | `menus.db` | Path to the SQLite database file |
| `MCP_TRANSPORT` | `stdio` | Transport mode: `stdio` or `http` |
| `MCP_PORT` | `3020` | HTTP port (when using `http` transport) |
| `MCP_HOST` | `127.0.0.1` | HTTP bind address |
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |

### CLI Commands

```bash
# Clear all menu entries
cargo run -- --clear

# Custom database path
MENUS_DB_PATH=/tmp/menus.db cargo run

# HTTP on all interfaces, port 8080
MCP_TRANSPORT=http MCP_HOST=0.0.0.0 MCP_PORT=8080 cargo run

# Debug logging
RUST_LOG=debug cargo run
```

## Database

The server creates a single SQLite table on first run:

```sql
CREATE TABLE menu_entries (
    date TEXT PRIMARY KEY,
    comida_id TEXT,
    comida_name TEXT,
    cena_id TEXT,
    cena_name TEXT
);
```

- **date**: Date in `YYYY-MM-DD` format (primary key)
- **comida_id / comida_name**: Recipe UUID and title for lunch
- **cena_id / cena_name**: Recipe UUID and title for dinner

Recipe IDs reference the UUIDs from [mcp-recipes](https://github.com/atareao/mcp-recipes). Names are denormalized for self-contained queries.

## Project Structure

```
src/
├── main.rs     # Entry point, MCP handler, all 4 tool structs
├── db.rs       # SQLite pool, schema init, query functions
└── models.rs   # Data structs (MenuEntry, params)
```

## Docker

### Build

```bash
# Build the image (requires just)
podman build -t atareao/mcp-menus:latest .
```

### Run

```bash
# Run with stdio transport
podman run -it atareao/mcp-menus:latest

# Run with HTTP transport
podman run -p 3020:3020 -e MCP_TRANSPORT=http atareao/mcp-menus:latest

# Persist database to host
podman run -v $(pwd)/data:/data -e MENUS_DB_PATH=/data/menus.db atareao/mcp-menus:latest
```

> The container runs as non-root user `mcp` (UID 1000) for security.

## Tech Stack

- **[rust-mcp-sdk](https://crates.io/crates/rust-mcp-sdk)** v0.9 — MCP server framework
- **SQLite** via **sqlx** — embedded database with WAL mode
- **Tokio** — async runtime