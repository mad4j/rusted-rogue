use crate::core_types::{COL1, COL2, DCOLS, DROWS, MIN_ROW, ROW1, ROW2, TileFlags};

use super::types::DungeonGrid;

pub(super) fn slot_bounds(slot: usize) -> (i16, i16, i16, i16) {
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

pub(super) fn same_row(room1: usize, room2: usize) -> bool {
    (room1 / 3) == (room2 / 3)
}

pub(super) fn same_col(room1: usize, room2: usize) -> bool {
    (room1 % 3) == (room2 % 3)
}

pub(super) fn slot_center(slot: usize) -> (i16, i16) {
    let (top, bottom, left, right) = slot_bounds(slot);
    ((top + bottom) / 2, (left + right) / 2)
}

pub(super) fn mask_slot(grid: &DungeonGrid, slot: usize, mask: TileFlags) -> Option<(i16, i16)> {
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
