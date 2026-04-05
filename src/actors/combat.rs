use std::collections::HashSet;
use crate::core_types::Position;
use crate::rng::GameRng;
use crate::world_gen::GeneratedLevel;
use super::types::{MonsterKind, SpecialHit, StatusEffectEvent, CombatEvent};
use super::monster::Monster;

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
