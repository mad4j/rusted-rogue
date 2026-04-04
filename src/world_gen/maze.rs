use crate::core_types::{TileFlags, MAXROOMS};
use crate::rng::GameRng;

use super::passage::hide_boxed_passage;
use super::rooms::rand_i16;
use super::slots::slot_bounds;
use super::types::{DungeonGrid, Room, SlotKind};

pub(super) fn maze_percent_for_level(level_depth: i16) -> i32 {
    if level_depth <= 1 {
        return 0;
    }

    let mut percent = (level_depth as i32 * 5) / 4;
    if level_depth > 15 {
        percent += level_depth as i32;
    }
    percent
}

pub(super) fn draw_maze_in_slot(grid: &mut DungeonGrid, rng: &mut GameRng, slot: usize) {
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

pub(super) fn add_mazes(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    slot_rooms: &mut [Option<Room>; MAXROOMS],
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
            let (top, bottom, left, right) = slot_bounds(slot);
            slot_rooms[slot] = Some(Room::with_slot(top, bottom, left, right, slot));
            let n = rng.get_rand(0, 2);
            hide_boxed_passage(grid, rng, top, left, bottom, right, n, level_depth);
        }
    }
}
