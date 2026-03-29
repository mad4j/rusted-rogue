use crate::core_types::{
    Position, TileFlags, COL1, COL2, DCOLS, DROWS, MAXROOMS, MIN_ROW, ROW1, ROW2,
};
use crate::rng::GameRng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Room {
    pub top_row: i16,
    pub bottom_row: i16,
    pub left_col: i16,
    pub right_col: i16,
}

impl Room {
    pub fn new(top_row: i16, bottom_row: i16, left_col: i16, right_col: i16) -> Self {
        Self {
            top_row,
            bottom_row,
            left_col,
            right_col,
        }
    }

    pub fn contains(&self, row: i16, col: i16) -> bool {
        row >= self.top_row
            && row <= self.bottom_row
            && col >= self.left_col
            && col <= self.right_col
    }
}

#[derive(Debug, Clone)]
pub struct DungeonGrid {
    pub cells: [[TileFlags; DCOLS]; DROWS],
}

#[derive(Debug, Clone)]
pub struct GeneratedLevel {
    pub grid: DungeonGrid,
    pub rooms: Vec<Room>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotKind {
    Nothing,
    Room,
    Maze,
    Cross,
    DeadEnd,
}

impl DungeonGrid {
    pub fn new() -> Self {
        Self {
            cells: [[TileFlags::NOTHING; DCOLS]; DROWS],
        }
    }

    pub fn in_bounds(row: i16, col: i16) -> bool {
        row >= 0 && row < DROWS as i16 && col >= 0 && col < DCOLS as i16
    }

    pub fn get(&self, row: i16, col: i16) -> Option<TileFlags> {
        if Self::in_bounds(row, col) {
            Some(self.cells[row as usize][col as usize])
        } else {
            None
        }
    }

    pub fn set(&mut self, row: i16, col: i16, flags: TileFlags) -> bool {
        if Self::in_bounds(row, col) {
            self.cells[row as usize][col as usize] = flags;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = TileFlags::NOTHING;
            }
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (DROWS, DCOLS)
    }

