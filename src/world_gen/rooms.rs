use crate::core_types::{TileFlags, MAXROOMS, MIN_ROW, DROWS, DCOLS};
use crate::rng::GameRng;

use super::slots::slot_bounds;
use super::types::{DoorLink, DungeonGrid, Room};

pub(super) fn rand_i16(rng: &mut GameRng, lo: i16, hi: i16) -> i16 {
    if lo >= hi {
        lo
    } else {
        rng.get_rand(lo as i32, hi as i32) as i16
    }
}

pub(super) fn make_big_room(rng: &mut GameRng, grid: &mut DungeonGrid) -> Room {
    let top_row = rand_i16(rng, MIN_ROW, MIN_ROW + 5);
    let bottom_row = rand_i16(rng, DROWS as i16 - 7, DROWS as i16 - 2);
    let left_col = rand_i16(rng, 0, 10);
    let right_col = rand_i16(rng, DCOLS as i16 - 11, DCOLS as i16 - 1);
    let room = Room::with_slot(top_row, bottom_row, left_col, right_col, 0);
    draw_room(grid, room);
    room
}

pub(super) fn make_room_in_slot(rng: &mut GameRng, slot: usize, must_exist: bool) -> Option<Room> {
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

pub(super) fn draw_room(grid: &mut DungeonGrid, room: Room) {
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

pub(super) fn set_room_door(
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

pub(super) fn set_room_door_position(
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
