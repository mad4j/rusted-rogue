use crate::core_types::Position;
use crate::rng::GameRng;

pub const MAX_PACK_ITEMS: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemCategory {
    Weapon,
    Armor,
    Ring,
    Potion,
    Wand,
    Scroll,
    Food,
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

    // ── Scrolls ──────────────────────────────────────────────────────────────

    pub const fn scroll_protect_armor() -> Self {
        Self { name: "scroll of protect armor", category: ItemCategory::Scroll, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn scroll_hold_monster() -> Self {
        Self { name: "scroll of hold monster", category: ItemCategory::Scroll, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn scroll_enchant_weapon() -> Self {
        Self { name: "scroll of enchant weapon", category: ItemCategory::Scroll, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn scroll_enchant_armor() -> Self {
        Self { name: "scroll of enchant armor", category: ItemCategory::Scroll, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn scroll_identify() -> Self {
        Self { name: "scroll of identify", category: ItemCategory::Scroll, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn scroll_teleport() -> Self {
        Self { name: "scroll of teleport", category: ItemCategory::Scroll, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn scroll_sleep() -> Self {
        Self { name: "scroll of sleep", category: ItemCategory::Scroll, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn scroll_scare_monster() -> Self {
        Self { name: "scroll of scare monster", category: ItemCategory::Scroll, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn scroll_remove_curse() -> Self {
        Self { name: "scroll of remove curse", category: ItemCategory::Scroll, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn scroll_create_monster() -> Self {
        Self { name: "scroll of create monster", category: ItemCategory::Scroll, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn scroll_aggravate_monster() -> Self {
        Self { name: "scroll of aggravate monster", category: ItemCategory::Scroll, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn scroll_magic_mapping() -> Self {
        Self { name: "scroll of magic mapping", category: ItemCategory::Scroll, attack_bonus: 0, armor_bonus: 0 }
    }

    // ── Potions ──────────────────────────────────────────────────────────────

    pub const fn potion_increase_strength() -> Self {
        Self { name: "potion of increase strength", category: ItemCategory::Potion, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn potion_restore_strength() -> Self {
        Self { name: "potion of restore strength", category: ItemCategory::Potion, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn potion_extra_healing() -> Self {
        Self { name: "potion of extra healing", category: ItemCategory::Potion, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn potion_poison() -> Self {
        Self { name: "potion of poison", category: ItemCategory::Potion, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn potion_raise_level() -> Self {
        Self { name: "potion of raise level", category: ItemCategory::Potion, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn potion_blindness() -> Self {
        Self { name: "potion of blindness", category: ItemCategory::Potion, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn potion_hallucination() -> Self {
        Self { name: "potion of hallucination", category: ItemCategory::Potion, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn potion_detect_monster() -> Self {
        Self { name: "potion of detect monster", category: ItemCategory::Potion, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn potion_detect_objects() -> Self {
        Self { name: "potion of detect objects", category: ItemCategory::Potion, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn potion_confusion() -> Self {
        Self { name: "potion of confusion", category: ItemCategory::Potion, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn potion_levitation() -> Self {
        Self { name: "potion of levitation", category: ItemCategory::Potion, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn potion_haste_self() -> Self {
        Self { name: "potion of haste self", category: ItemCategory::Potion, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn potion_see_invisible() -> Self {
        Self { name: "potion of see invisible", category: ItemCategory::Potion, attack_bonus: 0, armor_bonus: 0 }
    }

    // ── Wands ────────────────────────────────────────────────────────────────

    pub const fn wand_tele_away() -> Self {
        Self { name: "wand of teleportation", category: ItemCategory::Wand, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn wand_slow_monster() -> Self {
        Self { name: "wand of slow monster", category: ItemCategory::Wand, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn wand_confuse_monster() -> Self {
        Self { name: "wand of confuse monster", category: ItemCategory::Wand, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn wand_invisibility() -> Self {
        Self { name: "wand of invisibility", category: ItemCategory::Wand, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn wand_polymorph() -> Self {
        Self { name: "wand of polymorph", category: ItemCategory::Wand, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn wand_haste_monster() -> Self {
        Self { name: "wand of haste monster", category: ItemCategory::Wand, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn wand_put_to_sleep() -> Self {
        Self { name: "wand of sleep", category: ItemCategory::Wand, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn wand_cancellation() -> Self {
        Self { name: "wand of cancellation", category: ItemCategory::Wand, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn wand_do_nothing() -> Self {
        Self { name: "wand of nothing", category: ItemCategory::Wand, attack_bonus: 0, armor_bonus: 0 }
    }

    // ── Rings ────────────────────────────────────────────────────────────────

    pub const fn ring_stealth() -> Self {
        Self { name: "ring of stealth", category: ItemCategory::Ring, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn ring_teleportation() -> Self {
        Self { name: "ring of teleportation", category: ItemCategory::Ring, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn ring_regeneration() -> Self {
        Self { name: "ring of regeneration", category: ItemCategory::Ring, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn ring_slow_digestion() -> Self {
        Self { name: "ring of slow digestion", category: ItemCategory::Ring, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn ring_add_strength() -> Self {
        Self { name: "ring of add strength", category: ItemCategory::Ring, attack_bonus: 1, armor_bonus: 0 }
    }

    pub const fn ring_sustain_strength() -> Self {
        Self { name: "ring of sustain strength", category: ItemCategory::Ring, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn ring_dexterity() -> Self {
        Self { name: "ring of dexterity", category: ItemCategory::Ring, attack_bonus: 1, armor_bonus: 0 }
    }

    pub const fn ring_adornment() -> Self {
        Self { name: "ring of adornment", category: ItemCategory::Ring, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn ring_see_invisible() -> Self {
        Self { name: "ring of see invisible", category: ItemCategory::Ring, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn ring_maintain_armor() -> Self {
        Self { name: "ring of maintain armor", category: ItemCategory::Ring, attack_bonus: 0, armor_bonus: 1 }
    }

    pub const fn ring_searching() -> Self {
        Self { name: "ring of searching", category: ItemCategory::Ring, attack_bonus: 0, armor_bonus: 0 }
    }

    // ── Weapons ──────────────────────────────────────────────────────────────

    pub const fn bow() -> Self {
        Self { name: "bow", category: ItemCategory::Weapon, attack_bonus: 1, armor_bonus: 0 }
    }

    pub const fn dart() -> Self {
        Self { name: "dart", category: ItemCategory::Weapon, attack_bonus: 1, armor_bonus: 0 }
    }

    pub const fn arrow() -> Self {
        Self { name: "arrow", category: ItemCategory::Weapon, attack_bonus: 1, armor_bonus: 0 }
    }

    pub const fn shuriken() -> Self {
        Self { name: "shuriken", category: ItemCategory::Weapon, attack_bonus: 2, armor_bonus: 0 }
    }

    pub const fn mace() -> Self {
        Self { name: "mace", category: ItemCategory::Weapon, attack_bonus: 3, armor_bonus: 0 }
    }

    pub const fn long_sword() -> Self {
        Self { name: "long sword", category: ItemCategory::Weapon, attack_bonus: 4, armor_bonus: 0 }
    }

    pub const fn two_handed_sword() -> Self {
        Self { name: "two-handed sword", category: ItemCategory::Weapon, attack_bonus: 5, armor_bonus: 0 }
    }

    // ── Armor ────────────────────────────────────────────────────────────────

    pub const fn ring_mail() -> Self {
        Self { name: "ring mail", category: ItemCategory::Armor, attack_bonus: 0, armor_bonus: 2 }
    }

    pub const fn scale_armor() -> Self {
        Self { name: "scale armor", category: ItemCategory::Armor, attack_bonus: 0, armor_bonus: 3 }
    }

    pub const fn chain_mail() -> Self {
        Self { name: "chain mail", category: ItemCategory::Armor, attack_bonus: 0, armor_bonus: 4 }
    }

    pub const fn banded_mail() -> Self {
        Self { name: "banded mail", category: ItemCategory::Armor, attack_bonus: 0, armor_bonus: 5 }
    }

    pub const fn splint_mail() -> Self {
        Self { name: "splint mail", category: ItemCategory::Armor, attack_bonus: 0, armor_bonus: 6 }
    }

    pub const fn plate_armor() -> Self {
        Self { name: "plate armor", category: ItemCategory::Armor, attack_bonus: 0, armor_bonus: 7 }
    }

    // ── Food ─────────────────────────────────────────────────────────────────

    pub const fn food_ration() -> Self {
        Self { name: "food ration", category: ItemCategory::Food, attack_bonus: 0, armor_bonus: 0 }
    }

    pub const fn slime_mold() -> Self {
        Self { name: "slime-mold", category: ItemCategory::Food, attack_bonus: 0, armor_bonus: 0 }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            // weapons
            "dagger" => Some(Self::dagger()),
            "bow" => Some(Self::bow()),
            "dart" => Some(Self::dart()),
            "arrow" => Some(Self::arrow()),
            "shuriken" => Some(Self::shuriken()),
            "mace" => Some(Self::mace()),
            "long sword" => Some(Self::long_sword()),
            "two-handed sword" => Some(Self::two_handed_sword()),
            // armor
            "leather armor" => Some(Self::leather_armor()),
            "ring mail" => Some(Self::ring_mail()),
            "scale armor" => Some(Self::scale_armor()),
            "chain mail" => Some(Self::chain_mail()),
            "banded mail" => Some(Self::banded_mail()),
            "splint mail" => Some(Self::splint_mail()),
            "plate armor" => Some(Self::plate_armor()),
            // rings
            "ring of protection" => Some(Self::protection_ring()),
            "ring of accuracy" => Some(Self::accuracy_ring()),
            "ring of stealth" => Some(Self::ring_stealth()),
            "ring of teleportation" => Some(Self::ring_teleportation()),
            "ring of regeneration" => Some(Self::ring_regeneration()),
            "ring of slow digestion" => Some(Self::ring_slow_digestion()),
            "ring of add strength" => Some(Self::ring_add_strength()),
            "ring of sustain strength" => Some(Self::ring_sustain_strength()),
            "ring of dexterity" => Some(Self::ring_dexterity()),
            "ring of adornment" => Some(Self::ring_adornment()),
            "ring of see invisible" => Some(Self::ring_see_invisible()),
            "ring of maintain armor" => Some(Self::ring_maintain_armor()),
            "ring of searching" => Some(Self::ring_searching()),
            // potions
            "healing potion" => Some(Self::healing_potion()),
            "potion of increase strength" => Some(Self::potion_increase_strength()),
            "potion of restore strength" => Some(Self::potion_restore_strength()),
            "potion of extra healing" => Some(Self::potion_extra_healing()),
            "potion of poison" => Some(Self::potion_poison()),
            "potion of raise level" => Some(Self::potion_raise_level()),
            "potion of blindness" => Some(Self::potion_blindness()),
            "potion of hallucination" => Some(Self::potion_hallucination()),
            "potion of detect monster" => Some(Self::potion_detect_monster()),
            "potion of detect objects" => Some(Self::potion_detect_objects()),
            "potion of confusion" => Some(Self::potion_confusion()),
            "potion of levitation" => Some(Self::potion_levitation()),
            "potion of haste self" => Some(Self::potion_haste_self()),
            "potion of see invisible" => Some(Self::potion_see_invisible()),
            // wands
            "wand of magic missile" => Some(Self::magic_missile_wand()),
            "wand of teleportation" => Some(Self::wand_tele_away()),
            "wand of slow monster" => Some(Self::wand_slow_monster()),
            "wand of confuse monster" => Some(Self::wand_confuse_monster()),
            "wand of invisibility" => Some(Self::wand_invisibility()),
            "wand of polymorph" => Some(Self::wand_polymorph()),
            "wand of haste monster" => Some(Self::wand_haste_monster()),
            "wand of sleep" => Some(Self::wand_put_to_sleep()),
            "wand of cancellation" => Some(Self::wand_cancellation()),
            "wand of nothing" => Some(Self::wand_do_nothing()),
            // scrolls
            "scroll of protect armor" => Some(Self::scroll_protect_armor()),
            "scroll of hold monster" => Some(Self::scroll_hold_monster()),
            "scroll of enchant weapon" => Some(Self::scroll_enchant_weapon()),
            "scroll of enchant armor" => Some(Self::scroll_enchant_armor()),
            "scroll of identify" => Some(Self::scroll_identify()),
            "scroll of teleport" => Some(Self::scroll_teleport()),
            "scroll of sleep" => Some(Self::scroll_sleep()),
            "scroll of scare monster" => Some(Self::scroll_scare_monster()),
            "scroll of remove curse" => Some(Self::scroll_remove_curse()),
            "scroll of create monster" => Some(Self::scroll_create_monster()),
            "scroll of aggravate monster" => Some(Self::scroll_aggravate_monster()),
            "scroll of magic mapping" => Some(Self::scroll_magic_mapping()),
            // food
            "food ration" => Some(Self::food_ration()),
            "slime-mold" => Some(Self::slime_mold()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryEntry {
    pub id: u64,
    pub item: InventoryItem,
    pub equipped_slot: Option<EquipmentSlot>,
    /// Pack letter assigned on pick-up, mirroring the original `ichar` field.
    pub ichar: char,
    /// Stack size (1 for most items; >1 for stackable items like arrows).
    pub quantity: u16,
    /// Quiver id for stackable weapons (arrow, dagger, dart, shuriken).
    /// Two weapon stacks with different quiver ids never merge, matching the
    /// original `quiver` field in `struct obj`.
    pub quiver: Option<u8>,
}

/// Returns the next available pack letter ('a'–'z') not already used by an
/// inventory entry.  Falls back to 'a' if all slots are taken (pack full).
pub fn next_avail_ichar(inventory: &[InventoryEntry]) -> char {
    let used: std::collections::HashSet<char> = inventory.iter().map(|e| e.ichar).collect();
    ('a'..='z').find(|c| !used.contains(c)).unwrap_or('a')
}

/// True for weapon kinds that stack (arrow, dagger, dart, shuriken) — mirrors
/// the `check_duplicate` logic in the original pack.c.
pub fn is_stackable_weapon(item: &InventoryItem) -> bool {
    item.category == ItemCategory::Weapon
        && matches!(item.name, "arrow" | "dagger" | "dart" | "shuriken")
}

/// Compute the floor quantity and quiver id for an item about to be placed on
/// the dungeon floor.  Stackable weapons get a random quantity (3–15) and a
/// random quiver id (0–126), matching gr_weapon() in the original object.c.
/// Everything else has quantity = 1 and no quiver.
pub fn floor_quantity_and_quiver(item: &InventoryItem, rng: &mut GameRng) -> (u16, Option<u8>) {
    if is_stackable_weapon(item) {
        (rng.get_rand(3, 15) as u16, Some(rng.get_rand(0, 126) as u8))
    } else {
        (1, None)
    }
}

/// Find an existing pack entry that `floor_item` would condense into, matching
/// the `check_duplicate` logic from pack.c:
///   - FOOD (except "slime-mold"), SCROLL, POTION: merge by name.
///   - WEAPON (arrow/dagger/dart/shuriken): merge by name AND quiver id.
fn find_stackable_entry<'a>(
    inventory: &'a mut Vec<InventoryEntry>,
    floor_item: &FloorItem,
) -> Option<&'a mut InventoryEntry> {
    let item = &floor_item.item;
    match item.category {
        ItemCategory::Food if item.name != "slime-mold" => {
            inventory.iter_mut().find(|e| e.item.name == item.name)
        }
        ItemCategory::Scroll | ItemCategory::Potion => {
            inventory.iter_mut().find(|e| e.item.name == item.name)
        }
        ItemCategory::Weapon if is_stackable_weapon(item) => inventory
            .iter_mut()
            .find(|e| e.item.name == item.name && e.quiver == floor_item.quiver),
        _ => None,
    }
}

/// Generate a random item for floor placement, matching original gr_object() in object.c.
/// Roll 1–91: 1–30=Scroll, 31–60=Potion, 61–64=Wand, 65–74=Weapon,
///            75–83=Armor, 84–88=Food, 89–91=Ring.
pub fn gr_floor_item(rng: &mut GameRng) -> InventoryItem {
    let roll = rng.get_rand(1, 91);
    match roll {
        1..=30  => gr_scroll(rng),
        31..=60 => gr_potion(rng),
        61..=64 => gr_wand(rng),
        65..=74 => gr_weapon(rng),
        75..=83 => gr_armor(rng),
        84..=88 => gr_food(rng),
        _       => gr_ring(rng),
    }
}

fn gr_scroll(rng: &mut GameRng) -> InventoryItem {
    match rng.get_rand(0, 85) {
        0..=5   => InventoryItem::scroll_protect_armor(),
        6..=11  => InventoryItem::scroll_hold_monster(),
        12..=20 => InventoryItem::scroll_create_monster(),
        21..=35 => InventoryItem::scroll_identify(),
        36..=43 => InventoryItem::scroll_teleport(),
        44..=50 => InventoryItem::scroll_sleep(),
        51..=55 => InventoryItem::scroll_scare_monster(),
        56..=64 => InventoryItem::scroll_remove_curse(),
        65..=69 => InventoryItem::scroll_enchant_armor(),
        70..=74 => InventoryItem::scroll_enchant_weapon(),
        75..=80 => InventoryItem::scroll_aggravate_monster(),
        _       => InventoryItem::scroll_magic_mapping(),
    }
}

fn gr_potion(rng: &mut GameRng) -> InventoryItem {
    match rng.get_rand(1, 118) {
        1..=10  => InventoryItem::potion_increase_strength(),
        11..=20 => InventoryItem::potion_restore_strength(),
        21..=35 => InventoryItem::healing_potion(),
        36..=50 => InventoryItem::potion_extra_healing(),
        51..=55 => InventoryItem::potion_poison(),
        56..=58 => InventoryItem::potion_raise_level(),
        59..=65 => InventoryItem::potion_blindness(),
        66..=75 => InventoryItem::potion_hallucination(),
        76..=80 => InventoryItem::potion_detect_monster(),
        81..=85 => InventoryItem::potion_detect_objects(),
        86..=95 => InventoryItem::potion_confusion(),
        96..=100 => InventoryItem::potion_levitation(),
        101..=110 => InventoryItem::potion_haste_self(),
        _        => InventoryItem::potion_see_invisible(),
    }
}

fn gr_wand(rng: &mut GameRng) -> InventoryItem {
    match rng.get_rand(0, 9) {
        0 => InventoryItem::wand_tele_away(),
        1 => InventoryItem::wand_slow_monster(),
        2 => InventoryItem::wand_confuse_monster(),
        3 => InventoryItem::wand_invisibility(),
        4 => InventoryItem::wand_polymorph(),
        5 => InventoryItem::wand_haste_monster(),
        6 => InventoryItem::wand_put_to_sleep(),
        7 => InventoryItem::magic_missile_wand(),
        8 => InventoryItem::wand_cancellation(),
        _ => InventoryItem::wand_do_nothing(),
    }
}

fn gr_weapon(rng: &mut GameRng) -> InventoryItem {
    match rng.get_rand(0, 7) {
        0 => InventoryItem::bow(),
        1 => InventoryItem::dart(),
        2 => InventoryItem::arrow(),
        3 => InventoryItem::dagger(),
        4 => InventoryItem::shuriken(),
        5 => InventoryItem::mace(),
        6 => InventoryItem::long_sword(),
        _ => InventoryItem::two_handed_sword(),
    }
}

fn gr_armor(rng: &mut GameRng) -> InventoryItem {
    match rng.get_rand(0, 6) {
        0 => InventoryItem::leather_armor(),
        1 => InventoryItem::ring_mail(),
        2 => InventoryItem::scale_armor(),
        3 => InventoryItem::chain_mail(),
        4 => InventoryItem::banded_mail(),
        5 => InventoryItem::splint_mail(),
        _ => InventoryItem::plate_armor(),
    }
}

fn gr_food(rng: &mut GameRng) -> InventoryItem {
    if rng.rand_percent(75) {
        InventoryItem::food_ration()
    } else {
        InventoryItem::slime_mold()
    }
}

fn gr_ring(rng: &mut GameRng) -> InventoryItem {
    match rng.get_rand(0, 10) {
        0  => InventoryItem::ring_stealth(),
        1  => InventoryItem::ring_teleportation(),
        2  => InventoryItem::ring_regeneration(),
        3  => InventoryItem::ring_slow_digestion(),
        4  => InventoryItem::ring_add_strength(),
        5  => InventoryItem::ring_sustain_strength(),
        6  => InventoryItem::ring_dexterity(),
        7  => InventoryItem::ring_adornment(),
        8  => InventoryItem::ring_see_invisible(),
        9  => InventoryItem::ring_maintain_armor(),
        _  => InventoryItem::ring_searching(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloorItem {
    pub item: InventoryItem,
    pub position: Position,
    /// Stack size — > 1 for stackable weapons placed on the floor.
    pub quantity: u16,
    /// Quiver id — mirrors the original `quiver` field in `struct obj`.
    pub quiver: Option<u8>,
}

/// A pile of gold coins on the dungeon floor.  Not stored in the player's
/// pack — collected automatically when the player steps onto the tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldPile {
    pub position: Position,
    pub quantity: i64,
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

// ── Per-ichar action helpers ────────────────────────────────────────────────

/// Remove the item identified by `ch` from the inventory and return it.
pub fn remove_item_by_ichar(
    inventory: &mut Vec<InventoryEntry>,
    ch: char,
) -> Option<InventoryEntry> {
    let index = inventory.iter().position(|e| e.ichar == ch)?;
    Some(inventory.remove(index))
}

/// Equip the item identified by `ch` (weapon, armor, or ring).  Handles slot
/// conflict the same way as `equip_first_matching`.
pub fn equip_by_ichar(
    inventory: &mut Vec<InventoryEntry>,
    ch: char,
) -> Option<Vec<InventoryEvent>> {
    use ItemCategory::{Armor, Ring, Weapon};
    let index = inventory
        .iter()
        .position(|e| e.ichar == ch && e.equipped_slot.is_none())?;
    let preferred: &[EquipmentSlot] = match inventory[index].item.category {
        Weapon => &[EquipmentSlot::Weapon],
        Armor => &[EquipmentSlot::Armor],
        Ring => &[EquipmentSlot::LeftRing, EquipmentSlot::RightRing],
        _ => return None,
    };
    let slot = preferred
        .iter()
        .copied()
        .find(|s| inventory.iter().all(|e| e.equipped_slot != Some(*s)))
        .unwrap_or(preferred[0]);
    let mut events = Vec::new();
    if let Some(existing) = inventory.iter_mut().find(|e| e.equipped_slot == Some(slot)) {
        existing.equipped_slot = None;
        events.push(InventoryEvent::Unequipped {
            name: existing.item.name,
            slot,
        });
    }
    inventory[index].equipped_slot = Some(slot);
    events.push(InventoryEvent::Equipped {
        name: inventory[index].item.name,
        slot,
    });
    Some(events)
}

/// Unequip (take off) the equipped item identified by `ch`.
pub fn unequip_by_ichar(
    inventory: &mut [InventoryEntry],
    ch: char,
) -> Option<InventoryEvent> {
    let entry = inventory
        .iter_mut()
        .find(|e| e.ichar == ch && e.equipped_slot.is_some())?;
    let slot = entry.equipped_slot.take().unwrap();
    Some(InventoryEvent::Unequipped {
        name: entry.item.name,
        slot,
    })
}

/// Drop the item identified by `ch` at `position`.
///
/// Mirrors the original drop() in pack.c:
/// - Non-weapon stacks with quantity > 1: drop one, keep the rest in the pack.
/// - Everything else (weapons or single items): remove the entire entry from
///   the pack and place it on the floor with its full quantity / quiver intact.
pub fn drop_by_ichar(
    inventory: &mut Vec<InventoryEntry>,
    floor_items: &mut Vec<FloorItem>,
    ch: char,
    position: Position,
) -> Option<Vec<InventoryEvent>> {
    if floor_items.iter().any(|fi| fi.position == position) {
        return None;
    }
    let index = inventory.iter().position(|e| e.ichar == ch)?;
    let mut events = Vec::new();

    // Non-weapon stacks: drop 1, keep the rest.
    if inventory[index].quantity > 1
        && inventory[index].item.category != ItemCategory::Weapon
    {
        let entry = &mut inventory[index];
        entry.quantity -= 1;
        let item_name = entry.item.name;
        let item_clone = entry.item.clone();
        floor_items.push(FloorItem {
            item: item_clone,
            position,
            quantity: 1,
            quiver: None,
        });
        events.push(InventoryEvent::Dropped { name: item_name, position });
        return Some(events);
    }

    // Single item or weapon stack: remove from pack entirely.
    let mut entry = inventory.remove(index);
    if let Some(slot) = entry.equipped_slot.take() {
        events.push(InventoryEvent::Unequipped {
            name: entry.item.name,
            slot,
        });
    }
    floor_items.push(FloorItem {
        item: entry.item.clone(),
        position,
        quantity: entry.quantity,
        quiver: entry.quiver,
    });
    events.push(InventoryEvent::Dropped {
        name: entry.item.name,
        position,
    });
    Some(events)
}

pub fn pick_up_item(
    inventory: &mut Vec<InventoryEntry>,
    floor_items: &mut Vec<FloorItem>,
    next_item_id: &mut u64,
    position: Position,
) -> Option<InventoryEvent> {
    let idx = floor_items
        .iter()
        .position(|fi| fi.position == position)?;
    let floor_item = floor_items.remove(idx);
    let item_name = floor_item.item.name;

    // Try to condense into an existing pack stack (check_duplicate logic).
    // Capacity is NOT checked for merges — matching the original pack_count()
    // behaviour which excludes matching slots from the limit.
    if let Some(existing) = find_stackable_entry(inventory, &floor_item) {
        existing.quantity += floor_item.quantity;
        return Some(InventoryEvent::PickedUp { name: item_name });
    }

    // No merge found — needs a new slot; check capacity.
    if inventory.len() >= MAX_PACK_ITEMS {
        // Put the item back on the floor before returning.
        floor_items.insert(idx.min(floor_items.len()), floor_item);
        return Some(InventoryEvent::PackFull);
    }

    let ichar = next_avail_ichar(inventory);
    inventory.push(InventoryEntry {
        id: *next_item_id,
        item: floor_item.item,
        equipped_slot: None,
        ichar,
        quantity: floor_item.quantity,
        quiver: floor_item.quiver,
    });
    *next_item_id += 1;

    Some(InventoryEvent::PickedUp { name: item_name })
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

#[cfg(test)]
mod tests {
    use super::{
        drop_by_ichar, equip_by_ichar, pick_up_item, total_armor_bonus, total_attack_bonus,
        unequip_by_ichar, EquipmentSlot, FloorItem, InventoryEntry, InventoryEvent, InventoryItem,
    };
    use crate::core_types::Position;

    #[test]
    fn picking_up_item_moves_it_into_inventory() {
        let mut inventory = Vec::new();
        let mut floor_items = vec![FloorItem {
            item: InventoryItem::dagger(),
            position: Position::new(10, 10),
            quantity: 1,
            quiver: None,
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
                ichar: 'a',
                quantity: 1,
                quiver: None,
            },
            InventoryEntry {
                id: 2,
                item: InventoryItem::leather_armor(),
                equipped_slot: None,
                ichar: 'b',
                quantity: 1,
                quiver: None,
            },
            InventoryEntry {
                id: 3,
                item: InventoryItem::accuracy_ring(),
                equipped_slot: None,
                ichar: 'c',
                quantity: 1,
                quiver: None,
            },
        ];

        assert!(equip_by_ichar(&mut inventory, 'a').is_some());
        assert!(equip_by_ichar(&mut inventory, 'b').is_some());
        assert!(equip_by_ichar(&mut inventory, 'c').is_some());

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
            ichar: 'a',
            quantity: 1,
            quiver: None,
        }];
        let mut floor_items = Vec::new();

        let events = drop_by_ichar(&mut inventory, &mut floor_items, 'a', Position::new(5, 5));

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
    fn unequip_by_ichar_removes_ring() {
        let mut inventory = vec![
            InventoryEntry {
                id: 1,
                item: InventoryItem::protection_ring(),
                equipped_slot: Some(EquipmentSlot::LeftRing),
                ichar: 'a',
                quantity: 1,
                quiver: None,
            },
            InventoryEntry {
                id: 2,
                item: InventoryItem::accuracy_ring(),
                equipped_slot: Some(EquipmentSlot::RightRing),
                ichar: 'b',
                quantity: 1,
                quiver: None,
            },
        ];

        let event = unequip_by_ichar(&mut inventory, 'b');

        assert_eq!(
            event,
            Some(InventoryEvent::Unequipped {
                name: "ring of accuracy",
                slot: EquipmentSlot::RightRing,
            })
        );
    }
}