    pub fn is_walkable(&self, row: i16, col: i16) -> bool {
        self.get(row, col).is_some_and(|flags| {
            flags.intersects(
                TileFlags::FLOOR
                    | TileFlags::TUNNEL
                    | TileFlags::DOOR
                    | TileFlags::STAIRS
                    | TileFlags::TRAP,
            )
        })
    }
}

impl GeneratedLevel {
    pub fn spawn_position(&self) -> Position {
        let room = self.rooms[0];
        Position::new(
            (room.top_row + room.bottom_row) / 2,
            (room.left_col + room.right_col) / 2,
        )
    }
}

fn slot_bounds(slot: usize) -> (i16, i16, i16, i16) {
    match slot {
        0 => (MIN_ROW, ROW1 - 1, 0, COL1 - 1),
        1 => (MIN_ROW, ROW1 - 1, COL1 + 1, COL2 - 1),
        2 => (MIN_ROW, ROW1 - 1, COL2 + 1, DCOLS as i16 - 1),
        3 => (ROW1 + 1, ROW2 - 1, 0, COL1 - 1),
        4 => (ROW1 + 1, ROW2 - 1, COL1 + 1, COL2 - 1),
        5 => (ROW1 + 1, ROW2 - 1, COL2 + 1, DCOLS as i16 - 1),
        6 => (ROW2 + 1, DROWS as i16 - 2, 0, COL1 - 1),
        7 => (ROW2 + 1, DROWS as i16 - 2, COL1 + 1, COL2 - 1),
        8 => (ROW2 + 1, DROWS as i16 - 2, COL2 + 1, DCOLS as i16 - 1),
        _ => unreachable!("slot out of bounds"),
    }
}

fn same_row(room1: usize, room2: usize) -> bool {
    (room1 / 3) == (room2 / 3)
}

fn same_col(room1: usize, room2: usize) -> bool {
    (room1 % 3) == (room2 % 3)
}

fn required_room_group(rng: &mut GameRng) -> [usize; 3] {
    match rng.get_rand(0, 5) {
        0 => [0, 1, 2],
        1 => [3, 4, 5],
        2 => [6, 7, 8],
        3 => [0, 3, 6],
        4 => [1, 4, 7],
        _ => [2, 5, 8],
    }
}

fn rand_i16(rng: &mut GameRng, lo: i16, hi: i16) -> i16 {
    if lo >= hi {
        lo
    } else {
        rng.get_rand(lo as i32, hi as i32) as i16
    }
}

fn randomize_offsets(rng: &mut GameRng) -> [i16; 4] {
    let mut offsets = [-1, 1, 3, -3];

    for _ in 0..10 {
        let a = rng.get_rand(0, 3) as usize;
        let b = rng.get_rand(0, 3) as usize;
        offsets.swap(a, b);
    }

    offsets
}

fn draw_maze_in_slot(grid: &mut DungeonGrid, rng: &mut GameRng, slot: usize) {
    let (top, bottom, left, right) = slot_bounds(slot);

    if bottom - top < 2 || right - left < 2 {
        return;
    }

    let start_row = rand_i16(rng, top + 1, bottom - 1);
    let start_col = rand_i16(rng, left + 1, right - 1);
    let _ = grid.set(start_row, start_col, TileFlags::TUNNEL);

    let mut stack = vec![(start_row, start_col)];

    while let Some((row, col)) = stack.last().copied() {
        let mut candidates = Vec::new();

        for (drow, dcol) in [(-2, 0), (2, 0), (0, -2), (0, 2)] {
            let nr = row + drow;
            let nc = col + dcol;

            if nr <= top || nr >= bottom || nc <= left || nc >= right {
                continue;
            }

            if grid.get(nr, nc) == Some(TileFlags::NOTHING) {
                candidates.push((nr, nc, row + (drow / 2), col + (dcol / 2)));
            }
        }

        if candidates.is_empty() {
            let _ = stack.pop();
            continue;
        }

        let pick = rng.get_rand(0, (candidates.len() - 1) as i32) as usize;
        let (nr, nc, wr, wc) = candidates[pick];

        let _ = grid.set(wr, wc, TileFlags::TUNNEL);
        let _ = grid.set(nr, nc, TileFlags::TUNNEL);
        stack.push((nr, nc));
    }
}

fn add_mazes(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    slot_rooms: &[Option<Room>; MAXROOMS],
    slot_kinds: &mut [SlotKind; MAXROOMS],
    level_depth: i16,
) {
    if level_depth <= 1 {
        return;
    }

    let start = rng.get_rand(0, (MAXROOMS - 1) as i32) as usize;
    let maze_percent = maze_percent_for_level(level_depth);

    for i in 0..MAXROOMS {
        let slot = (start + i) % MAXROOMS;

        if slot_rooms[slot].is_none() && slot_kinds[slot] == SlotKind::Nothing && rng.rand_percent(maze_percent) {
            slot_kinds[slot] = SlotKind::Maze;
            draw_maze_in_slot(grid, rng, slot);
        }
    }
}

fn maze_percent_for_level(level_depth: i16) -> i32 {
    if level_depth <= 1 {
        return 0;
    }

    let mut percent = (level_depth as i32 * 5) / 4;
    if level_depth > 15 {
        percent += level_depth as i32;
    }
    percent
}

fn carve_tunnel(grid: &mut DungeonGrid, row: i16, col: i16) {
    if let Some(tile) = grid.get(row, col) {
        if tile.contains(TileFlags::DOOR) {
            return;
        }
        let _ = grid.set(row, col, TileFlags::TUNNEL);
    }
}

fn draw_room(grid: &mut DungeonGrid, room: Room) {
    for row in room.top_row..=room.bottom_row {
        for col in room.left_col..=room.right_col {
            let is_border =
                row == room.top_row || row == room.bottom_row || col == room.left_col || col == room.right_col;
            let tile = if is_border {
                if row == room.top_row || row == room.bottom_row {
                    TileFlags::HORWALL
                } else {
                    TileFlags::VERTWALL
                }
            } else {
                TileFlags::FLOOR
            };
            let _ = grid.set(row, col, tile);
        }
    }
}

fn make_room_in_slot(rng: &mut GameRng, slot: usize, must_exist: bool) -> Option<Room> {
    let (mut top_row, bottom_limit, mut left_col, right_limit) = slot_bounds(slot);

    let available_height = (bottom_limit - top_row + 1).max(4);
    let available_width = (right_limit - left_col - 2).max(7);

    let height = rng.get_rand(4, available_height as i32) as i16;
    let width = rng.get_rand(7, available_width as i32) as i16;

    let row_offset_max = ((bottom_limit - top_row) - height + 1).max(0);
    let col_offset_max = ((right_limit - left_col) - width + 1).max(0);

    let row_offset = rng.get_rand(0, row_offset_max as i32) as i16;
    let col_offset = rng.get_rand(0, col_offset_max as i32) as i16;

    top_row += row_offset;
    left_col += col_offset;
    let bottom_row = top_row + height - 1;
    let right_col = left_col + width - 1;

    if !must_exist && rng.rand_percent(40) {
        return None;
    }

    Some(Room::new(top_row, bottom_row, left_col, right_col))
}

fn put_horizontal_doors(grid: &mut DungeonGrid, rng: &mut GameRng, left: Room, right: Room) -> ((i16, i16), (i16, i16)) {
    let row1 = rng.get_rand((left.top_row + 1) as i32, (left.bottom_row - 1) as i32) as i16;
    let col1 = left.right_col;
    let row2 = rng.get_rand((right.top_row + 1) as i32, (right.bottom_row - 1) as i32) as i16;
    let col2 = right.left_col;

    let _ = grid.set(row1, col1, TileFlags::DOOR);
    let _ = grid.set(row2, col2, TileFlags::DOOR);
    ((row1, col1), (row2, col2))
}

fn put_vertical_doors(grid: &mut DungeonGrid, rng: &mut GameRng, top: Room, bottom: Room) -> ((i16, i16), (i16, i16)) {
    let row1 = top.bottom_row;
    let col1 = rng.get_rand((top.left_col + 1) as i32, (top.right_col - 1) as i32) as i16;
    let row2 = bottom.top_row;
    let col2 = rng.get_rand((bottom.left_col + 1) as i32, (bottom.right_col - 1) as i32) as i16;

    let _ = grid.set(row1, col1, TileFlags::DOOR);
    let _ = grid.set(row2, col2, TileFlags::DOOR);
    ((row1, col1), (row2, col2))
}

fn door_on_room_side(room: Room, toward_horizontal: bool, toward_positive: bool, rng: &mut GameRng) -> (i16, i16) {
    if toward_horizontal {
        let row = rand_i16(rng, room.top_row + 1, room.bottom_row - 1);
        let col = if toward_positive {
            room.right_col
        } else {
            room.left_col
        };
        (row, col)
    } else {
        let row = if toward_positive {
            room.bottom_row
        } else {
            room.top_row
        };
        let col = rand_i16(rng, room.left_col + 1, room.right_col - 1);
        (row, col)
    }
}

fn draw_simple_passage(grid: &mut DungeonGrid, rng: &mut GameRng, start: (i16, i16), end: (i16, i16), horizontal: bool) {
    let (mut row1, mut col1) = start;
    let (mut row2, mut col2) = end;

    if horizontal {
        if col1 > col2 {
            std::mem::swap(&mut row1, &mut row2);
            std::mem::swap(&mut col1, &mut col2);
        }

        let middle = rng.get_rand((col1 + 1) as i32, (col2 - 1) as i32) as i16;

        for col in (col1 + 1)..middle {
            carve_tunnel(grid, row1, col);
        }

        let step = if row1 <= row2 { 1 } else { -1 };
        let mut row = row1;
        while row != row2 {
            carve_tunnel(grid, row, middle);
            row += step;
        }

        for col in middle..col2 {
            carve_tunnel(grid, row2, col);
        }
    } else {
        if row1 > row2 {
            std::mem::swap(&mut row1, &mut row2);
            std::mem::swap(&mut col1, &mut col2);
        }

        let middle = rng.get_rand((row1 + 1) as i32, (row2 - 1) as i32) as i16;

        for row in (row1 + 1)..middle {
            carve_tunnel(grid, row, col1);
        }

        let step = if col1 <= col2 { 1 } else { -1 };
        let mut col = col1;
        while col != col2 {
            carve_tunnel(grid, middle, col);
            col += step;
        }

        for row in middle..row2 {
            carve_tunnel(grid, row, col2);
        }
    }
}

fn connect_rooms(grid: &mut DungeonGrid, rng: &mut GameRng, rooms: &[Option<Room>; MAXROOMS], room1: usize, room2: usize) -> bool {
    let Some(a) = rooms[room1] else {
        return false;
    };
    let Some(b) = rooms[room2] else {
        return false;
    };

    if same_row(room1, room2) {
        let (left, right) = if a.left_col <= b.left_col { (a, b) } else { (b, a) };
        let (d1, d2) = put_horizontal_doors(grid, rng, left, right);
        draw_simple_passage(grid, rng, d1, d2, true);
        return true;
    }

    if same_col(room1, room2) {
        let (top, bottom) = if a.top_row <= b.top_row { (a, b) } else { (b, a) };
        let (d1, d2) = put_vertical_doors(grid, rng, top, bottom);
        draw_simple_passage(grid, rng, d1, d2, false);
        return true;
    }

    false
}

fn fill_out_level(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    slot_rooms: &[Option<Room>; MAXROOMS],
    slot_kinds: &mut [SlotKind; MAXROOMS],
) {
    for slot in 0..MAXROOMS {
        if slot_kinds[slot] != SlotKind::Nothing && slot_kinds[slot] != SlotKind::Cross {
            continue;
        }

        let offsets = randomize_offsets(rng);
        let mut connected = false;

        for offset in offsets {
            let target = slot as i16 + offset;
            if !(0..MAXROOMS as i16).contains(&target) {
                continue;
            }

            let target = target as usize;
            if !(same_row(slot, target) || same_col(slot, target)) {
                continue;
            }

            let target_kind = slot_kinds[target];
            let target_playable = slot_rooms[target].is_some()
                || target_kind == SlotKind::Maze
                || target_kind == SlotKind::DeadEnd
                || target_kind == SlotKind::Cross;

            if !target_playable {
                continue;
            }

            let (top, bottom, left, right) = slot_bounds(slot);
            let horizontal = same_row(slot, target);

            let start = if horizontal {
                let row = rand_i16(rng, top + 1, bottom - 1);
                let col = if slot < target { right } else { left };
                (row, col)
            } else {
                let row = if slot < target { bottom } else { top };
                let col = rand_i16(rng, left + 1, right - 1);
                (row, col)
            };

            let end = if let Some(target_room) = slot_rooms[target] {
                let toward_positive = target > slot;
                let (drow, dcol) = door_on_room_side(target_room, horizontal, !toward_positive, rng);
                let _ = grid.set(drow, dcol, TileFlags::DOOR);
                (drow, dcol)
            } else {
                let (ttop, tbottom, tleft, tright) = slot_bounds(target);
                if horizontal {
                    let row = rand_i16(rng, ttop + 1, tbottom - 1);
                    let col = if slot < target { tleft } else { tright };
                    (row, col)
                } else {
                    let row = if slot < target { ttop } else { tbottom };
                    let col = rand_i16(rng, tleft + 1, tright - 1);
                    (row, col)
                }
            };

            let _ = grid.set(start.0, start.1, TileFlags::TUNNEL);
            draw_simple_passage(grid, rng, start, end, horizontal);
            slot_kinds[slot] = SlotKind::DeadEnd;
            connected = true;
            break;
        }

        if !connected {
            slot_kinds[slot] = SlotKind::Nothing;
        }
    }
}

pub fn generate_level_with_depth(rng: &mut GameRng, level_depth: i16) -> GeneratedLevel {
    let mut grid = DungeonGrid::new();
    let required = required_room_group(rng);
    let mut slot_rooms: [Option<Room>; MAXROOMS] = [None; MAXROOMS];
    let mut slot_kinds = [SlotKind::Nothing; MAXROOMS];

    for slot in 0..MAXROOMS {
        let must_exist = required.contains(&slot);
        slot_rooms[slot] = make_room_in_slot(rng, slot, must_exist);
        if let Some(room) = slot_rooms[slot] {
            draw_room(&mut grid, room);
            slot_kinds[slot] = SlotKind::Room;
        }
    }

    add_mazes(&mut grid, rng, &slot_rooms, &mut slot_kinds, level_depth);

    for i in 0..MAXROOMS {
        if i < (MAXROOMS - 1) {
            let _ = connect_rooms(&mut grid, rng, &slot_rooms, i, i + 1);
        }
        if i < (MAXROOMS - 3) {
            let _ = connect_rooms(&mut grid, rng, &slot_rooms, i, i + 3);
        }
        if i < (MAXROOMS - 2) && slot_rooms[i + 1].is_none() {
            if connect_rooms(&mut grid, rng, &slot_rooms, i, i + 2) {
                slot_kinds[i + 1] = SlotKind::Cross;
            }
        }
        if i < (MAXROOMS - 6) && slot_rooms[i + 3].is_none() {
            if connect_rooms(&mut grid, rng, &slot_rooms, i, i + 6) {
                slot_kinds[i + 3] = SlotKind::Cross;
            }
        }
    }

    fill_out_level(&mut grid, rng, &slot_rooms, &mut slot_kinds);

    let rooms: Vec<Room> = slot_rooms.into_iter().flatten().collect();

    GeneratedLevel { grid, rooms }
}

pub fn generate_level(rng: &mut GameRng) -> GeneratedLevel {
    generate_level_with_depth(rng, 1)
}

#[cfg(test)]
mod tests {
    use std::collections::{HashSet, VecDeque};

