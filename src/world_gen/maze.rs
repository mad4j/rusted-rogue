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

/// Returns a direction array [UP, DOWN, LEFT, RIGHT] as (dr, dc) pairs,
/// shuffled with 33% probability (10 random swaps, matching original `make_maze`).
fn maze_dirs(rng: &mut GameRng) -> [(i16, i16); 4] {
    let mut dirs: [(i16, i16); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    if rng.rand_percent(33) {
        for _ in 0..10 {
            let t1 = rng.get_rand(0, 3) as usize;
            let t2 = rng.get_rand(0, 3) as usize;
            dirs.swap(t1, t2);
        }
    }
    dirs
}

fn is_tunnel(grid: &DungeonGrid, r: i16, c: i16) -> bool {
    grid.get(r, c) == Some(TileFlags::TUNNEL)
}

/// Checks whether the maze DFS can move one step in direction `(dr, dc)` from `(r, c)`
/// to `(nr, nc)`, applying the same 4-cell adjacency constraints as the original C `make_maze`.
fn maze_can_move(
    grid: &DungeonGrid,
    dr: i16,
    dc: i16,
    nr: i16,
    nc: i16,
    top: i16,
    bottom: i16,
    left: i16,
    right: i16,
) -> bool {
    if nr < top || nr > bottom || nc < left || nc > right {
        return false;
    }
    if is_tunnel(grid, nr, nc) {
        return false;
    }
    // Adjacency checks: prevent two tunnel cells from being laterally adjacent.
    // Mirrors original C conditions for each direction.
    if dr == -1 {
        // UP: destination (r-1, c) — check [r-1][c-1], [r-1][c+1], [r-2][c]
        if is_tunnel(grid, nr, nc - 1) { return false; }
        if is_tunnel(grid, nr, nc + 1) { return false; }
        if is_tunnel(grid, nr - 1, nc) { return false; }
    } else if dr == 1 {
        // DOWN: destination (r+1, c) — check [r+1][c-1], [r+1][c+1], [r+2][c]
        if is_tunnel(grid, nr, nc - 1) { return false; }
        if is_tunnel(grid, nr, nc + 1) { return false; }
        if is_tunnel(grid, nr + 1, nc) { return false; }
    } else if dc == -1 {
        // LEFT: destination (r, c-1) — check [r-1][c-1], [r+1][c-1], [r][c-2]
        if is_tunnel(grid, nr - 1, nc) { return false; }
        if is_tunnel(grid, nr + 1, nc) { return false; }
        if is_tunnel(grid, nr, nc - 1) { return false; }
    } else {
        // RIGHT: destination (r, c+1) — check [r-1][c+1], [r+1][c+1], [r][c+2]
        if is_tunnel(grid, nr - 1, nc) { return false; }
        if is_tunnel(grid, nr + 1, nc) { return false; }
        if is_tunnel(grid, nr, nc + 1) { return false; }
    }
    true
}

pub(super) fn draw_maze_in_slot(grid: &mut DungeonGrid, rng: &mut GameRng, slot: usize) {
    let (top, bottom, left, right) = slot_bounds(slot);

    if bottom - top < 2 || right - left < 2 {
        return;
    }

    let start_row = rand_i16(rng, top + 1, bottom - 1);
    let start_col = rand_i16(rng, left + 1, right - 1);
    let _ = grid.set(start_row, start_col, TileFlags::TUNNEL);

    // Iterative simulation of the original recursive make_maze (1-cell DFS).
    // Each stack frame holds (row, col, dir_index, dirs) so we can resume iterating
    // through all 4 directions after returning from a recursive branch.
    let initial_dirs = maze_dirs(rng);
    let mut stack: Vec<(i16, i16, usize, [(i16, i16); 4])> =
        vec![(start_row, start_col, 0, initial_dirs)];

    while !stack.is_empty() {
        let dir_idx = {
            let frame = stack.last_mut().unwrap();
            let idx = frame.2;
            frame.2 += 1;
            idx
        };

        if dir_idx >= 4 {
            stack.pop();
            continue;
        }

        let (r, c, dirs) = {
            let frame = stack.last().unwrap();
            (frame.0, frame.1, frame.3)
        };

        let (dr, dc) = dirs[dir_idx];
        let nr = r + dr;
        let nc = c + dc;

        if !maze_can_move(grid, dr, dc, nr, nc, top, bottom, left, right) {
            continue;
        }

        let _ = grid.set(nr, nc, TileFlags::TUNNEL);
        let new_dirs = maze_dirs(rng);
        stack.push((nr, nc, 0, new_dirs));
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
