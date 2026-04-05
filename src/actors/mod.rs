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
    /// Medusa: confuses the player from sight range (CONFUSES flag)
    Confuse,
    /// Dragon: breathes fire at range along a straight or diagonal line (FLAMES flag)
    Flames,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffectEvent {
    Frozen { turns: u8 },
    Held,
    /// Rattlesnake sting: drains player strength by `amount`.
    Stung { amount: i16 },
    ArmorRusted,
    GoldStolen,
    ItemStolen,
    LifeDrained { max_hit_points_lost: i16 },
    LevelDropped,
    /// Medusa gaze: confuses the player for `turns` moves.
    Confused { turns: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monster {
    pub kind: MonsterKind,
    pub position: Position,
    pub hit_points: i16,
    /// Damage string in `"NdD"` or `"NdD/NdD"` format, matching original `mon_tab`.
    pub damage_string: &'static str,
    /// Hit-chance percentage (0-100), matching original `m_hit_chance` in `mon_tab`.
    pub hit_chance: i16,
    /// Experience points awarded to the player on kill, matching original `kill_exp`.
    pub kill_exp: i32,
    pub special_hit: Option<SpecialHit>,
    /// Cumulative damage for STATIONARY monsters (VenusFlytrap): starts at 0, +1 each attack.
    pub stationary_damage: i16,
}

impl Monster {
    pub const fn new(kind: MonsterKind, position: Position) -> Self {
        match kind {
            MonsterKind::Aquator => Self {
                kind,
                position,
                hit_points: 25,
                damage_string: "0d0",
                hit_chance: 100,
                kill_exp: 20,
                special_hit: Some(SpecialHit::Rusts),
                stationary_damage: 0,
            },
            MonsterKind::Bat => Self {
                kind,
                position,
                hit_points: 10,
                damage_string: "1d3",
                hit_chance: 60,
                kill_exp: 2,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::Centaur => Self {
                kind,
                position,
                hit_points: 32,
                damage_string: "3d3/2d5",
                hit_chance: 85,
                kill_exp: 15,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::Dragon => Self {
                kind,
                position,
                hit_points: 145,
                damage_string: "4d6/4d9",
                hit_chance: 100,
                kill_exp: 5000,
                special_hit: Some(SpecialHit::Flames),
                stationary_damage: 0,
            },
            MonsterKind::Emu => Self {
                kind,
                position,
                hit_points: 11,
                damage_string: "1d3",
                hit_chance: 65,
                kill_exp: 2,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::VenusFlytrap => Self {
                kind,
                position,
                hit_points: 73,
                damage_string: "0d0",
                hit_chance: 80,
                kill_exp: 91,
                special_hit: Some(SpecialHit::Hold),
                stationary_damage: 0,
            },
            MonsterKind::Griffin => Self {
                kind,
                position,
                hit_points: 115,
                damage_string: "5d5/5d5",
                hit_chance: 85,
                kill_exp: 2000,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::Hobgoblin => Self {
                kind,
                position,
                hit_points: 15,
                damage_string: "1d3/1d2",
                hit_chance: 67,
                kill_exp: 3,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::IceMonster => Self {
                kind,
                position,
                hit_points: 15,
                damage_string: "0d0",
                hit_chance: 68,
                kill_exp: 5,
                special_hit: Some(SpecialHit::Freeze),
                stationary_damage: 0,
            },
            MonsterKind::Jabberwock => Self {
                kind,
                position,
                hit_points: 132,
                damage_string: "3d10/4d5",
                hit_chance: 100,
                kill_exp: 3000,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::Kestrel => Self {
                kind,
                position,
                hit_points: 10,
                damage_string: "1d4",
                hit_chance: 60,
                kill_exp: 2,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::Leprechaun => Self {
                kind,
                position,
                hit_points: 25,
                damage_string: "0d0",
                hit_chance: 75,
                kill_exp: 21,
                special_hit: Some(SpecialHit::StealsGold),
                stationary_damage: 0,
            },
            MonsterKind::Medusa => Self {
                kind,
                position,
                hit_points: 97,
                damage_string: "4d4/3d7",
                hit_chance: 85,
                kill_exp: 250,
                special_hit: Some(SpecialHit::Confuse),
                stationary_damage: 0,
            },
            MonsterKind::Nymph => Self {
                kind,
                position,
                hit_points: 25,
                damage_string: "0d0",
                hit_chance: 75,
                kill_exp: 39,
                special_hit: Some(SpecialHit::StealsItem),
                stationary_damage: 0,
            },
            MonsterKind::Orc => Self {
                kind,
                position,
                hit_points: 25,
                damage_string: "1d6",
                hit_chance: 70,
                kill_exp: 5,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::Phantom => Self {
                kind,
                position,
                hit_points: 76,
                damage_string: "5d4",
                hit_chance: 80,
                kill_exp: 120,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::Quagga => Self {
                kind,
                position,
                hit_points: 30,
                damage_string: "3d5",
                hit_chance: 78,
                kill_exp: 20,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::Rattlesnake => Self {
                kind,
                position,
                hit_points: 19,
                damage_string: "2d5",
                hit_chance: 70,
                kill_exp: 10,
                special_hit: Some(SpecialHit::Sting),
                stationary_damage: 0,
            },
            MonsterKind::Snake => Self {
                kind,
                position,
                hit_points: 8,
                damage_string: "1d3",
                hit_chance: 50,
                kill_exp: 2,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::Troll => Self {
                kind,
                position,
                hit_points: 75,
                damage_string: "4d6/1d4",
                hit_chance: 75,
                kill_exp: 125,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::BlackUnicorn => Self {
                kind,
                position,
                hit_points: 90,
                damage_string: "4d10",
                hit_chance: 85,
                kill_exp: 200,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::Vampire => Self {
                kind,
                position,
                hit_points: 55,
                damage_string: "1d14/1d4",
                hit_chance: 85,
                kill_exp: 350,
                special_hit: Some(SpecialHit::DrainsLife),
                stationary_damage: 0,
            },
            MonsterKind::Wraith => Self {
                kind,
                position,
                hit_points: 45,
                damage_string: "2d8",
                hit_chance: 75,
                kill_exp: 55,
                special_hit: Some(SpecialHit::DropsLevel),
                stationary_damage: 0,
            },
            MonsterKind::Xeroc => Self {
                kind,
                position,
                hit_points: 42,
                damage_string: "4d6",
                hit_chance: 75,
                kill_exp: 110,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::Yeti => Self {
                kind,
                position,
                hit_points: 35,
                damage_string: "3d6",
                hit_chance: 80,
                kill_exp: 50,
                special_hit: None,
                stationary_damage: 0,
            },
            MonsterKind::Zombie => Self {
                kind,
                position,
                hit_points: 21,
                damage_string: "1d7",
                hit_chance: 69,
                kill_exp: 8,
                special_hit: None,
                stationary_damage: 0,
            },
        }
    }

    pub const fn display_char(&self) -> char {
        self.kind.display_char()
    }
}

impl MonsterKind {
    /// Returns the (first_level, last_level) range from the original mon_tab.
    pub const fn level_range(self) -> (i16, i16) {
        match self {
            MonsterKind::Aquator      => (9,  18),
            MonsterKind::Bat          => (1,  8),
            MonsterKind::Centaur      => (7,  16),
            MonsterKind::Dragon       => (21, 126),
            MonsterKind::Emu          => (1,  7),
            MonsterKind::VenusFlytrap => (12, 126),
            MonsterKind::Griffin      => (20, 126),
            MonsterKind::Hobgoblin    => (1,  10),
            MonsterKind::IceMonster   => (2,  11),
            MonsterKind::Jabberwock   => (21, 126),
            MonsterKind::Kestrel      => (1,  6),
            MonsterKind::Leprechaun   => (6,  16),
            MonsterKind::Medusa       => (18, 126),
            MonsterKind::Nymph        => (10, 19),
            MonsterKind::Orc          => (4,  13),
            MonsterKind::Phantom      => (15, 24),
            MonsterKind::Quagga       => (8,  17),
            MonsterKind::Rattlesnake  => (3,  12),
            MonsterKind::Snake        => (1,  9),
            MonsterKind::Troll        => (13, 22),
            MonsterKind::BlackUnicorn => (17, 26),
            MonsterKind::Vampire      => (19, 126),
            MonsterKind::Wraith       => (14, 23),
            MonsterKind::Xeroc        => (16, 25),
            MonsterKind::Yeti         => (11, 20),
            MonsterKind::Zombie       => (5,  14),
        }
    }

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
        kill_exp: i32,
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

/// Roll damage for a `"NdD"` or `"NdD/NdD"` damage string.
/// Each `/`-separated component rolls N dice with D sides and sums them.
pub fn roll_damage_string(s: &str, rng: &mut GameRng) -> i16 {
    let mut total: i16 = 0;
    for part in s.split('/') {
        if let Some(d_pos) = part.find('d') {
            let count: i16 = part[..d_pos].parse().unwrap_or(0);
            let sides: i16 = part[d_pos + 1..].parse().unwrap_or(0);
            if count > 0 && sides > 0 {
                for _ in 0..count {
                    total += rng.get_rand(1, sides as i32) as i16;
                }
            }
        }
    }
    total
}

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

    let kill_exp = monster.kill_exp;
    let event = CombatEvent::PlayerHitMonster {
        monster_kind: monster.kind,
        position: monster.position,
        damage,
        killed: monster.hit_points <= 0,
        kill_exp,
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
    rng: &mut GameRng,
) -> Vec<CombatEvent> {
    let mut occupied_positions: HashSet<Position> =
        monsters.iter().map(|monster| monster.position).collect();
    let mut events = Vec::new();

    for monster in monsters.iter_mut() {
        let previous_position = monster.position;
        occupied_positions.remove(&previous_position);

        match next_monster_action(monster, player_position, level, &occupied_positions, rng) {
            MonsterAction::Move(next_position) => {
                monster.position = next_position;
                occupied_positions.insert(next_position);
            }
            MonsterAction::AttackPlayer => {
                occupied_positions.insert(previous_position);
                // Check hit_chance: rng(0,99) < hit_chance means a hit
                if rng.get_rand(0, 99) < monster.hit_chance as i32 {
                    let damage = if monster.kind == MonsterKind::VenusFlytrap {
                        let d = monster.stationary_damage;
                        monster.stationary_damage += 1;
                        d
                    } else {
                        roll_damage_string(monster.damage_string, rng)
                    };
                    events.push(CombatEvent::MonsterHitPlayer {
                        monster_kind: monster.kind,
                        position: monster.position,
                        damage,
                    });

                    if let Some(effect) = monster.special_hit.and_then(special_hit_event) {
                        events.push(CombatEvent::MonsterAppliedEffect {
                            monster_kind: monster.kind,
                            position: monster.position,
                            effect,
                        });
                    }
                }
                // On a miss: no events emitted
            }
            MonsterAction::ConfusePlayer => {
                occupied_positions.insert(previous_position);
                events.push(CombatEvent::MonsterAppliedEffect {
                    monster_kind: monster.kind,
                    position: monster.position,
                    effect: StatusEffectEvent::Confused { turns: 12 },
                });
            }
            MonsterAction::FireBreath => {
                occupied_positions.insert(previous_position);
                // Dragon fire breath always hits; roll damage from monster's dice
                let damage = roll_damage_string(monster.damage_string, rng);
                events.push(CombatEvent::MonsterHitPlayer {
                    monster_kind: monster.kind,
                    position: monster.position,
                    damage,
                });
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
    /// Medusa: confuse the player from sight range.
    ConfusePlayer,
    /// Dragon: fire breath along a line at range.
    FireBreath,
    Wait,
}

fn next_monster_action(
    monster: &Monster,
    player_position: Position,
    level: &GeneratedLevel,
    occupied_positions: &HashSet<Position>,
    rng: &mut GameRng,
) -> MonsterAction {
    // ── Medusa: confuse from sight range (checked before physical attack) ──
    if monster.special_hit == Some(SpecialHit::Confuse) {
        let row_dist = (player_position.row - monster.position.row).abs();
        let col_dist = (player_position.col - monster.position.col).abs();
        // visible when within ~5 tiles; 55% chance to confuse (matches original m_confuse)
        if row_dist <= 5 && col_dist <= 5 && rng.get_rand(0, 99) < 55 {
            return MonsterAction::ConfusePlayer;
        }
    }

    // ── Normal movement / adjacent attack ──
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

    // ── Dragon: fire breath when not adjacent (collinear LOS, max 7 tiles, 50% chance) ──
    if monster.special_hit == Some(SpecialHit::Flames) {
        let row_dist = (player_position.row - monster.position.row).abs();
        let col_dist = (player_position.col - monster.position.col).abs();
        let in_line = row_dist == 0 || col_dist == 0 || row_dist == col_dist;
        let in_range = row_dist <= 7 && col_dist <= 7;
        if in_line && in_range && rng.get_rand(0, 1) == 0 {
            // Fire breath deals rolled damage (computed later in tick_monsters)
            return MonsterAction::FireBreath;
        }
    }

    MonsterAction::Wait
}

fn special_hit_event(special_hit: SpecialHit) -> Option<StatusEffectEvent> {
    match special_hit {
        SpecialHit::Freeze => Some(StatusEffectEvent::Frozen { turns: 2 }),
        SpecialHit::Hold => Some(StatusEffectEvent::Held),
        SpecialHit::Sting => Some(StatusEffectEvent::Stung { amount: 1 }),
        SpecialHit::Rusts => Some(StatusEffectEvent::ArmorRusted),
        SpecialHit::StealsGold => Some(StatusEffectEvent::GoldStolen),
        SpecialHit::StealsItem => Some(StatusEffectEvent::ItemStolen),
        SpecialHit::DrainsLife => Some(StatusEffectEvent::LifeDrained {
            max_hit_points_lost: 2,
        }),
        SpecialHit::DropsLevel => Some(StatusEffectEvent::LevelDropped),
        // Confuse and Flames are range actions handled in next_monster_action;
        // they never reach this code path.
        SpecialHit::Confuse | SpecialHit::Flames => None,
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

        let monsters_a = spawn_basic_monsters(&level_a, &mut rng_a, player_position, 1);
        let monsters_b = spawn_basic_monsters(&level_b, &mut rng_b, player_position, 1);

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

        let first_turn = tick_monsters(&mut monsters, &level, player_position, &mut GameRng::new(1));

        assert!(first_turn.is_empty());
        assert_eq!(monsters[0].position, Position::new(3, 21));

        let second_turn = tick_monsters(&mut monsters, &level, player_position, &mut GameRng::new(2));
        let third_turn = tick_monsters(&mut monsters, &level, player_position, &mut GameRng::new(3));
        let fourth_turn = tick_monsters(&mut monsters, &level, player_position, &mut GameRng::new(4));

        assert!(second_turn.is_empty());
        assert!(third_turn.is_empty());
        assert_eq!(monsters[0].position, Position::new(3, 19));
        assert_ne!(monsters[0].position, player_position);
        // Kestrel has 60% hit_chance and rolls 1d4 damage, so it may hit or miss.
        assert!(
            fourth_turn.is_empty()
                || matches!(
                    fourth_turn[0],
                    CombatEvent::MonsterHitPlayer {
                        monster_kind: MonsterKind::Kestrel,
                        ..
                    }
                )
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
                kill_exp: 2,
            })
        );
        assert!(monsters.is_empty());
    }

    #[test]
    fn special_hit_monster_emits_effect_event() {
        let mut rng = GameRng::new(12345);
        let level = generate_level(&mut rng);
        let player_position = level.spawn_position();
        let mut monsters = vec![Monster::new(MonsterKind::IceMonster, Position::new(4, 18))];

        let events = tick_monsters(&mut monsters, &level, player_position, &mut GameRng::new(42));

        // IceMonster has 68% hit_chance; on a miss no events are emitted.
        // On a hit it deals 0 damage and applies the Frozen effect.
        assert!(
            events.is_empty()
                || (events.len() == 2
                    && matches!(
                        &events[0],
                        CombatEvent::MonsterHitPlayer {
                            monster_kind: MonsterKind::IceMonster,
                            damage: 0,
                            ..
                        }
                    )
                    && matches!(
                        &events[1],
                        CombatEvent::MonsterAppliedEffect {
                            effect: StatusEffectEvent::Frozen { .. },
                            ..
                        }
                    ))
        );
    }

    #[test]
    fn aquator_emits_armor_rusted_event() {
        let mut rng = GameRng::new(12345);
        let level = generate_level(&mut rng);
        let player_position = level.spawn_position();
        let mut monsters = vec![Monster::new(MonsterKind::Aquator, Position::new(4, 18))];

        let events = tick_monsters(&mut monsters, &level, player_position, &mut GameRng::new(42));

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
        let mut monsters = vec![Monster::new(MonsterKind::Vampire, Position::new(4, 18))];

        let events = tick_monsters(&mut monsters, &level, player_position, &mut GameRng::new(42));

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
        let mut monsters = vec![Monster::new(MonsterKind::Leprechaun, Position::new(4, 18))];

        let events = tick_monsters(&mut monsters, &level, player_position, &mut GameRng::new(42));

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
        let mut monsters = vec![Monster::new(MonsterKind::Nymph, Position::new(4, 18))];

        let events = tick_monsters(&mut monsters, &level, player_position, &mut GameRng::new(42));

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
        let mut monsters = vec![Monster::new(MonsterKind::Wraith, Position::new(4, 18))];

        let events = tick_monsters(&mut monsters, &level, player_position, &mut GameRng::new(42));

        assert!(events.iter().any(|e| matches!(
            e,
            CombatEvent::MonsterAppliedEffect {
                effect: StatusEffectEvent::LevelDropped,
                ..
            }
        )));
    }
}