    use proptest::prelude::*;

    use super::{generate_level, generate_level_with_depth, maze_percent_for_level, DungeonGrid, Room};
    use crate::core_types::{TileFlags, DCOLS, DROWS, MAXROOMS};
    use crate::rng::GameRng;

    #[test]
    fn dungeon_has_legacy_dimensions() {
        let grid = DungeonGrid::new();
        assert_eq!(grid.dimensions(), (DROWS, DCOLS));
    }

    #[test]
    fn set_and_get_tile_flags() {
        let mut grid = DungeonGrid::new();
        let flags = TileFlags::FLOOR | TileFlags::OBJECT;
        assert!(grid.set(10, 20, flags));
        assert_eq!(grid.get(10, 20), Some(flags));
    }

    #[test]
    fn out_of_bounds_is_rejected() {
        let mut grid = DungeonGrid::new();
        assert!(!grid.set(-1, 0, TileFlags::FLOOR));
        assert_eq!(grid.get(0, 80), None);
    }

    #[test]
    fn room_contains_points_inside_bounds() {
        let room = Room::new(5, 10, 15, 20);
        assert!(room.contains(5, 15));
        assert!(room.contains(10, 20));
        assert!(!room.contains(4, 15));
        assert!(!room.contains(10, 21));
    }

    #[test]
    fn generated_level_is_deterministic_for_seed() {
        let mut rng_a = GameRng::new(12345);
        let mut rng_b = GameRng::new(12345);

        let a = generate_level(&mut rng_a);
        let b = generate_level(&mut rng_b);

        assert_eq!(a.rooms, b.rooms);
    }

