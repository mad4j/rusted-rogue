use crate::core_types::{TileFlags, MAXROOMS};
use crate::rng::GameRng;

use super::rooms::{rand_i16, set_room_door};
use super::slots::{same_col, same_row};
use super::types::{DungeonGrid, Room, DIR_DOWN, DIR_LEFT, DIR_RIGHT, DIR_UP};

pub(super) fn carve_tunnel(grid: &mut DungeonGrid, row: i16, col: i16) {
    if let Some(tile) = grid.get(row, col) {
        if tile.contains(TileFlags::DOOR) {
            return;
        }
        let _ = grid.set(row, col, TileFlags::TUNNEL);
    }
}

pub(super) fn maybe_hide_door(grid: &mut DungeonGrid, rng: &mut GameRng, row: i16, col: i16, level_depth: i16) {
    if level_depth > 2 && rng.rand_percent(12) {
        if let Some(tile) = grid.get(row, col) {
            let _ = grid.set(row, col, tile | TileFlags::HIDDEN);
        }
    }
}

pub(super) fn hide_boxed_passage(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    row1: i16,
    col1: i16,
    row2: i16,
    col2: i16,
    n: i32,
    level_depth: i16,
) {
    if level_depth <= 2 {
        return;
    }
    let (row1, row2) = if row1 > row2 { (row2, row1) } else { (row1, row2) };
    let (col1, col2) = if col1 > col2 { (col2, col1) } else { (col1, col2) };
    let h = row2 - row1;
    let w = col2 - col1;
    if w < 5 && h < 5 {
        return;
    }
    let row_cut: i16 = if h >= 2 { 1 } else { 0 };
    let col_cut: i16 = if w >= 2 { 1 } else { 0 };
    for _ in 0..n {
        for _ in 0..10 {
            let row = rng.get_rand((row1 + row_cut) as i32, (row2 - row_cut) as i32) as i16;
            let col = rng.get_rand((col1 + col_cut) as i32, (col2 - col_cut) as i32) as i16;
            if grid.get(row, col) == Some(TileFlags::TUNNEL) {
                if let Some(tile) = grid.get(row, col) {
                    let _ = grid.set(row, col, tile | TileFlags::HIDDEN);
                }
                break;
            }
        }
    }
}

pub(super) fn draw_simple_passage(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    start: (i16, i16),
    end: (i16, i16),
    horizontal: bool,
    level_depth: i16,
) {
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

    if rng.rand_percent(12) {
        hide_boxed_passage(grid, rng, row1, col1, row2, col2, 1, level_depth);
    }
}

fn put_horizontal_doors(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    left: Room,
    right: Room,
    level_depth: i16,
) -> ((i16, i16), (i16, i16)) {
    let row1 = rng.get_rand((left.top_row + 1) as i32, (left.bottom_row - 1) as i32) as i16;
    let col1 = left.right_col;
    let row2 = rng.get_rand((right.top_row + 1) as i32, (right.bottom_row - 1) as i32) as i16;
    let col2 = right.left_col;

    let _ = grid.set(row1, col1, TileFlags::DOOR);
    maybe_hide_door(grid, rng, row1, col1, level_depth);
    let _ = grid.set(row2, col2, TileFlags::DOOR);
    maybe_hide_door(grid, rng, row2, col2, level_depth);
    ((row1, col1), (row2, col2))
}

fn put_vertical_doors(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    top: Room,
    bottom: Room,
    level_depth: i16,
) -> ((i16, i16), (i16, i16)) {
    let row1 = top.bottom_row;
    let col1 = rng.get_rand((top.left_col + 1) as i32, (top.right_col - 1) as i32) as i16;
    let row2 = bottom.top_row;
    let col2 = rng.get_rand((bottom.left_col + 1) as i32, (bottom.right_col - 1) as i32) as i16;

    let _ = grid.set(row1, col1, TileFlags::DOOR);
    maybe_hide_door(grid, rng, row1, col1, level_depth);
    let _ = grid.set(row2, col2, TileFlags::DOOR);
    maybe_hide_door(grid, rng, row2, col2, level_depth);
    ((row1, col1), (row2, col2))
}

pub(super) fn door_on_room_side(
    room: Room,
    toward_horizontal: bool,
    toward_positive: bool,
    rng: &mut GameRng,
) -> (i16, i16) {
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

pub(super) fn connect_rooms(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    rooms: &mut [Option<Room>; MAXROOMS],
    room1: usize,
    room2: usize,
    level_depth: i16,
) -> bool {
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
        let (d1, d2) = put_horizontal_doors(grid, rng, left, right, level_depth);
        draw_simple_passage(grid, rng, d1, d2, true, level_depth);
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
        let (d1, d2) = put_vertical_doors(grid, rng, top, bottom, level_depth);
        draw_simple_passage(grid, rng, d1, d2, false, level_depth);
        set_room_door(rooms, top_slot, DIR_DOWN, d1, bottom_slot, d2);
        set_room_door(rooms, bottom_slot, DIR_UP, d2, top_slot, d1);
        return true;
    }

    false
}
