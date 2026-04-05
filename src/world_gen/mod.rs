mod generator;
mod maze;
mod passage;
mod rooms;
mod slots;
mod types;

pub use generator::{generate_level, generate_level_with_depth};
pub use types::{DoorLink, DungeonGrid, GeneratedLevel, Room};

#[cfg(test)]
mod tests {
    use std::collections::{HashSet, VecDeque};

    use proptest::prelude::*;

    use super::{generate_level, generate_level_with_depth, DungeonGrid, Room};
    use super::maze::maze_percent_for_level;
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
        assert_eq!(grid.get(0, DCOLS as i16), None);
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

        let a = generate_level_with_depth(&mut rng_a, 20, i16::MAX);
        let b = generate_level_with_depth(&mut rng_b, 20, i16::MAX);

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
            let low = generate_level_with_depth(&mut low_rng, 2, i16::MAX);

            let mut high_rng = GameRng::new(seed);
            let high = generate_level_with_depth(&mut high_rng, 20, i16::MAX);

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