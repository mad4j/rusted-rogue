use crate::core_types::Position;
use crate::rng::GameRng;
use crate::world_gen::GeneratedLevel;
use super::types::MonsterKind;
use super::monster::Monster;

const ALL_KINDS: [MonsterKind; 26] = [
    MonsterKind::Aquator,
    MonsterKind::Bat,
    MonsterKind::Centaur,
    MonsterKind::Dragon,
    MonsterKind::Emu,
    MonsterKind::VenusFlytrap,
    MonsterKind::Griffin,
    MonsterKind::Hobgoblin,
    MonsterKind::IceMonster,
    MonsterKind::Jabberwock,
    MonsterKind::Kestrel,
    MonsterKind::Leprechaun,
    MonsterKind::Medusa,
    MonsterKind::Nymph,
    MonsterKind::Orc,
    MonsterKind::Phantom,
    MonsterKind::Quagga,
    MonsterKind::Rattlesnake,
    MonsterKind::Snake,
    MonsterKind::Troll,
    MonsterKind::BlackUnicorn,
    MonsterKind::Vampire,
    MonsterKind::Wraith,
    MonsterKind::Xeroc,
    MonsterKind::Yeti,
    MonsterKind::Zombie,
];

/// Spawn 4–6 monsters across the whole level, matching original put_mons() in monster.c.
/// Each monster is placed at a random walkable cell at least 3 cells (Chebyshev)
/// from the player spawn position, and filtered by level_range.
pub fn spawn_basic_monsters(
    level: &GeneratedLevel,
    rng: &mut GameRng,
    player_position: Position,
    level_depth: i16,
) -> Vec<Monster> {
    let eligible: Vec<MonsterKind> = ALL_KINDS
        .into_iter()
        .filter(|kind| {
            let (first, last) = kind.level_range();
            level_depth >= first && level_depth <= last
        })
        .collect();

    if eligible.is_empty() {
        return Vec::new();
    }

    // Collect all walkable cells across the whole level.
    let (rows, cols) = level.grid.dimensions();
    let mut candidates: Vec<Position> = Vec::new();
    for row in 1..rows as i16 - 1 {
        for col in 1..cols as i16 - 1 {
            if !level.grid.is_walkable(row, col) {
                continue;
            }
            let pos = Position::new(row, col);
            // Chebyshev distance >= 3 from player spawn
            let dr = (pos.row - player_position.row).abs();
            let dc = (pos.col - player_position.col).abs();
            if dr.max(dc) >= 3 {
                candidates.push(pos);
            }
        }
    }

    if candidates.is_empty() {
        return Vec::new();
    }

    let count = rng.get_rand(4, 6) as usize;
    let mut monsters: Vec<Monster> = Vec::with_capacity(count);

    for _ in 0..count {
        if candidates.is_empty() {
            break;
        }
        // Pick a position not already occupied by a spawned monster.
        let available: Vec<usize> = (0..candidates.len())
            .filter(|&i| monsters.iter().all(|m| m.position != candidates[i]))
            .collect();
        if available.is_empty() {
            break;
        }
        let pick = rng.get_rand(0, (available.len() - 1) as i32) as usize;
        let pos = candidates[available[pick]];

        let kind_idx = rng.get_rand(0, (eligible.len() - 1) as i32) as usize;
        let kind = eligible[kind_idx];

        monsters.push(Monster::new(kind, pos));
    }

    monsters
}