    #[test]
    fn generated_room_within_bounds() {
        let mut rng = GameRng::new(12345);
        let generated = generate_level(&mut rng);
        assert!(!generated.rooms.is_empty());
        assert!(generated.rooms.len() <= MAXROOMS);

        for room in &generated.rooms {
            assert!(room.top_row >= 1);
            assert!(room.left_col >= 0);
            assert!(room.bottom_row < DROWS as i16);
            assert!(room.right_col < DCOLS as i16);
        }
    }

    #[test]
    fn generated_level_has_multiple_rooms_for_seed_12345() {
        let mut rng = GameRng::new(12345);
        let generated = generate_level(&mut rng);
        assert!(generated.rooms.len() >= 3);
    }

    #[test]
    fn generated_level_has_walkable_spawn() {
        let mut rng = GameRng::new(12345);
        let generated = generate_level(&mut rng);
        let spawn = generated.spawn_position();

        assert!(generated.grid.is_walkable(spawn.row, spawn.col));
    }

    #[test]
    fn generated_level_has_doors_or_tunnels() {
        let mut rng = GameRng::new(12345);
        let generated = generate_level(&mut rng);

        let (rows, cols) = generated.grid.dimensions();
        let mut found_connection_tile = false;

        for row in 0..rows as i16 {
            for col in 0..cols as i16 {
                let tile = generated.grid.get(row, col).unwrap_or(TileFlags::NOTHING);
                if tile.intersects(TileFlags::DOOR | TileFlags::TUNNEL) {
                    found_connection_tile = true;
                    break;
                }
            }
            if found_connection_tile {
                break;
            }
        }

        assert!(found_connection_tile);
    }

