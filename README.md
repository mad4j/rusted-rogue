# rusted-rogue

Rust port of [Rogue](https://en.wikipedia.org/wiki/Rogue_(video_game)) — the classic 1980 dungeon-crawler roguelike.

**Status: RC1** — all 25 backlog tasks complete, full quality-gate suite passing.

---

## Build & run

```sh
# Release build
cargo build --release

# Run
cargo run --release

# Run with a fixed seed (deterministic game)
RUSTED_ROGUE_SEED=12345 cargo run --release
```

### Map Viewer (Debugging Tool)

Visualize generated dungeon maps with textual output:

```sh
# Single map with seed 42
cargo run --bin map_viewer 42

# View 10 consecutive levels with same seed and increasing dungeon depth (1..10)
cargo run --bin map_viewer 42 10

# Deterministic output for visual regression testing
cargo run --bin map_viewer 12345 5 > maps_baseline.txt
```

Map symbols:
- `·` = floor
- `─` = horizontal wall
- `│` = vertical wall  
- `#` = tunnel (corridor)
- `+` = door
- `^` = stairs
- `.` = empty

**Analysis:** See [bl-map-generator-analysis.md](automation/bl-map-generator-analysis.md) for detailed comparison of C vs Rust map generation algorithms.

Requires **Rust 1.70+** and a terminal with **ANSI color support**
(Windows 10 1809+, any modern Linux/macOS terminal).

---

## Development

```sh
cargo fmt          # format
cargo clippy       # lint
cargo test         # 52 unit + 1 smoke tests
```

All three gates must pass before merging.

---

## Architecture

| Module | Responsibility |
|--------|---------------|
| `core_types` | Constants, bitflags, `Position`, `TileFlags`, `ObjectFlags` |
| `rng` | Deterministic seed-based RNG (parity-validated vs C original) |
| `world_gen` | Dungeon and room generation |
| `actors` | Monster spawn, turns, movement, combat |
| `inventory_items` | Inventory state: pick/drop/equip/use/wand/ring/throw |
| `game_loop` | Command dispatch and turn orchestration |
| `ui_terminal` | crossterm rendering and input mapping |
| `persistence` | Save/load (JSON) and high-score storage |
| `platform` | OS adapter stub (signals, user identity — see known limits) |

---

## Release notes & known limits

See [docs/release-rc1.md](docs/release-rc1.md) for the full RC1 checklist,
known limits, and post-parity roadmap.

---

## Original source

The original C source is preserved in `original/` for reference and parity testing.
