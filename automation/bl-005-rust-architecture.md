# BL-005 - Rust module architecture and API boundaries

## Objective
Define a Rust module architecture mapped from legacy C modules and enforce strict domain-vs-IO boundaries.

## Target module map

### Core domain (pure logic)
- `core_types`
  - Source mapping: `rogue.h`
  - Responsibilities: constants, enums, bitflags, coordinates, core structs.
- `rng`
  - Source mapping: `random.c`
  - Responsibilities: deterministic RNG and seed control.
- `world_gen`
  - Source mapping: `level.c`, `room.c`, `trap.c`, `object.c`
  - Responsibilities: level generation, room topology, tile placement.
- `actors`
  - Source mapping: `monster.c`, `move.c`, `hit.c`, `spec_hit.c`
  - Responsibilities: actor turns, movement rules, combat resolution.
- `inventory_items`
  - Source mapping: `pack.c`, `inventory.c`, `use.c`, `ring.c`, `throw.c`, `zap.c`
  - Responsibilities: inventory state transitions and item effects.

### Application orchestration
- `game_loop`
  - Source mapping: `main.c`, `play.c`
  - Responsibilities: command dispatch and turn orchestration.

### Platform and adapters (impure)
- `ui_terminal`
  - Source mapping: `curses.c`, `message.c`
  - Responsibilities: rendering/input adapter for terminal backend.
- `persistence`
  - Source mapping: `save.c`, `score.c`
  - Responsibilities: save/load and high score storage abstraction.
- `platform`
  - Source mapping: `machdep.c`
  - Responsibilities: signals, time, user identity, process/env/fs metadata wrappers.

## API boundary rules

### Rule 1: One-way dependency direction
- `core_types`, `rng`, `world_gen`, `actors`, `inventory_items` cannot depend on terminal, filesystem, or OS APIs.
- `game_loop` depends on domain modules plus trait-based interfaces.
- `ui_terminal`, `persistence`, and `platform` implement interfaces consumed by `game_loop`.

### Rule 2: Trait-based adapter contracts
- `InputPort`: read command events.
- `RenderPort`: render frame/messages/stats.
- `SavePort`: save/load snapshots.
- `ClockPort`, `SeedPort`, `SignalPort`, `UserPort`, `FsMetaPort`: platform wrappers.

### Rule 3: State ownership
- `GameState` is the single mutable root passed to systems.
- Domain systems receive `&mut GameState` and deterministic dependencies as explicit parameters.

## Initial crate layout

```text
src/
  main.rs
  game_loop/
  core_types/
  rng/
  world_gen/
  actors/
  inventory_items/
  ui_terminal/
  persistence/
  platform/
```

## Acceptance mapping
- DoD item 1 satisfied: module-to-responsibility map defined.
- DoD item 2 satisfied: explicit domain-vs-IO boundary rules specified.