    #[test]
    fn maze_percent_matches_legacy_formula() {
        assert_eq!(maze_percent_for_level(1), 0);
        assert_eq!(maze_percent_for_level(2), 2);
        assert_eq!(maze_percent_for_level(10), 12);
        assert_eq!(maze_percent_for_level(16), 36);
    }

    #[test]
    fn depth_aware_generator_is_deterministic() {
        let mut rng_a = GameRng::new(777);
        let mut rng_b = GameRng::new(777);

        let a = generate_level_with_depth(&mut rng_a, 20);
        let b = generate_level_with_depth(&mut rng_b, 20);

        assert_eq!(a.rooms, b.rooms);
    }

    #[test]
    fn all_room_centers_are_reachable_from_spawn_for_seed_window() {
        for seed in 0..64 {
            let mut rng = GameRng::new(seed);
            let generated = generate_level(&mut rng);
            let spawn = generated.spawn_position();

            let mut visited: HashSet<(i16, i16)> = HashSet::new();
            let mut queue = VecDeque::new();

            visited.insert((spawn.row, spawn.col));
            queue.push_back((spawn.row, spawn.col));

            while let Some((row, col)) = queue.pop_front() {
                for (drow, dcol) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nr = row + drow;
                    let nc = col + dcol;
                    if !generated.grid.is_walkable(nr, nc) {
                        continue;
                    }
                    if visited.insert((nr, nc)) {
                        queue.push_back((nr, nc));
                    }
                }
            }

            for room in &generated.rooms {
                let center = ((room.top_row + room.bottom_row) / 2, (room.left_col + room.right_col) / 2);
                assert!(
                    visited.contains(&center),
                    "seed {seed}: room center {:?} not reachable from spawn",
                    center
                );
            }
        }
    }

