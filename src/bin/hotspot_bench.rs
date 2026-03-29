use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    row: i16,
    col: i16,
}

fn main() {
    bench_monster_occupancy_hotspot();
    bench_render_lookup_hotspot();
}

fn bench_monster_occupancy_hotspot() {
    let monsters = synthetic_monsters(300);
    let iterations = 300;

    let old_started = Instant::now();
    let old_work = old_monster_blocked_scan(&monsters, iterations);
    let old_elapsed = old_started.elapsed();

    let new_started = Instant::now();
    let new_work = new_monster_blocked_scan(&monsters, iterations);
    let new_elapsed = new_started.elapsed();

    println!(
        "monster_occupancy old_ms={} new_ms={} speedup_x={:.2} work_old={} work_new={}",
        old_elapsed.as_millis(),
        new_elapsed.as_millis(),
        old_elapsed.as_secs_f64() / new_elapsed.as_secs_f64(),
        old_work,
        new_work
    );
}

fn bench_render_lookup_hotspot() {
    let monsters = synthetic_monsters(120);
    let floor_items = synthetic_floor_items(80);
    let known_traps = synthetic_known_traps(40);
    let frames = 400;

    let old_started = Instant::now();
    let old_work = old_render_lookup_scan(&monsters, &floor_items, &known_traps, frames);
    let old_elapsed = old_started.elapsed();

    let new_started = Instant::now();
    let new_work = new_render_lookup_scan(&monsters, &floor_items, &known_traps, frames);
    let new_elapsed = new_started.elapsed();

    println!(
        "render_lookup old_ms={} new_ms={} speedup_x={:.2} work_old={} work_new={}",
        old_elapsed.as_millis(),
        new_elapsed.as_millis(),
        old_elapsed.as_secs_f64() / new_elapsed.as_secs_f64(),
        old_work,
        new_work
    );
}

fn synthetic_monsters(count: usize) -> Vec<Position> {
    (0..count)
        .map(|i| Position {
            row: ((i as i16) % 24).clamp(1, 22),
            col: ((i as i16 * 3) % 80).clamp(1, 78),
        })
        .collect()
}

fn synthetic_floor_items(count: usize) -> Vec<Position> {
    (0..count)
        .map(|i| Position {
            row: ((i as i16 * 5) % 24).clamp(1, 22),
            col: ((i as i16 * 7) % 80).clamp(1, 78),
        })
        .collect()
}

fn synthetic_known_traps(count: usize) -> Vec<Position> {
    (0..count)
        .map(|i| Position {
            row: ((i as i16 * 11) % 24).clamp(1, 22),
            col: ((i as i16 * 13) % 80).clamp(1, 78),
        })
        .collect()
}

fn old_monster_blocked_scan(monsters: &[Position], iterations: usize) -> u64 {
    let mut blocked_count = 0u64;

    for _ in 0..iterations {
        let occupied = monsters.to_vec();

        for (idx, monster) in monsters.iter().enumerate() {
            let candidates = [
                Position {
                    row: monster.row + 1,
                    col: monster.col,
                },
                Position {
                    row: monster.row,
                    col: monster.col + 1,
                },
                Position {
                    row: monster.row + 1,
                    col: monster.col + 1,
                },
            ];

            for candidate in candidates {
                let blocked = occupied
                    .iter()
                    .enumerate()
                    .any(|(other_idx, other)| other_idx != idx && *other == candidate);
                if blocked {
                    blocked_count += 1;
                }
            }
        }
    }

    blocked_count
}

fn new_monster_blocked_scan(monsters: &[Position], iterations: usize) -> u64 {
    let mut blocked_count = 0u64;

    for _ in 0..iterations {
        let occupied: HashSet<Position> = monsters.iter().copied().collect();

        for monster in monsters {
            let candidates = [
                Position {
                    row: monster.row + 1,
                    col: monster.col,
                },
                Position {
                    row: monster.row,
                    col: monster.col + 1,
                },
                Position {
                    row: monster.row + 1,
                    col: monster.col + 1,
                },
            ];

            for candidate in candidates {
                if occupied.contains(&candidate) {
                    blocked_count += 1;
                }
            }
        }
    }

    blocked_count
}

#[allow(clippy::manual_contains)]
fn old_render_lookup_scan(
    monsters: &[Position],
    floor_items: &[Position],
    known_traps: &[Position],
    frames: usize,
) -> u64 {
    let mut score = 0u64;

    for _ in 0..frames {
        for row in 0..24i16 {
            for col in 0..80i16 {
                let pos = Position { row, col };

                // old-style: linear scan per cell (intentional baseline – O(n) per cell)
                if monsters.iter().any(|m| *m == pos) {
                    score += 3;
                    continue;
                }

                if floor_items.iter().any(|f| *f == pos) {
                    score += 2;
                    continue;
                }

                if known_traps.iter().any(|t| *t == pos) {
                    score += 1;
                }
            }
        }
    }

    score
}

fn new_render_lookup_scan(
    monsters: &[Position],
    floor_items: &[Position],
    known_traps: &[Position],
    frames: usize,
) -> u64 {
    let monster_map: HashMap<Position, char> = monsters.iter().map(|p| (*p, 'M')).collect();
    let floor_map: HashMap<Position, char> = floor_items.iter().map(|p| (*p, ')')).collect();
    let trap_set: HashSet<Position> = known_traps.iter().copied().collect();

    let mut score = 0u64;

    for _ in 0..frames {
        for row in 0..24i16 {
            for col in 0..80i16 {
                let pos = Position { row, col };

                if monster_map.contains_key(&pos) {
                    score += 3;
                    continue;
                }

                if floor_map.contains_key(&pos) {
                    score += 2;
                    continue;
                }

                if trap_set.contains(&pos) {
                    score += 1;
                }
            }
        }
    }

    score
}
