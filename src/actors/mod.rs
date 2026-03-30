use crate::core_types::Position;
use crate::rng::GameRng;
use crate::world_gen::GeneratedLevel;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterKind {
    Aquator,
    Bat,
    Centaur,
    Dragon,
    Emu,
    VenusFlytrap,
    Griffin,
    Hobgoblin,
    IceMonster,
    Jabberwock,
    Kestrel,
    Leprechaun,
    Medusa,
    Nymph,
    Orc,
    Phantom,
    Quagga,
    Rattlesnake,
    Snake,
    Troll,
    BlackUnicorn,
    Vampire,
    Wraith,
    Xeroc,
    Yeti,
    Zombie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialHit {
    Freeze,
    Hold,
    Sting,
    Rusts,
    StealsGold,
    StealsItem,
    DrainsLife,
    DropsLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffectEvent {
    Frozen { turns: u8 },
    Held,
    Stung { max_hit_points_lost: i16 },
    ArmorRusted,
    GoldStolen,
    ItemStolen,
    LifeDrained { max_hit_points_lost: i16 },
    LevelDropped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monster {
    pub kind: MonsterKind,
    pub position: Position,
    pub hit_points: i16,
    pub attack_damage: i16,
    pub special_hit: Option<SpecialHit>,
}

impl Monster {
    pub const fn new(kind: MonsterKind, position: Position) -> Self {
        match kind {
            MonsterKind::Aquator => Self {
                kind,
                position,
                hit_points: 25,
                attack_damage: 1,
                special_hit: Some(SpecialHit::Rusts),
            },
            MonsterKind::Bat => Self {
                kind,
                position,
                hit_points: 10,
                attack_damage: 3,
                special_hit: None,
            },
            MonsterKind::Centaur => Self {
                kind,
                position,
                hit_points: 32,
                attack_damage: 9,
                special_hit: None,
            },
            MonsterKind::Dragon => Self {
                kind,
                position,
                hit_points: 145,
                attack_damage: 24,
                special_hit: None,
            },
            MonsterKind::Emu => Self {
                kind,
                position,
                hit_points: 11,
                attack_damage: 3,
                special_hit: None,
            },
            MonsterKind::VenusFlytrap => Self {
                kind,
                position,
                hit_points: 4,
                attack_damage: 1,
                special_hit: Some(SpecialHit::Hold),
            },
            MonsterKind::Griffin => Self {
                kind,
                position,
                hit_points: 115,
                attack_damage: 25,
                special_hit: None,
            },
            MonsterKind::Hobgoblin => Self {
                kind,
                position,
                hit_points: 15,
                attack_damage: 3,
                special_hit: None,
            },
            MonsterKind::IceMonster => Self {
                kind,
                position,
                hit_points: 3,
                attack_damage: 1,
                special_hit: Some(SpecialHit::Freeze),
            },
            MonsterKind::Jabberwock => Self {
                kind,
                position,
                hit_points: 132,
                attack_damage: 30,
                special_hit: None,
            },
            MonsterKind::Kestrel => Self {
                kind,
                position,
                hit_points: 2,
                attack_damage: 1,
                special_hit: None,
            },
            MonsterKind::Leprechaun => Self {
                kind,
                position,
                hit_points: 25,
                attack_damage: 0,
                special_hit: Some(SpecialHit::StealsGold),
            },
            MonsterKind::Medusa => Self {
                kind,
                position,
                hit_points: 97,
                attack_damage: 16,
                special_hit: None,
            },
            MonsterKind::Nymph => Self {
                kind,
                position,
                hit_points: 25,
                attack_damage: 0,
                special_hit: Some(SpecialHit::StealsItem),
            },
            MonsterKind::Orc => Self {
                kind,
                position,
                hit_points: 25,
                attack_damage: 6,
                special_hit: None,
            },
            MonsterKind::Phantom => Self {
                kind,
                position,
                hit_points: 76,
                attack_damage: 20,
                special_hit: None,
            },
            MonsterKind::Quagga => Self {
                kind,
                position,
                hit_points: 30,
                attack_damage: 15,
                special_hit: None,
            },
            MonsterKind::Rattlesnake => Self {
                kind,
                position,
                hit_points: 2,
                attack_damage: 1,
                special_hit: Some(SpecialHit::Sting),
            },
            MonsterKind::Snake => Self {
                kind,
                position,
                hit_points: 8,
                attack_damage: 3,
                special_hit: None,
            },
            MonsterKind::Troll => Self {
                kind,
                position,
                hit_points: 75,
                attack_damage: 24,
                special_hit: None,
            },
            MonsterKind::BlackUnicorn => Self {
                kind,
                position,
                hit_points: 90,
                attack_damage: 40,
                special_hit: None,
            },
            MonsterKind::Vampire => Self {
                kind,
                position,
                hit_points: 55,
                attack_damage: 14,
                special_hit: Some(SpecialHit::DrainsLife),
            },
            MonsterKind::Wraith => Self {
                kind,
                position,
                hit_points: 45,
                attack_damage: 16,
                special_hit: Some(SpecialHit::DropsLevel),
            },
            MonsterKind::Xeroc => Self {
                kind,
                position,
                hit_points: 42,
                attack_damage: 24,
                special_hit: None,
            },
            MonsterKind::Yeti => Self {
                kind,
                position,
                hit_points: 35,
                attack_damage: 18,
                special_hit: None,
            },
            MonsterKind::Zombie => Self {
                kind,
                position,
                hit_points: 21,
                attack_damage: 7,
                special_hit: None,
            },
        }
    }

    pub const fn display_char(&self) -> char {
        self.kind.display_char()
    }
}

impl MonsterKind {
    pub const fn display_char(self) -> char {
        match self {
            MonsterKind::Aquator => 'A',
            MonsterKind::Bat => 'B',
            MonsterKind::Centaur => 'C',
            MonsterKind::Dragon => 'D',
            MonsterKind::Emu => 'E',
            MonsterKind::VenusFlytrap => 'F',
            MonsterKind::Griffin => 'G',
            MonsterKind::Hobgoblin => 'H',
            MonsterKind::IceMonster => 'I',
            MonsterKind::Jabberwock => 'J',
            MonsterKind::Kestrel => 'K',
            MonsterKind::Leprechaun => 'L',
            MonsterKind::Medusa => 'M',
            MonsterKind::Nymph => 'N',
            MonsterKind::Orc => 'O',
            MonsterKind::Phantom => 'P',
            MonsterKind::Quagga => 'Q',
            MonsterKind::Rattlesnake => 'R',
            MonsterKind::Snake => 'S',
            MonsterKind::Troll => 'T',
            MonsterKind::BlackUnicorn => 'U',
            MonsterKind::Vampire => 'V',
            MonsterKind::Wraith => 'W',
            MonsterKind::Xeroc => 'X',
            MonsterKind::Yeti => 'Y',
            MonsterKind::Zombie => 'Z',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatEvent {
    PlayerHitMonster {
        monster_kind: MonsterKind,
        position: Position,
        damage: i16,
        killed: bool,
    },
    MonsterHitPlayer {
        monster_kind: MonsterKind,
        position: Position,
        damage: i16,
    },
    MonsterAppliedEffect {
        monster_kind: MonsterKind,
        position: Position,
        effect: StatusEffectEvent,
    },
}

pub fn spawn_basic_monsters(
    level: &GeneratedLevel,
    rng: &mut GameRng,
    player_position: Position,
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

    let all_kinds = [
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
    let kind_index = rng.get_rand(0, (all_kinds.len() - 1) as i32) as usize;
    let kind = all_kinds[kind_index];

    let index = rng.get_rand(0, (candidates.len() - 1) as i32) as usize;
    vec![Monster::new(kind, candidates[index])]
}

pub fn attack_monster(
    monsters: &mut Vec<Monster>,
    target_position: Position,
    damage: i16,
) -> Option<CombatEvent> {
    let index = monsters
        .iter()
        .position(|monster| monster.position == target_position)?;

    let monster = &mut monsters[index];
    monster.hit_points -= damage;

    let event = CombatEvent::PlayerHitMonster {
        monster_kind: monster.kind,
        position: monster.position,
        damage,
        killed: monster.hit_points <= 0,
    };

    if monster.hit_points <= 0 {
        let _ = monsters.remove(index);
    }

    Some(event)
}

pub fn tick_monsters(
    monsters: &mut [Monster],
    level: &GeneratedLevel,
    player_position: Position,
) -> Vec<CombatEvent> {
    let mut occupied_positions: HashSet<Position> =
        monsters.iter().map(|monster| monster.position).collect();
    let mut events = Vec::new();

    for monster in monsters.iter_mut() {
        let previous_position = monster.position;
        occupied_positions.remove(&previous_position);

        match next_monster_action(monster, player_position, level, &occupied_positions) {
            MonsterAction::Move(next_position) => {
                monster.position = next_position;
                occupied_positions.insert(next_position);
            }
            MonsterAction::AttackPlayer => {
                occupied_positions.insert(previous_position);
                events.push(CombatEvent::MonsterHitPlayer {
                    monster_kind: monster.kind,
                    position: monster.position,
                    damage: monster.attack_damage,
                });

                if let Some(effect) = monster.special_hit.and_then(special_hit_event) {
                    events.push(CombatEvent::MonsterAppliedEffect {
                        monster_kind: monster.kind,
                        position: monster.position,
                        effect,
                    });
                }
            }
            MonsterAction::Wait => {
                occupied_positions.insert(previous_position);
            }
        }
    }

    events
}

enum MonsterAction {
    Move(Position),
    AttackPlayer,
    Wait,
}

fn next_monster_action(
    monster: &Monster,
    player_position: Position,
    level: &GeneratedLevel,
    occupied_positions: &HashSet<Position>,
) -> MonsterAction {
    let row_step = (player_position.row - monster.position.row).signum();
    let col_step = (player_position.col - monster.position.col).signum();

    let candidates = [
        Position::new(
            monster.position.row + row_step,
            monster.position.col + col_step,
        ),
        Position::new(monster.position.row + row_step, monster.position.col),
        Position::new(monster.position.row, monster.position.col + col_step),
    ];

    for candidate in candidates {
        if candidate == monster.position {
            continue;
        }

        if candidate == player_position {
            return MonsterAction::AttackPlayer;
        }

        if !level.grid.is_walkable(candidate.row, candidate.col) {
            continue;
        }

        if !occupied_positions.contains(&candidate) {
            return MonsterAction::Move(candidate);
        }
    }

    MonsterAction::Wait
}

fn special_hit_event(special_hit: SpecialHit) -> Option<StatusEffectEvent> {
    match special_hit {
        SpecialHit::Freeze => Some(StatusEffectEvent::Frozen { turns: 2 }),
        SpecialHit::Hold => Some(StatusEffectEvent::Held),
        SpecialHit::Sting => Some(StatusEffectEvent::Stung {
            max_hit_points_lost: 1,
        }),
        SpecialHit::Rusts => Some(StatusEffectEvent::ArmorRusted),
        SpecialHit::StealsGold => Some(StatusEffectEvent::GoldStolen),
        SpecialHit::StealsItem => Some(StatusEffectEvent::ItemStolen),
        SpecialHit::DrainsLife => Some(StatusEffectEvent::LifeDrained {
            max_hit_points_lost: 2,
        }),
        SpecialHit::DropsLevel => Some(StatusEffectEvent::LevelDropped),
    }
}
#[cfg(test)]
mod tests {
    use super::{
        attack_monster, spawn_basic_monsters, tick_monsters, CombatEvent, Monster, MonsterKind,
        StatusEffectEvent,
    };
    use crate::core_types::Position;
    use crate::rng::GameRng;
    use crate::world_gen::generate_level;

    #[test]
    fn spawn_basic_monsters_is_deterministic_for_seed() {
        let mut rng_a = GameRng::new(12345);
        let mut rng_b = GameRng::new(12345);
        let level_a = generate_level(&mut rng_a);
        let level_b = generate_level(&mut rng_b);
        let player_position = level_a.spawn_position();

        let monsters_a = spawn_basic_monsters(&level_a, &mut rng_a, player_position);
        let monsters_b = spawn_basic_monsters(&level_b, &mut rng_b, player_position);

        assert_eq!(monsters_a, monsters_b);
        assert_eq!(monsters_a.len(), 1);
        assert_ne!(monsters_a[0].position, player_position);
    }

    #[test]
    fn monster_turn_moves_toward_player_without_entering_player_tile() {
        let mut rng = GameRng::new(12345);
        let level = generate_level(&mut rng);
        let player_position = level.spawn_position();
        let mut monsters = vec![Monster::new(MonsterKind::Kestrel, Position::new(3, 22))];

        let first_turn = tick_monsters(&mut monsters, &level, player_position);

        assert!(first_turn.is_empty());
        assert_eq!(monsters[0].position, Position::new(3, 21));

        let second_turn = tick_monsters(&mut monsters, &level, player_position);
        let third_turn = tick_monsters(&mut monsters, &level, player_position);
        let fourth_turn = tick_monsters(&mut monsters, &level, player_position);

        assert!(second_turn.is_empty());
        assert!(third_turn.is_empty());
        assert_eq!(monsters[0].position, Position::new(3, 19));
        assert_ne!(monsters[0].position, player_position);
        assert_eq!(
            fourth_turn,
            vec![CombatEvent::MonsterHitPlayer {
                monster_kind: MonsterKind::Kestrel,
                position: Position::new(3, 19),
                damage: 1,
            }]
        );
    }

    #[test]
    fn player_attack_removes_monster_at_zero_hit_points() {
        let mut monsters = vec![Monster::new(MonsterKind::Kestrel, Position::new(18, 13))];
        monsters[0].hit_points = 1;

        let event = attack_monster(&mut monsters, Position::new(18, 13), 1);

        assert_eq!(
            event,
            Some(CombatEvent::PlayerHitMonster {
                monster_kind: MonsterKind::Kestrel,
                position: Position::new(18, 13),
                damage: 1,
                killed: true,
            })
        );
        assert!(monsters.is_empty());
    }

    #[test]
    fn special_hit_monster_emits_effect_event() {
        let mut rng = GameRng::new(12345);
        let level = generate_level(&mut rng);
        let player_position = level.spawn_position();
        let mut monsters = vec![Monster::new(MonsterKind::IceMonster, Position::new(3, 19))];

        let events = tick_monsters(&mut monsters, &level, player_position);

        assert_eq!(
            events,
            vec![
                CombatEvent::MonsterHitPlayer {
                    monster_kind: MonsterKind::IceMonster,
                    position: Position::new(3, 19),
                    damage: 1,
                },
                CombatEvent::MonsterAppliedEffect {
                    monster_kind: MonsterKind::IceMonster,
                    position: Position::new(3, 19),
                    effect: StatusEffectEvent::Frozen { turns: 2 },
                },
            ]
        );
    }

    #[test]
    fn aquator_emits_armor_rusted_event() {
        let mut rng = GameRng::new(12345);
        let level = generate_level(&mut rng);
        let player_position = level.spawn_position();
        let mut monsters = vec![Monster::new(MonsterKind::Aquator, Position::new(3, 19))];

        let events = tick_monsters(&mut monsters, &level, player_position);

        assert!(events.iter().any(|e| matches!(
            e,
            CombatEvent::MonsterAppliedEffect {
                effect: StatusEffectEvent::ArmorRusted,
                ..
            }
        )));
    }

    #[test]
    fn vampire_emits_life_drained_event() {
        let mut rng = GameRng::new(12345);
        let level = generate_level(&mut rng);
        let player_position = level.spawn_position();
        let mut monsters = vec![Monster::new(MonsterKind::Vampire, Position::new(3, 19))];

        let events = tick_monsters(&mut monsters, &level, player_position);

        assert!(events.iter().any(|e| matches!(
            e,
            CombatEvent::MonsterAppliedEffect {
                effect: StatusEffectEvent::LifeDrained {
                    max_hit_points_lost: 2
                },
                ..
            }
        )));
    }

    #[test]
    fn leprechaun_emits_gold_stolen_event() {
        let mut rng = GameRng::new(12345);
        let level = generate_level(&mut rng);
        let player_position = level.spawn_position();
        let mut monsters = vec![Monster::new(MonsterKind::Leprechaun, Position::new(3, 19))];

        let events = tick_monsters(&mut monsters, &level, player_position);

        assert!(events.iter().any(|e| matches!(
            e,
            CombatEvent::MonsterAppliedEffect {
                effect: StatusEffectEvent::GoldStolen,
                ..
            }
        )));
    }

    #[test]
    fn nymph_emits_item_stolen_event() {
        let mut rng = GameRng::new(12345);
        let level = generate_level(&mut rng);
        let player_position = level.spawn_position();
        let mut monsters = vec![Monster::new(MonsterKind::Nymph, Position::new(3, 19))];

        let events = tick_monsters(&mut monsters, &level, player_position);

        assert!(events.iter().any(|e| matches!(
            e,
            CombatEvent::MonsterAppliedEffect {
                effect: StatusEffectEvent::ItemStolen,
                ..
            }
        )));
    }

    #[test]
    fn wraith_emits_level_dropped_event() {
        let mut rng = GameRng::new(12345);
        let level = generate_level(&mut rng);
        let player_position = level.spawn_position();
        let mut monsters = vec![Monster::new(MonsterKind::Wraith, Position::new(3, 19))];

        let events = tick_monsters(&mut monsters, &level, player_position);

        assert!(events.iter().any(|e| matches!(
            e,
            CombatEvent::MonsterAppliedEffect {
                effect: StatusEffectEvent::LevelDropped,
                ..
            }
        )));
    }
}