    #[test]
    fn seed_window_contains_dead_end_tunnels() {
        let mut found_dead_end = false;

        for seed in 0..128 {
            let mut rng = GameRng::new(seed);
            let generated = generate_level(&mut rng);
            let (rows, cols) = generated.grid.dimensions();

            for row in 1..(rows as i16 - 1) {
                for col in 1..(cols as i16 - 1) {
                    let tile = generated.grid.get(row, col).unwrap_or(TileFlags::NOTHING);
                    if !tile.contains(TileFlags::TUNNEL) {
                        continue;
                    }

                    let mut walkable_neighbors = 0;
                    for (drow, dcol) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        if generated.grid.is_walkable(row + drow, col + dcol) {
                            walkable_neighbors += 1;
                        }
                    }

                    if walkable_neighbors == 1 {
                        found_dead_end = true;
                        break;
                    }
                }

                if found_dead_end {
                    break;
                }
            }

            if found_dead_end {
                break;
            }
        }

        assert!(found_dead_end, "expected at least one dead-end tunnel in seed window");
    }

    #[test]
    fn higher_depth_produces_more_tunnels_on_average() {
        let mut low_depth_tunnels = 0usize;
        let mut high_depth_tunnels = 0usize;

        for seed in 0..128 {
            let mut low_rng = GameRng::new(seed);
            let low = generate_level_with_depth(&mut low_rng, 2);

            let mut high_rng = GameRng::new(seed);
            let high = generate_level_with_depth(&mut high_rng, 20);

            let (rows, cols) = low.grid.dimensions();

            for row in 0..rows as i16 {
                for col in 0..cols as i16 {
                    let low_tile = low.grid.get(row, col).unwrap_or(TileFlags::NOTHING);
                    if low_tile.contains(TileFlags::TUNNEL) {
                        low_depth_tunnels += 1;
                    }

                    let high_tile = high.grid.get(row, col).unwrap_or(TileFlags::NOTHING);
                    if high_tile.contains(TileFlags::TUNNEL) {
                        high_depth_tunnels += 1;
                    }
                }
            }
        }

        assert!(
            high_depth_tunnels > low_depth_tunnels,
            "expected higher depth to produce more tunnels on average (low={low_depth_tunnels}, high={high_depth_tunnels})"
        );
    }

    proptest! {
        #[test]
        fn generated_room_stays_in_bounds_for_any_seed(seed in any::<i32>()) {
            let mut rng = GameRng::new(seed);
            let generated = generate_level(&mut rng);

            prop_assert!(!generated.rooms.is_empty());
            prop_assert!(generated.rooms.len() <= MAXROOMS);

            for room in &generated.rooms {
                prop_assert!(room.top_row >= 1);
                prop_assert!(room.left_col >= 0);
                prop_assert!(room.bottom_row < DROWS as i16);
                prop_assert!(room.right_col < DCOLS as i16);
                prop_assert!(room.top_row < room.bottom_row);
                prop_assert!(room.left_col < room.right_col);
            }
        }

        #[test]
        fn generated_spawn_is_walkable_for_any_seed(seed in any::<i32>()) {
            let mut rng = GameRng::new(seed);
            let generated = generate_level(&mut rng);
            let spawn = generated.spawn_position();

            prop_assert!(generated.grid.is_walkable(spawn.row, spawn.col));
        }
    }
}
