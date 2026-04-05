pub mod types;
pub mod monster;
pub mod spawn;
pub mod combat;

pub use types::{MonsterKind, SpecialHit, StatusEffectEvent, CombatEvent};
pub use monster::Monster;
pub use spawn::spawn_basic_monsters;
pub use combat::{roll_damage_string, attack_monster, tick_monsters};

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
