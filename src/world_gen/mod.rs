use crate::core_types::{
    Position, TileFlags, COL1, COL2, DCOLS, DROWS, MAXROOMS, MIN_ROW, ROW1, ROW2,
};
use crate::rng::GameRng;

const DIR_UP: usize = 0;
const DIR_RIGHT: usize = 1;
const DIR_DOWN: usize = 2;
const DIR_LEFT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorLink {
    pub door_row: i16,
    pub door_col: i16,
    pub oth_room: i16,
    pub oth_row: i16,
    pub oth_col: i16,
}

impl DoorLink {
    pub const NONE: Self = Self {
        door_row: -1,
        door_col: -1,
        oth_room: -1,
        oth_row: -1,
        oth_col: -1,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Room {
    pub top_row: i16,
    pub bottom_row: i16,
    pub left_col: i16,
    pub right_col: i16,
    pub slot_index: i16,
    pub doors: [DoorLink; 4],
}

impl Room {
    pub fn new(top_row: i16, bottom_row: i16, left_col: i16, right_col: i16) -> Self {
        Self {
            top_row,
            bottom_row,
            left_col,
            right_col,
            slot_index: -1,
            doors: [DoorLink::NONE; 4],
        }
    }

    pub fn with_slot(top_row: i16, bottom_row: i16, left_col: i16, right_col: i16, slot_index: usize) -> Self {
        let mut room = Self::new(top_row, bottom_row, left_col, right_col);
        room.slot_index = slot_index as i16;
        room
    }

    pub fn with_metadata(
        top_row: i16,
        bottom_row: i16,
        left_col: i16,
        right_col: i16,
        slot_index: i16,
        doors: [DoorLink; 4],
    ) -> Self {
        Self {
            top_row,
            bottom_row,
            left_col,
            right_col,
            slot_index,
            doors,
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

fn coin_toss(rng: &mut GameRng) -> bool {
    rng.rand_percent(50)
}

fn mixed_room_order(rng: &mut GameRng) -> [usize; MAXROOMS] {
    let mut order = [0, 1, 2, 3, 4, 5, 6, 7, 8];

    for _ in 0..(3 * MAXROOMS) {
        let a = rng.get_rand(0, (MAXROOMS - 1) as i32) as usize;
        let mut b = rng.get_rand(0, (MAXROOMS - 1) as i32) as usize;
        while a == b {
            b = rng.get_rand(0, (MAXROOMS - 1) as i32) as usize;
        }
        order.swap(a, b);
    }

    order
}

fn mark_connection(connections: &mut [[bool; MAXROOMS]; MAXROOMS], a: usize, b: usize) {
    connections[a][b] = true;
    connections[b][a] = true;
}

fn are_playable_rooms_connected(
    slot_rooms: &[Option<Room>; MAXROOMS],
    connections: &[[bool; MAXROOMS]; MAXROOMS],
) -> bool {
    let Some(start) = slot_rooms.iter().position(Option::is_some) else {
        return true;
    };

    let mut visited = [false; MAXROOMS];
    let mut stack = vec![start];
    visited[start] = true;

    while let Some(node) = stack.pop() {
        for next in 0..MAXROOMS {
            if !connections[node][next] || visited[next] || slot_rooms[next].is_none() {
                continue;
            }
            visited[next] = true;
            stack.push(next);
        }
    }

    for i in 0..MAXROOMS {
        if slot_rooms[i].is_some() && !visited[i] {
            return false;
        }
    }

    true
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

fn slot_center(slot: usize) -> (i16, i16) {
    let (top, bottom, left, right) = slot_bounds(slot);
    ((top + bottom) / 2, (left + right) / 2)
}

fn mask_slot(grid: &DungeonGrid, slot: usize, mask: TileFlags) -> Option<(i16, i16)> {
    let (top, bottom, left, right) = slot_bounds(slot);

    for row in top..=bottom {
        for col in left..=right {
            let Some(tile) = grid.get(row, col) else {
                continue;
            };
            if tile.intersects(mask) {
                return Some((row, col));
            }
        }
    }

    None
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

    Some(Room::with_slot(top_row, bottom_row, left_col, right_col, slot))
}

fn set_room_door(
    rooms: &mut [Option<Room>; MAXROOMS],
    from_slot: usize,
    direction: usize,
    door: (i16, i16),
    to_slot: usize,
    to_door: (i16, i16),
) {
    if let Some(room) = &mut rooms[from_slot] {
        room.doors[direction] = DoorLink {
            door_row: door.0,
            door_col: door.1,
            oth_room: to_slot as i16,
            oth_row: to_door.0,
            oth_col: to_door.1,
        };
    }
}

fn set_room_door_position(
    rooms: &mut [Option<Room>; MAXROOMS],
    room_slot: usize,
    direction: usize,
    door: (i16, i16),
) {
    if let Some(room) = &mut rooms[room_slot] {
        room.doors[direction] = DoorLink {
            door_row: door.0,
            door_col: door.1,
            oth_room: -1,
            oth_row: -1,
            oth_col: -1,
        };
    }
}

fn recursive_deadend(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    slot_kinds: &mut [SlotKind; MAXROOMS],
    rn: usize,
    offsets: [i16; 4],
    srow: i16,
    scol: i16,
    r_de: &mut Option<usize>,
) {
    slot_kinds[rn] = SlotKind::DeadEnd;
    let _ = grid.set(srow, scol, TileFlags::TUNNEL);

    for offset in offsets {
        let target = rn as i16 + offset;
        if !(0..MAXROOMS as i16).contains(&target) {
            continue;
        }

        let de = target as usize;
        if !(same_row(rn, de) || same_col(rn, de)) {
            continue;
        }
        if slot_kinds[de] != SlotKind::Nothing {
            continue;
        }

        let (drow, dcol) = slot_center(de);
        draw_simple_passage(grid, rng, (srow, scol), (drow, dcol), same_row(rn, de));
        *r_de = Some(de);
        recursive_deadend(grid, rng, slot_kinds, de, offsets, drow, dcol, r_de);
    }
}

fn fill_it(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    slot_rooms: &mut [Option<Room>; MAXROOMS],
    slot_kinds: &mut [SlotKind; MAXROOMS],
    rn: usize,
    do_rec_de: bool,
    r_de: &mut Option<usize>,
) {
    let offsets = randomize_offsets(rng);
    let mut rooms_found = 0;
    let mut did_this = false;

    for (index, offset) in offsets.into_iter().enumerate() {
        let target_raw = rn as i16 + offset;
        if !(0..MAXROOMS as i16).contains(&target_raw) {
            continue;
        }

        let target = target_raw as usize;
        if !(same_row(rn, target) || same_col(rn, target)) {
            continue;
        }

        if slot_kinds[target] != SlotKind::Room && slot_kinds[target] != SlotKind::Maze {
            continue;
        }

        let horizontal = same_row(rn, target);
        let tunnel_dir = if horizontal {
            if rn < target { DIR_RIGHT } else { DIR_LEFT }
        } else if rn < target {
            DIR_DOWN
        } else {
            DIR_UP
        };
        let door_dir = (tunnel_dir + 2) % 4;

        let Some(target_room) = slot_rooms[target] else {
            continue;
        };
        if target_room.doors[door_dir].oth_room >= 0 {
            continue;
        }

        let (mut srow, mut scol) = slot_center(rn);
        if do_rec_de && !did_this {
            if let Some((row, col)) = mask_slot(grid, rn, TileFlags::TUNNEL) {
                srow = row;
                scol = col;
            }
        }

        let toward_positive = rn > target;
        let (drow, dcol) = door_on_room_side(target_room, horizontal, toward_positive, rng);
        let _ = grid.set(drow, dcol, TileFlags::DOOR);
        set_room_door_position(slot_rooms, target, door_dir, (drow, dcol));

        rooms_found += 1;
        draw_simple_passage(grid, rng, (srow, scol), (drow, dcol), horizontal);
        slot_kinds[rn] = SlotKind::DeadEnd;
        let _ = grid.set(srow, scol, TileFlags::TUNNEL);

        if (index < 3) && !did_this {
            did_this = true;
            if coin_toss(rng) {
                continue;
            }
        }

        if rooms_found < 2 && do_rec_de {
            recursive_deadend(grid, rng, slot_kinds, rn, offsets, srow, scol, r_de);
        }
        break;
    }
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

fn connect_rooms(grid: &mut DungeonGrid, rng: &mut GameRng, rooms: &mut [Option<Room>; MAXROOMS], room1: usize, room2: usize) -> bool {
    let Some(a) = rooms[room1] else {
        return false;
    };
    let Some(b) = rooms[room2] else {
        return false;
    };

    if same_row(room1, room2) {
        let (left_slot, right_slot, left, right) = if a.left_col <= b.left_col {
            (room1, room2, a, b)
        } else {
            (room2, room1, b, a)
        };
        let (d1, d2) = put_horizontal_doors(grid, rng, left, right);
        draw_simple_passage(grid, rng, d1, d2, true);
        set_room_door(rooms, left_slot, DIR_RIGHT, d1, right_slot, d2);
        set_room_door(rooms, right_slot, DIR_LEFT, d2, left_slot, d1);
        return true;
    }

    if same_col(room1, room2) {
        let (top_slot, bottom_slot, top, bottom) = if a.top_row <= b.top_row {
            (room1, room2, a, b)
        } else {
            (room2, room1, b, a)
        };
        let (d1, d2) = put_vertical_doors(grid, rng, top, bottom);
        draw_simple_passage(grid, rng, d1, d2, false);
        set_room_door(rooms, top_slot, DIR_DOWN, d1, bottom_slot, d2);
        set_room_door(rooms, bottom_slot, DIR_UP, d2, top_slot, d1);
        return true;
    }

    false
}

fn fill_out_level(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    slot_rooms: &mut [Option<Room>; MAXROOMS],
    slot_kinds: &mut [SlotKind; MAXROOMS],
) {
    let order = mixed_room_order(rng);
    let mut r_de: Option<usize> = None;

    for rn in order {
        let kind = slot_kinds[rn];
        if kind == SlotKind::Nothing || (kind == SlotKind::Cross && coin_toss(rng)) {
            fill_it(grid, rng, slot_rooms, slot_kinds, rn, true, &mut r_de);
        }
    }

    if let Some(de) = r_de {
        fill_it(grid, rng, slot_rooms, slot_kinds, de, false, &mut r_de);
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

    let room_order = mixed_room_order(rng);
    let mut connections = [[false; MAXROOMS]; MAXROOMS];

    for i in room_order {
        if i < (MAXROOMS - 1) {
            if connect_rooms(&mut grid, rng, &mut slot_rooms, i, i + 1) {
                mark_connection(&mut connections, i, i + 1);
            }
        }
        if i < (MAXROOMS - 3) {
            if connect_rooms(&mut grid, rng, &mut slot_rooms, i, i + 3) {
                mark_connection(&mut connections, i, i + 3);
            }
        }
        if i < (MAXROOMS - 2) && slot_rooms[i + 1].is_none() {
            if connect_rooms(&mut grid, rng, &mut slot_rooms, i, i + 2) {
                slot_kinds[i + 1] = SlotKind::Cross;
                mark_connection(&mut connections, i, i + 2);
            }
        }
        if i < (MAXROOMS - 6) && slot_rooms[i + 3].is_none() {
            if connect_rooms(&mut grid, rng, &mut slot_rooms, i, i + 6) {
                slot_kinds[i + 3] = SlotKind::Cross;
                mark_connection(&mut connections, i, i + 6);
            }
        }

        if are_playable_rooms_connected(&slot_rooms, &connections) {
            break;
        }
    }

    fill_out_level(&mut grid, rng, &mut slot_rooms, &mut slot_kinds);

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
