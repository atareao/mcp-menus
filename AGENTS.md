# mcp-menus

## Project

MCP Menu server — assigns recipes from `mcp-recipes` to dates. Each entry stores a date, lunch recipe (ID + name), and dinner recipe (ID + name). SQLite-backed.

## Build & run

```sh
cargo run                          # stdio transport, DB at ./menus.db
MENUS_DB_PATH=/tmp/m.db cargo run  # custom DB path
MCP_TRANSPORT=http cargo run       # HTTP on 127.0.0.1:3020
RUST_LOG=debug cargo run

cargo run -- --clear               # delete all menu entries
```

No test / lint / fmt suite configured yet.

## Architecture

Single binary, single crate. All MCP tool handlers live in `main.rs` (4 tools, no separate module needed):

- `src/main.rs` — entry point, MCP handler dispatch, all tool structs
- `src/db.rs` — SQLite via `sqlx`, `init_db`, all queries
- `src/models.rs` — data structs

## Database

SQLite via `sqlx`. Table created at startup (`CREATE TABLE IF NOT EXISTS`):

| Column       | Type | Notes |
|--------------|------|-------|
| `date`       | TEXT | PK, `"2026-05-23"` |
| `comida_id`  | TEXT | recipe UUID from `mcp-recipes` |
| `comida_name`| TEXT | denormalized recipe title |
| `cena_id`    | TEXT | |
| `cena_name`  | TEXT | |

Path from `MENUS_DB_PATH` env var (default: `menus.db`).

## MCP tools

- `set_menu(date, comida_id, comida_name, cena_id, cena_name)` — upsert a day's meals
- `get_menu(date)` — get single day
- `list_menus(start_date?, end_date?)` — range query, defaults to next 7 days
- `clear_day(date)` — delete entry for a date

## Conventions

- Edition 2024, `rust-mcp-sdk` 0.9
- `async-trait`, state via `Arc<Mutex<AppState>>` with `SqlitePool`
- Tool structs with `call_tool(&self, state)` method
- Add new tool: add handler fn to `main.rs`, register in `main.rs` dispatch via `tool_box!` macro and match arm