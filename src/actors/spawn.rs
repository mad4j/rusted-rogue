use crate::core_types::Position;
use crate::rng::GameRng;
use crate::world_gen::GeneratedLevel;
use super::types::MonsterKind;
use super::monster::Monster;

pub fn spawn_basic_monsters(
    level: &GeneratedLevel,
    rng: &mut GameRng,
    player_position: Position,
    level_depth: i16,
) -> Vec<Monster> {
    let Some(room) = level.rooms.first().copied() else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    for row in (room.top_row + 1)..room.bottom_row {
        for col in (room.left_col + 1)..room.right_col {
            let position = Position::new(row, col);
            let distance = (position.row - player_position.row).abs()
                + (position.col - player_position.col).abs();

            if position != player_position && distance >= 3 && level.grid.is_walkable(row, col) {
                candidates.push(position);
            }
        }
    }

    if candidates.is_empty() {
        return Vec::new();
    }

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

    let kind_index = rng.get_rand(0, (eligible.len() - 1) as i32) as usize;
    let kind = eligible[kind_index];

    let index = rng.get_rand(0, (candidates.len() - 1) as i32) as usize;
    vec![Monster::new(kind, candidates[index])]
}
