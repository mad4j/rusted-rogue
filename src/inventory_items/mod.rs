use crate::core_types::Position;

pub const MAX_PACK_ITEMS: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemCategory {
    Weapon,
    Armor,
    Ring,
    Potion,
    Wand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    Weapon,
    Armor,
    LeftRing,
    RightRing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryItem {
    pub name: &'static str,
    pub category: ItemCategory,
    pub attack_bonus: i16,
    pub armor_bonus: i16,
}

impl InventoryItem {
    pub const fn dagger() -> Self {
        Self {
            name: "dagger",
            category: ItemCategory::Weapon,
            attack_bonus: 1,
            armor_bonus: 0,
        }
    }

    pub const fn leather_armor() -> Self {
        Self {
            name: "leather armor",
            category: ItemCategory::Armor,
            attack_bonus: 0,
            armor_bonus: 1,
        }
    }

    pub const fn protection_ring() -> Self {
        Self {
            name: "ring of protection",
            category: ItemCategory::Ring,
            attack_bonus: 0,
            armor_bonus: 1,
        }
    }

    pub const fn accuracy_ring() -> Self {
        Self {
            name: "ring of accuracy",
            category: ItemCategory::Ring,
            attack_bonus: 1,
            armor_bonus: 0,
        }
    }

    pub const fn healing_potion() -> Self {
        Self {
            name: "healing potion",
            category: ItemCategory::Potion,
            attack_bonus: 0,
            armor_bonus: 0,
        }
    }

    pub const fn magic_missile_wand() -> Self {
        Self {
            name: "wand of magic missile",
            category: ItemCategory::Wand,
            attack_bonus: 0,
            armor_bonus: 0,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "dagger" => Some(Self::dagger()),
            "leather armor" => Some(Self::leather_armor()),
            "ring of protection" => Some(Self::protection_ring()),
            "ring of accuracy" => Some(Self::accuracy_ring()),
            "healing potion" => Some(Self::healing_potion()),
            "wand of magic missile" => Some(Self::magic_missile_wand()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryEntry {
    pub id: u64,
    pub item: InventoryItem,
    pub equipped_slot: Option<EquipmentSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloorItem {
    pub item: InventoryItem,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryEvent {
    PickedUp {
        name: &'static str,
    },
    Dropped {
        name: &'static str,
        position: Position,
    },
    Equipped {
        name: &'static str,
        slot: EquipmentSlot,
    },
    Unequipped {
        name: &'static str,
        slot: EquipmentSlot,
    },
    Used {
        name: &'static str,
    },
    Thrown {
        name: &'static str,
    },
    PackFull,
}

pub fn apply_item_effects() {}

pub fn pick_up_item(
    inventory: &mut Vec<InventoryEntry>,
    floor_items: &mut Vec<FloorItem>,
    next_item_id: &mut u64,
    position: Position,
) -> Option<InventoryEvent> {
    if inventory.len() >= MAX_PACK_ITEMS {
        return Some(InventoryEvent::PackFull);
    }

    let index = floor_items
        .iter()
        .position(|floor_item| floor_item.position == position)?;
    let floor_item = floor_items.remove(index);

    inventory.push(InventoryEntry {
        id: *next_item_id,
        item: floor_item.item.clone(),
        equipped_slot: None,
    });
    *next_item_id += 1;

    Some(InventoryEvent::PickedUp {
        name: floor_item.item.name,
    })
}

pub fn drop_first_item(
    inventory: &mut Vec<InventoryEntry>,
    floor_items: &mut Vec<FloorItem>,
    position: Position,
) -> Option<Vec<InventoryEvent>> {
    if floor_items
        .iter()
        .any(|floor_item| floor_item.position == position)
    {
        return None;
    }

    let mut entry = inventory.pop()?;
    let mut events = Vec::new();

    if let Some(slot) = entry.equipped_slot.take() {
        events.push(InventoryEvent::Unequipped {
            name: entry.item.name,
            slot,
        });
    }

    floor_items.push(FloorItem {
        item: entry.item.clone(),
        position,
    });
    events.push(InventoryEvent::Dropped {
        name: entry.item.name,
        position,
    });

    Some(events)
}

pub fn equip_first_weapon(inventory: &mut [InventoryEntry]) -> Option<Vec<InventoryEvent>> {
    equip_first_matching(inventory, ItemCategory::Weapon, &[EquipmentSlot::Weapon])
}

pub fn equip_first_armor(inventory: &mut [InventoryEntry]) -> Option<Vec<InventoryEvent>> {
    equip_first_matching(inventory, ItemCategory::Armor, &[EquipmentSlot::Armor])
}

pub fn put_on_first_ring(inventory: &mut [InventoryEntry]) -> Option<Vec<InventoryEvent>> {
    equip_first_matching(
        inventory,
        ItemCategory::Ring,
        &[EquipmentSlot::LeftRing, EquipmentSlot::RightRing],
    )
}

pub fn unequip_armor(inventory: &mut [InventoryEntry]) -> Option<InventoryEvent> {
    unequip_slot(inventory, EquipmentSlot::Armor)
}

pub fn remove_ring(inventory: &mut [InventoryEntry]) -> Option<InventoryEvent> {
    unequip_slot(inventory, EquipmentSlot::RightRing)
        .or_else(|| unequip_slot(inventory, EquipmentSlot::LeftRing))
}

pub fn total_attack_bonus(inventory: &[InventoryEntry]) -> i16 {
    inventory
        .iter()
        .filter(|entry| entry.equipped_slot.is_some())
        .map(|entry| entry.item.attack_bonus)
        .sum()
}

pub fn total_armor_bonus(inventory: &[InventoryEntry]) -> i16 {
    inventory
        .iter()
        .filter(|entry| entry.equipped_slot.is_some())
        .map(|entry| entry.item.armor_bonus)
        .sum()
}

pub fn remove_first_item_by_category(
    inventory: &mut Vec<InventoryEntry>,
    category: ItemCategory,
) -> Option<InventoryEntry> {
    let index = inventory
        .iter()
        .position(|entry| entry.item.category == category)?;
    Some(inventory.remove(index))
}

fn equip_first_matching(
    inventory: &mut [InventoryEntry],
    category: ItemCategory,
    preferred_slots: &[EquipmentSlot],
) -> Option<Vec<InventoryEvent>> {
    let target_index = inventory
        .iter()
        .position(|entry| entry.item.category == category && entry.equipped_slot.is_none())?;

    let slot = preferred_slots
        .iter()
        .copied()
        .find(|candidate| {
            inventory
                .iter()
                .all(|entry| entry.equipped_slot != Some(*candidate))
        })
        .unwrap_or(preferred_slots[0]);

    let mut events = Vec::new();
    if let Some(existing) = inventory
        .iter_mut()
        .find(|entry| entry.equipped_slot == Some(slot))
    {
        existing.equipped_slot = None;
        events.push(InventoryEvent::Unequipped {
            name: existing.item.name,
            slot,
        });
    }

    inventory[target_index].equipped_slot = Some(slot);
    events.push(InventoryEvent::Equipped {
        name: inventory[target_index].item.name,
        slot,
    });

    Some(events)
}

fn unequip_slot(inventory: &mut [InventoryEntry], slot: EquipmentSlot) -> Option<InventoryEvent> {
    let entry = inventory
        .iter_mut()
        .find(|entry| entry.equipped_slot == Some(slot))?;
    entry.equipped_slot = None;
    Some(InventoryEvent::Unequipped {
        name: entry.item.name,
        slot,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        drop_first_item, equip_first_armor, equip_first_weapon, pick_up_item, put_on_first_ring,
        remove_first_item_by_category, remove_ring, total_armor_bonus, total_attack_bonus,
        EquipmentSlot, FloorItem, InventoryEntry, InventoryEvent, InventoryItem, ItemCategory,
    };
    use crate::core_types::Position;

    #[test]
    fn picking_up_item_moves_it_into_inventory() {
        let mut inventory = Vec::new();
        let mut floor_items = vec![FloorItem {
            item: InventoryItem::dagger(),
            position: Position::new(10, 10),
        }];
        let mut next_item_id = 1;

        let event = pick_up_item(
            &mut inventory,
            &mut floor_items,
            &mut next_item_id,
            Position::new(10, 10),
        );

        assert_eq!(event, Some(InventoryEvent::PickedUp { name: "dagger" }));
        assert_eq!(inventory.len(), 1);
        assert!(floor_items.is_empty());
    }

    #[test]
    fn equipping_items_updates_bonuses() {
        let mut inventory = vec![
            InventoryEntry {
                id: 1,
                item: InventoryItem::dagger(),
                equipped_slot: None,
            },
            InventoryEntry {
                id: 2,
                item: InventoryItem::leather_armor(),
                equipped_slot: None,
            },
            InventoryEntry {
                id: 3,
                item: InventoryItem::accuracy_ring(),
                equipped_slot: None,
            },
        ];

        assert!(equip_first_weapon(&mut inventory).is_some());
        assert!(equip_first_armor(&mut inventory).is_some());
        assert!(put_on_first_ring(&mut inventory).is_some());

        assert_eq!(total_attack_bonus(&inventory), 2);
        assert_eq!(total_armor_bonus(&inventory), 1);
        assert_eq!(inventory[0].equipped_slot, Some(EquipmentSlot::Weapon));
        assert_eq!(inventory[1].equipped_slot, Some(EquipmentSlot::Armor));
    }

    #[test]
    fn dropping_equipped_item_unequips_it() {
        let mut inventory = vec![InventoryEntry {
            id: 1,
            item: InventoryItem::dagger(),
            equipped_slot: Some(EquipmentSlot::Weapon),
        }];
        let mut floor_items = Vec::new();

        let events = drop_first_item(&mut inventory, &mut floor_items, Position::new(5, 5));

        assert_eq!(
            events,
            Some(vec![
                InventoryEvent::Unequipped {
                    name: "dagger",
                    slot: EquipmentSlot::Weapon,
                },
                InventoryEvent::Dropped {
                    name: "dagger",
                    position: Position::new(5, 5),
                },
            ])
        );
        assert!(inventory.is_empty());
        assert_eq!(floor_items.len(), 1);
    }

    #[test]
    fn remove_ring_prefers_right_hand() {
        let mut inventory = vec![
            InventoryEntry {
                id: 1,
                item: InventoryItem::protection_ring(),
                equipped_slot: Some(EquipmentSlot::LeftRing),
            },
            InventoryEntry {
                id: 2,
                item: InventoryItem::accuracy_ring(),
                equipped_slot: Some(EquipmentSlot::RightRing),
            },
        ];

        let event = remove_ring(&mut inventory);

        assert_eq!(
            event,
            Some(InventoryEvent::Unequipped {
                name: "ring of accuracy",
                slot: EquipmentSlot::RightRing,
            })
        );
    }

    #[test]
    fn remove_first_item_by_category_finds_consumables() {
        let mut inventory = vec![
            InventoryEntry {
                id: 1,
                item: InventoryItem::dagger(),
                equipped_slot: None,
            },
            InventoryEntry {
                id: 2,
                item: InventoryItem::healing_potion(),
                equipped_slot: None,
            },
        ];

        let removed = remove_first_item_by_category(&mut inventory, ItemCategory::Potion)
            .expect("potion should be removable");

        assert_eq!(removed.item.name, "healing potion");
        assert_eq!(inventory.len(), 1);
    }
}
