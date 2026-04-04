# Map Generator — Feature Reference

Derived from `original/rogue-libc5-ncurses/rogue/level.c` and `rogue.h`.

---

## Grid layout

- The dungeon is a **3×3 grid of 9 slots** (MAXROOMS = 9), indexed 0–8.
- Screen size: 24 rows × 80 columns (DROWS × DCOLS).
- Row dividers: `ROW1 = 7`, `ROW2 = 15`. Column dividers: `COL1 = 26`, `COL2 = 52`.
- Slot bounds (top_row, bottom_row, left_col, right_col):

| Slot | Rows | Cols |
| ------ | ------ | ------ |
| 0 | MIN_ROW (1) .. ROW1-1 (6) | 0 .. COL1-1 (25) |
| 1 | MIN_ROW .. ROW1-1 | COL1+1 (27) .. COL2-1 (51) |
| 2 | MIN_ROW .. ROW1-1 | COL2+1 (53) .. DCOLS-1 (79) |
| 3 | ROW1+1 (8) .. ROW2-1 (14) | 0 .. COL1-1 |
| 4 | ROW1+1 .. ROW2-1 | COL1+1 .. COL2-1 |
| 5 | ROW1+1 .. ROW2-1 | COL2+1 .. DCOLS-1 |
| 6 | ROW2+1 (16) .. DROWS-2 (22) | 0 .. COL1-1 |
| 7 | ROW2+1 .. DROWS-2 | COL1+1 .. COL2-1 |
| 8 | ROW2+1 .. DROWS-2 | COL2+1 .. DCOLS-1 |

---

## Required room groups (`make_level`)

One of 6 patterns is chosen randomly; the 3 slots in the pattern are **guaranteed to exist**:

| Pattern ID | Slots | Description |
| ------------ | ------- | ------------- |
| 0 | 0, 1, 2 | top row |
| 1 | 3, 4, 5 | middle row |
| 2 | 6, 7, 8 | bottom row |
| 3 | 0, 3, 6 | left column |
| 4 | 1, 4, 7 | centre column |
| 5 | 2, 5, 8 | right column |

All other slots have a **40% chance of being skipped** (`rand_percent(40)`).

---

## Room drawing (`make_room`)

For each non-skipped slot:

- Random **height**: `get_rand(4, slot_height)`
- Random **width**: `get_rand(7, slot_width - 2)`
- Random **row/col offset** to position the room within the slot bounds
- Tile assignment:
  - `HORWALL` on top and bottom rows
  - `VERTWALL` on left and right columns (not corners)
  - `FLOOR` everywhere else inside

Room coordinates are stored in `rooms[rn]` including the slot bounds (used by maze slots too).

---

## Big Room (`BIG_ROOM`)

Triggered when `cur_level == party_counter` **and** `rand_percent(1)` (~1%).

- `party_counter` starts at `PARTY_TIME (10)` and is incremented by 10 each time a party room is used.
- A single oversized room is drawn:
  - Top row: `get_rand(MIN_ROW, MIN_ROW+5)`
  - Bottom row: `get_rand(DROWS-7, DROWS-2)`
  - Left col: `get_rand(0, 10)`
  - Right col: `get_rand(DCOLS-11, DCOLS-1)`
- No passages or mazes are generated on a big-room level.

---

## Maze generation (`add_mazes`)

Active from **level 2** onward.

- Maze probability per empty slot: `maze_percent = (cur_level * 5) / 4`; if `cur_level > 15`: `maze_percent += cur_level`.
- Iterates all 9 slots starting from a random offset; only fills `R_NOTHING` slots.
- Algorithm (`make_maze`): recursive DFS, moves **1 cell at a time**, checks **3 adjacent cells** in each direction to prevent corridors from touching (wider, more organic labyrinths).
  - Example UP check: `dungeon[r-1][c]`, `dungeon[r-1][c-1]`, `dungeon[r-1][c+1]`, `dungeon[r-2][c]` must all be non-TUNNEL.
- Direction shuffle: `rand_percent(33)` triggers 10 random swaps of the 4 directions before recursing.
- After drawing each maze: `hide_boxed_passage` applied `get_rand(0, 2)` times to hide some tunnel tiles.
- Maze slots are stored in `rooms[j]` with `is_room = R_MAZE` and the **same slot bounds**, so they participate in `connect_rooms` and `fill_it` exactly like regular rooms.

---

## Room connections (`connect_rooms`)

Only connects rooms that share the same row (`slot/3` equal) or same column (`slot%3` equal).

