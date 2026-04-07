use crate::core_types::{Position, TileFlags, MAXROOMS};
use crate::rng::GameRng;

use super::maze::add_mazes;
use super::passage::{connect_rooms, door_on_room_side, draw_simple_passage, maybe_hide_door};
use super::rooms::{draw_room, make_big_room, make_room_in_slot, set_room_door_position};
use super::slots::{mask_slot, same_col, same_row, slot_center};
use super::types::{DungeonGrid, GeneratedLevel, Room, SlotKind, DIR_DOWN, DIR_LEFT, DIR_RIGHT, DIR_UP};

fn place_stairs(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    slot_rooms: &[Option<Room>; MAXROOMS],
    slot_kinds: &[SlotKind; MAXROOMS],
) -> Option<Position> {
    // Mirrors original put_stairs: gr_row_col(FLOOR | TUNNEL).
    // Accepts cells that are exactly FLOOR or exactly TUNNEL (no other flags set),
    // inside a R_ROOM or R_MAZE slot. STAIRS is OR-ed onto the existing tile.
    let acceptable = TileFlags::FLOOR | TileFlags::TUNNEL;
    let mut candidates: Vec<(i16, i16)> = Vec::new();

    for (i, maybe_room) in slot_rooms.iter().enumerate() {
        let Some(room) = maybe_room else { continue };
        if slot_kinds[i] != SlotKind::Room && slot_kinds[i] != SlotKind::Maze {
            continue;
        }
        for row in room.top_row..=room.bottom_row {
            for col in room.left_col..=room.right_col {
                let Some(tile) = grid.get(row, col) else { continue };
                if tile.intersects(acceptable) && (tile & !acceptable).is_empty() {
                    candidates.push((row, col));
                }
            }
        }
    }

    if candidates.is_empty() {
        return None;
    }

    let idx = rng.get_rand(0, (candidates.len() - 1) as i32) as usize;
    let (row, col) = candidates[idx];
    if let Some(tile) = grid.get(row, col) {
        let _ = grid.set(row, col, tile | TileFlags::STAIRS);
    }
    Some(Position::new(row, col))
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

fn recursive_deadend(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    slot_kinds: &mut [SlotKind; MAXROOMS],
    rn: usize,
    offsets: [i16; 4],
    srow: i16,
    scol: i16,
    r_de: &mut Option<usize>,
    level_depth: i16,
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
        draw_simple_passage(grid, rng, (srow, scol), (drow, dcol), same_row(rn, de), level_depth);
        *r_de = Some(de);
        recursive_deadend(grid, rng, slot_kinds, de, offsets, drow, dcol, r_de, level_depth);
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
    level_depth: i16,
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
        maybe_hide_door(grid, rng, drow, dcol, level_depth);
        set_room_door_position(slot_rooms, target, door_dir, (drow, dcol));

        rooms_found += 1;
        draw_simple_passage(grid, rng, (srow, scol), (drow, dcol), horizontal, level_depth);
        slot_kinds[rn] = SlotKind::DeadEnd;
        let _ = grid.set(srow, scol, TileFlags::TUNNEL);

        if (index < 3) && !did_this {
            did_this = true;
            if coin_toss(rng) {
                continue;
            }
        }

        if rooms_found < 2 && do_rec_de {
            recursive_deadend(grid, rng, slot_kinds, rn, offsets, srow, scol, r_de, level_depth);
        }
        break;
    }
}

fn fill_out_level(
    grid: &mut DungeonGrid,
    rng: &mut GameRng,
    slot_rooms: &mut [Option<Room>; MAXROOMS],
    slot_kinds: &mut [SlotKind; MAXROOMS],
    level_depth: i16,
) {
    let order = mixed_room_order(rng);
    let mut r_de: Option<usize> = None;

    for rn in order {
        let kind = slot_kinds[rn];
        if kind == SlotKind::Nothing || (kind == SlotKind::Cross && coin_toss(rng)) {
            fill_it(grid, rng, slot_rooms, slot_kinds, rn, true, &mut r_de, level_depth);
        }
    }

    if let Some(de) = r_de {
        fill_it(grid, rng, slot_rooms, slot_kinds, de, false, &mut r_de, level_depth);
    }
}

pub fn generate_level_with_depth(rng: &mut GameRng, level_depth: i16, party_counter: i16) -> GeneratedLevel {
    let mut grid = DungeonGrid::new();

    let is_big_room = level_depth == party_counter && rng.rand_percent(1);
    if is_big_room {
        let big_room = make_big_room(rng, &mut grid);
        let mut slot_rooms_big: [Option<Room>; MAXROOMS] = [None; MAXROOMS];
        let mut slot_kinds_big = [SlotKind::Nothing; MAXROOMS];
        slot_rooms_big[0] = Some(big_room);
        slot_kinds_big[0] = SlotKind::Room;
        let stairs_position = place_stairs(&mut grid, rng, &slot_rooms_big, &slot_kinds_big);
        let rooms = vec![big_room];
        return GeneratedLevel { grid, rooms, stairs_position };
    }

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

    add_mazes(&mut grid, rng, &mut slot_rooms, &mut slot_kinds, level_depth);

    let room_order = mixed_room_order(rng);
    let mut connections = [[false; MAXROOMS]; MAXROOMS];

    for i in room_order {
        if i < (MAXROOMS - 1) {
            if connect_rooms(&mut grid, rng, &mut slot_rooms, i, i + 1, level_depth) {
                mark_connection(&mut connections, i, i + 1);
            }
        }
        if i < (MAXROOMS - 3) {
            if connect_rooms(&mut grid, rng, &mut slot_rooms, i, i + 3, level_depth) {
                mark_connection(&mut connections, i, i + 3);
            }
        }
        if i < (MAXROOMS - 2) && slot_rooms[i + 1].is_none() {
            if connect_rooms(&mut grid, rng, &mut slot_rooms, i, i + 2, level_depth) {
                slot_kinds[i + 1] = SlotKind::Cross;
                mark_connection(&mut connections, i, i + 2);
            }
        }
        if i < (MAXROOMS - 6) && slot_rooms[i + 3].is_none() {
            if connect_rooms(&mut grid, rng, &mut slot_rooms, i, i + 6, level_depth) {
                slot_kinds[i + 3] = SlotKind::Cross;
                mark_connection(&mut connections, i, i + 6);
            }
        }

        if are_playable_rooms_connected(&slot_rooms, &connections) {
            break;
        }
    }

    fill_out_level(&mut grid, rng, &mut slot_rooms, &mut slot_kinds, level_depth);

    let stairs_position = place_stairs(&mut grid, rng, &slot_rooms, &slot_kinds);
    let rooms: Vec<Room> = slot_rooms.into_iter().flatten().collect();

    GeneratedLevel { grid, rooms, stairs_position }
}

#[cfg(test)]
pub fn generate_level(rng: &mut GameRng) -> GeneratedLevel {
    generate_level_with_depth(rng, 1, i16::MAX)
}
