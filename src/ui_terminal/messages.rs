use crate::actors::{CombatEvent, MonsterKind, StatusEffectEvent};
use crate::game_loop::GameLoop;
use crate::inventory_items::{EquipmentSlot, InventoryEvent};

pub(super) fn render_last_message(game: &GameLoop) -> String {
    if let Some(event) = game.state().last_inventory_events.last() {
        return inventory_message(event);
    }

    if let Some(message) = &game.state().last_system_message {
        return message.clone();
    }

    if let Some(event) = game.state().last_turn_events.last() {
        return combat_message(event);
    }

    if game.state().player_hit_points == 0 {
        return "You died.".to_string();
    }

    if game.state().quit_requested {
        return "Quit requested.".to_string();
    }

    if game.state().last_move_blocked {
        return "Blocked.".to_string();
    }

    "Awaiting input.".to_string()
}

fn inventory_message(event: &InventoryEvent) -> String {
    match event {
        InventoryEvent::PickedUp { name } => format!("Picked up {name}."),
        InventoryEvent::Dropped { name, position } => {
            format!("Dropped {name} at {},{}.", position.row, position.col)
        }
        InventoryEvent::Equipped { name, slot } => {
            format!("Equipped {name} in {}.", equipment_slot_name(*slot))
        }
        InventoryEvent::Unequipped { name, slot } => {
            format!("Unequipped {name} from {}.", equipment_slot_name(*slot))
        }
        InventoryEvent::Used { name } => format!("Used {name}."),
        InventoryEvent::Thrown { name } => format!("Threw {name}."),
        InventoryEvent::PackFull => "Pack full.".to_string(),
    }
}

fn combat_message(event: &CombatEvent) -> String {
    match event {
        CombatEvent::PlayerHitMonster {
            monster_kind,
            damage,
            killed,
            ..
        } => {
            if *killed {
                format!(
                    "You hit {} for {damage} and kill it.",
                    monster_name(*monster_kind)
                )
            } else {
                format!("You hit {} for {damage}.", monster_name(*monster_kind))
            }
        }
        CombatEvent::MonsterHitPlayer {
            monster_kind,
            damage,
            ..
        } => format!("{} hits you for {damage}.", monster_name(*monster_kind)),
        CombatEvent::MonsterAppliedEffect {
            monster_kind,
            effect,
            ..
        } => match effect {
            StatusEffectEvent::Frozen { turns } => {
                format!(
                    "{} freezes you for {turns} turns.",
                    monster_name(*monster_kind)
                )
            }
            StatusEffectEvent::Held => {
                format!("{} holds you in place.", monster_name(*monster_kind))
            }
            StatusEffectEvent::Stung {
                max_hit_points_lost,
            } => format!(
                "{} stings you. Max HP -{max_hit_points_lost}.",
                monster_name(*monster_kind)
            ),
            StatusEffectEvent::ArmorRusted => {
                format!("{} rusts your armor.", monster_name(*monster_kind))
            }
            StatusEffectEvent::GoldStolen => {
                format!("{} steals your gold.", monster_name(*monster_kind))
            }
            StatusEffectEvent::ItemStolen => {
                format!("{} steals an item.", monster_name(*monster_kind))
            }
            StatusEffectEvent::LifeDrained { max_hit_points_lost } => format!(
                "{} drains your life. Max HP -{max_hit_points_lost}.",
                monster_name(*monster_kind)
            ),
            StatusEffectEvent::LevelDropped => {
                format!("{} drains your experience.", monster_name(*monster_kind))
            }
        },
    }
}

fn equipment_slot_name(slot: EquipmentSlot) -> &'static str {
    match slot {
        EquipmentSlot::Weapon => "weapon hand",
        EquipmentSlot::Armor => "armor slot",
        EquipmentSlot::LeftRing => "left hand",
        EquipmentSlot::RightRing => "right hand",
    }
}

fn monster_name(kind: MonsterKind) -> &'static str {
    match kind {
        MonsterKind::Aquator => "aquator",
        MonsterKind::Bat => "bat",
        MonsterKind::Centaur => "centaur",
        MonsterKind::Dragon => "dragon",
        MonsterKind::Emu => "emu",
        MonsterKind::VenusFlytrap => "venus flytrap",
        MonsterKind::Griffin => "griffin",
        MonsterKind::Hobgoblin => "hobgoblin",
        MonsterKind::IceMonster => "ice monster",
        MonsterKind::Jabberwock => "jabberwock",
        MonsterKind::Kestrel => "kestrel",
        MonsterKind::Leprechaun => "leprechaun",
        MonsterKind::Medusa => "medusa",
        MonsterKind::Nymph => "nymph",
        MonsterKind::Orc => "orc",
        MonsterKind::Phantom => "phantom",
        MonsterKind::Quagga => "quagga",
        MonsterKind::Rattlesnake => "rattlesnake",
        MonsterKind::Snake => "snake",
        MonsterKind::Troll => "troll",
        MonsterKind::BlackUnicorn => "black unicorn",
        MonsterKind::Vampire => "vampire",
        MonsterKind::Wraith => "wraith",
        MonsterKind::Xeroc => "xeroc",
        MonsterKind::Yeti => "yeti",
        MonsterKind::Zombie => "zombie",
    }
}