Connection order uses a shuffled `random_rooms[]` array. For each slot `i` in that order:

1. Try connect `i → i+1` (adjacent in same row)
2. Try connect `i → i+3` (adjacent in same column)
3. If slot `i+1` is `R_NOTHING`: try connect `i → i+2` (skip over empty slot); if successful mark `i+1` as `R_CROSS`
4. If slot `i+3` is `R_NOTHING`: try connect `i → i+6` (skip over empty slot); if successful mark `i+3` as `R_CROSS`
5. After each step, check `is_all_connected()`; if true, stop early.

### Passage drawing (`draw_simple_passage`)

L-shaped tunnel between two points:

- Horizontal then vertical segment (or vertical then horizontal for UP/DOWN direction)
- Random **middle bend point** between the two endpoints
- `do { draw_simple_passage(...) } while (rand_percent(4))` — ~4% chance of drawing an additional passage between the same two points
- After drawing: `rand_percent(HIDE_PERCENT=12)` → call `hide_boxed_passage` once to hide part of the passage

### Door placement (`put_door`)

- For `R_ROOM` slots: tile is set to `DOOR` on the appropriate wall
- For `R_MAZE` slots: `wall_width = 0`, so the door can be placed at the outermost cell
- Door position is randomised along the wall between `left_col+wall_width` and `right_col-wall_width` (or top/bottom equivalent)
- If `cur_level > 2` and `rand_percent(HIDE_PERCENT=12)`: tile gets `HIDDEN` flag (secret door)

---

## Fill-out pass (`fill_out_level` → `fill_it` → `recursive_deadend`)

After main connections, isolated slots are resolved:

1. Shuffle `offsets[] = {-1, 1, 3, -3}` (10 random swaps).
2. For each slot in random order:
   - `R_NOTHING`: call `fill_it(rn, do_rec_de=1)`
   - `R_CROSS` with `coin_toss()`: call `fill_it(rn, do_rec_de=1)`
3. `r_de` (last dead-end found during recursion): call `fill_it(r_de, do_rec_de=0)` as a second pass without further recursion.

**`fill_it` logic**:

- Looks for a neighbouring `R_ROOM | R_MAZE` slot reachable via offsets.
- Uses `mask_room` to find an existing `TUNNEL` tile in the slot as start point; falls back to centre of slot.
- Places a door on the target room and draws a `draw_simple_passage` to connect.
- Marks the slot as `R_DEADEND`.
- If `rooms_found < 2` and `do_rec_de = true`: calls `recursive_deadend`.

**`recursive_deadend`**: chains `R_NOTHING` slots together as a sequence of dead-end passages, storing the last one in `r_de`.

---

## Hidden passages (`hide_boxed_passage`)

Applies `HIDDEN` flag to random `TUNNEL` tiles within a bounding box.

- Active only when `cur_level > 2`.
- Active only when `width >= 5` OR `height >= 5`.
- For each repetition: up to 10 attempts to find a `TUNNEL` tile at a random position within the box (with 1-cell margin if the dimension allows).

Called from:

| Caller | Repetitions | Condition |
| -------- | ------------- | ----------- |
| `add_mazes` (per maze room) | `get_rand(0, 2)` | always |
| `draw_simple_passage` | 1 | `rand_percent(12)` |

---

## Connectivity check (`is_all_connected`)

BFS/flood-fill from the first connected room; verifies that every `R_ROOM | R_MAZE` slot is reachable. Used as an early-exit condition inside the connection loop.

---

## Player placement (`put_player`)

- Picks a random `FLOOR | TUNNEL | OBJECT | STAIRS` cell via `gr_row_col`.
- Tries up to 2 times to land in a room different from the previous one.

---

## Constants reference

| Constant | Value | Meaning |
| ---------- | ------- | --------- |
| `MAXROOMS` | 9 | Total dungeon slots |
| `DROWS` | 24 | Screen rows |
| `DCOLS` | 80 | Screen columns |
| `MIN_ROW` | 1 | First usable row (row 0 = status bar) |
| `ROW1` | 7 | First horizontal slot divider |
| `ROW2` | 15 | Second horizontal slot divider |
| `COL1` | 26 | First vertical slot divider |
| `COL2` | 52 | Second vertical slot divider |
| `LAST_DUNGEON` | 99 | Deepest reachable level |
| `AMULET_LEVEL` | 26 | Amulet placed at/below this depth |
| `PARTY_TIME` | 10 | Big-room cycle length |
| `HIDE_PERCENT` | 12 | % chance for hidden door/passage |
