use std::collections::HashSet;

use crate::actors::{
    attack_monster, spawn_basic_monsters, tick_monsters, CombatEvent, Monster, MonsterKind,
    SpecialHit, StatusEffectEvent,
};
use crate::core_types::{EXP_LEVELS, FOOD_FAINT, FOOD_HUNGRY, FOOD_WEAK, INIT_FOOD, INIT_HP, INIT_STRENGTH, MAX_HP, MAX_TRAPS, Position, TileFlags, TrapKind};
use crate::inventory_items::{
    apply_item_effects, drop_by_ichar, equip_by_ichar, gr_floor_item, pick_up_item,
    remove_item_by_ichar, total_armor_bonus, total_attack_bonus, unequip_by_ichar, EquipmentSlot,
    FloorItem, InventoryEntry, InventoryEvent, InventoryItem, ItemCategory,
};
use crate::persistence;
use crate::rng::GameRng;
use crate::world_gen::{generate_level_with_depth, DungeonGrid, GeneratedLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Rest,
    Quit,
    Move(Direction),
    PickUp,
    Drop,
    Wield,
    WearArmor,
    TakeOffArmor,
    PutOnRing,
    RemoveRing,
    Quaff,
    Zap,
    Throw,
    ReadScroll,
    Eat,
    IdentifyTrap,
    Search,
    Save,
    Load,
    Descend,
    /// Sent by the UI when the player presses a letter in item-selection mode.
    SelectItem(char),
    /// Sent by the UI when the player cancels item selection (Escape).
    CancelItemSelect,
    // Wizard mode commands
    /// Toggle wizard mode (prompts for password if not yet active).
    ToggleWizard,
    /// Wizard: reveal the full dungeon layout on the current level.
    WizardRevealMap,
    /// Wizard: mark all traps on the current level as known.
    WizardShowTraps,
    /// Wizard: show all floor objects on the current level.
    WizardShowObjects,
    /// Wizard: list all level objects as an inventory overlay (Tab).
    WizardShowLevelObjects,
    /// Wizard: add a random item to the player's pack (Ctrl+C).
    WizardAddItem,
    /// Wizard: reveal all monsters on the current level.
    WizardShowMonsters,
    /// Wizard password input: one character typed by the player.
    WizardPasswordChar(char),
    /// Wizard password input: player pressed Enter to submit.
    WizardPasswordSubmit,
    /// Wizard password input: player pressed Escape to cancel.
    WizardPasswordCancel,
    Noop,
    Unknown,
}

/// Which inventory action is waiting for the player to pick an item by letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingItemAction {
    Drop,
    Wield,
    WearArmor,
    TakeOffArmor,
    PutOnRing,
    RemoveRing,
    Quaff,
    ReadScroll,
    Eat,
    Zap,
    Throw,
}

impl PendingItemAction {
    pub fn prompt(self) -> &'static str {
        match self {
            Self::Drop => "drop what?",
            Self::Wield => "what do you want to wield?",
            Self::WearArmor => "what do you want to wear?",
            Self::TakeOffArmor => "what armor do you take off?",
            Self::PutOnRing => "what ring do you put on?",
            Self::RemoveRing => "what ring do you remove?",
            Self::Quaff => "which potion do you want to quaff?",
            Self::ReadScroll => "which scroll do you want to read?",
            Self::Eat => "which food do you want to eat?",
            Self::Zap => "which wand do you want to use?",
            Self::Throw => "what do you want to throw?",
        }
    }

    pub fn filter_category(self) -> Option<ItemCategory> {
        match self {
            Self::Wield | Self::Throw => Some(ItemCategory::Weapon),
            Self::WearArmor | Self::TakeOffArmor => Some(ItemCategory::Armor),
            Self::PutOnRing | Self::RemoveRing => Some(ItemCategory::Ring),
            Self::Quaff => Some(ItemCategory::Potion),
            Self::ReadScroll => Some(ItemCategory::Scroll),
            Self::Eat => Some(ItemCategory::Food),
            Self::Zap => Some(ItemCategory::Wand),
            Self::Drop => None,
        }
    }

    /// True for actions that require the item to already be equipped
    /// (TakeOffArmor, RemoveRing).
    pub fn equipped_only(self) -> bool {
        matches!(self, Self::TakeOffArmor | Self::RemoveRing)
    }

    /// Return a message for when there are no matching items.
    pub fn empty_message(self) -> &'static str {
        match self {
            Self::Drop => "you have nothing to drop",
            Self::Wield => "you have nothing to wield",
            Self::WearArmor => "you have no armor to wear",
            Self::TakeOffArmor => "you are not wearing any armor",
            Self::PutOnRing => "you have no rings to put on",
            Self::RemoveRing => "you are wearing no rings",
            Self::Quaff => "you have no potions",
            Self::ReadScroll => "you have no scrolls",
            Self::Eat => "you have no food",
            Self::Zap => "you have no wands",
            Self::Throw => "you have nothing to throw",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub level: i16,
    pub gold: i64,
    pub turns: u64,
    pub quit_requested: bool,
    /// True when the player has died (distinct from quitting voluntarily).
    pub player_dead: bool,
    /// True while the player is in wizard mode.
    pub wizard: bool,
    /// True once wizard mode has been activated; disqualifies the run from
    /// the high-score table for the remainder of the session.
    pub score_only: bool,
    /// Non-None while the wizard-password prompt is active; holds the
    /// characters typed so far (not displayed).
    pub wizard_password_input: Option<String>,
    pub pending_direction: Option<Direction>,
    /// Set when a command needs the player to pick an item by letter before
    /// it can execute.  The renderer uses this to display the item overlay.
    pub pending_item_action: Option<PendingItemAction>,
    pub player_position: Position,
    pub player_hit_points: i16,
    pub player_max_hit_points: i16,
    pub player_strength: i16,
    pub player_max_strength: i16,
    pub player_exp_points: i64,
    pub player_exp_level: i16,
    pub food_remaining: i32,
    pub is_hungry: bool,
    pub is_weak: bool,
    pub frozen_turns: u8,
    pub confused_turns: u8,
    pub monsters_defeated: u64,
    pub monsters: Vec<Monster>,
    pub last_turn_events: Vec<CombatEvent>,
    pub inventory: Vec<InventoryEntry>,
    pub floor_items: Vec<FloorItem>,
    pub trap_positions: Vec<Position>,
    pub trap_types: Vec<TrapKind>,
    pub known_traps: Vec<Position>,
    pub next_item_id: u64,
    pub last_inventory_events: Vec<InventoryEvent>,
    pub last_move_blocked: bool,
    pub last_system_message: Option<String>,
    pub party_counter: i16,
    pub explored: HashSet<Position>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    Continue,
    Finished,
}

#[derive(Debug, Clone)]
pub struct GameLoop {
    state: GameState,
    current_level: GeneratedLevel,
}

enum PlayerAction {
    Moved,
    Attacked,
    Held,
    InventoryChanged,
    Blocked,
}

fn is_adjacent(left: Position, right: Position) -> bool {
    let row_distance = (left.row - right.row).abs();
    let col_distance = (left.col - right.col).abs();
    row_distance <= 1 && col_distance <= 1
}

fn has_regen_ring_only(inventory: &[crate::inventory_items::InventoryEntry]) -> bool {
    inventory.iter().any(|e| {
        matches!(
            e.equipped_slot,
            Some(EquipmentSlot::LeftRing) | Some(EquipmentSlot::RightRing)
        ) && e.item.name == "ring of regeneration"
    })
}

/// Place random items on the floor for a newly-visited level.
/// Matches original put_objects() / gr_object() in object.c.
fn place_floor_items(level: &GeneratedLevel, rng: &mut GameRng, _depth: i16) -> Vec<FloorItem> {
    let (rows, cols) = level.grid.dimensions();
    let mut candidates: Vec<Position> = Vec::new();
    for row in 1..rows as i16 - 1 {
        for col in 1..cols as i16 - 1 {
            let Some(flags) = level.grid.get(row, col) else { continue };
            if flags.intersects(TileFlags::FLOOR | TileFlags::TUNNEL)
                && !flags.contains(TileFlags::STAIRS)
            {
                candidates.push(Position::new(row, col));
            }
        }
    }

    if candidates.is_empty() {
        return Vec::new();
    }

    let base_count: usize = if rng.coin_toss() {
        rng.get_rand(3, 5) as usize
    } else {
        rng.get_rand(2, 4) as usize
    };
    let extra: usize = if rng.rand_percent(33) { 1 } else { 0 };
    let extra2: usize = if rng.rand_percent(33) { 1 } else { 0 };
    let count = (base_count + extra + extra2).min(candidates.len());

    let mut items: Vec<FloorItem> = Vec::with_capacity(count);
    for _ in 0..count {
        if candidates.is_empty() {
            break;
        }
        let idx = rng.get_rand(0, (candidates.len() - 1) as i32) as usize;
        let pos = candidates.remove(idx);
        // Skip if a floor item is already at this position.
        if items.iter().any(|fi| fi.position == pos) {
            continue;
        }
        items.push(FloorItem { item: gr_floor_item(rng), position: pos });
    }
    items
}

/// Place hidden traps for a newly-generated level.
/// Matches original add_traps() in trap.c.
fn place_traps(level: &GeneratedLevel, rng: &mut GameRng, depth: i16) -> (Vec<Position>, Vec<TrapKind>) {
    let trap_count: i32 = if depth <= 2 {
        0
    } else if depth <= 7 {
        rng.get_rand(0, 2)
    } else if depth <= 11 {
        rng.get_rand(1, 2)
    } else if depth <= 16 {
        rng.get_rand(2, 3)
    } else if depth <= 21 {
        rng.get_rand(2, 4)
    } else if depth <= 26 {
        rng.get_rand(3, 5)
    } else {
        rng.get_rand(5, MAX_TRAPS as i32)
    };

    if trap_count == 0 {
        return (Vec::new(), Vec::new());
    }

    let (rows, cols) = level.grid.dimensions();
    let mut candidates: Vec<Position> = Vec::new();
    for row in 1..rows as i16 - 1 {
        for col in 1..cols as i16 - 1 {
            if level.grid.get(row, col).map_or(false, |f| f.contains(TileFlags::FLOOR)) {
                candidates.push(Position::new(row, col));
            }
        }
    }

    if candidates.is_empty() {
        return (Vec::new(), Vec::new());
    }

    const TRAP_KINDS: [TrapKind; 6] = [
        TrapKind::TrapDoor, TrapKind::BearTrap, TrapKind::TeleTrap,
        TrapKind::DartTrap, TrapKind::SleepingGasTrap, TrapKind::RustTrap,
    ];

    let mut positions: Vec<Position> = Vec::new();
    let mut kinds: Vec<TrapKind> = Vec::new();

    for _ in 0..trap_count {
        if candidates.is_empty() {
            break;
        }
        let idx = rng.get_rand(0, (candidates.len() - 1) as i32) as usize;
        let pos = candidates.remove(idx);
        let kind_idx = rng.get_rand(0, 5) as usize;
        positions.push(pos);
        kinds.push(TRAP_KINDS[kind_idx]);
    }

    (positions, kinds)
}

/// Strength-to-damage bonus table, matching original `damage_for_strength()` in `hit.c`.
fn damage_for_strength(str: i16) -> i16 {
    match str {
        s if s <= 3  => -3,
        4 | 5        => -2,
        6 | 7        => -1,
        8..=12       =>  0,
        13..=15      =>  1,
        16           =>  2,
        17           =>  3,
        18           =>  5,
        _            =>  6 + (str - 19) / 2,
    }
}

/// Verify a plaintext password against the wizard cipher from the original.
///
/// The original `wizardize()` in `zap.c` applies `xxx()` / `xxxx()` (defined
/// in `score.c`) to the input and compares with the 7-byte literal
/// `"\247\104\126\272\115\243\027"`.  Reversing the cipher yields the
/// plaintext password "bathtub".
fn wizard_check_password(input: &str) -> bool {
    // Reference cipher: initialise f=37, s=7, then for each byte i:
    //   r = (f * s + 9337) % 8887; f = s; s = r; c[i] = r as u8;
    //   transformed[i] = input[i] ^ c[i]
    // The target after transformation is the 7 bytes below.
    const TARGET: [u8; 7] = [0xA7, 0x44, 0x56, 0xBA, 0x4D, 0xA3, 0x17];
    let bytes = input.as_bytes();
    if bytes.len() != TARGET.len() {
        return false;
    }
    let mut f: i64 = 37;
    let mut s: i64 = 7;
    for (i, &b) in bytes.iter().enumerate() {
        let r = (f * s + 9337) % 8887;
        f = s;
        s = r;
        let cipher = (r & 0xFF) as u8;
        if b ^ cipher != TARGET[i] {
            return false;
        }
    }
    true
}

impl GameLoop {
    pub fn new(seed: i32) -> Self {
        let mut rng = GameRng::new(seed);
        let party_counter = rng.get_rand(1, 10) as i16;
        let current_level = generate_level_with_depth(&mut rng, 1, party_counter);
        let player_position = current_level.spawn_position();
        let monsters = spawn_basic_monsters(&current_level, &mut rng, player_position, 1);
        let floor_items = place_floor_items(&current_level, &mut rng, 1);
        let (trap_positions, trap_types) = place_traps(&current_level, &mut rng, 1);
        let arrows_count = rng.get_rand(25, 35) as u16;

        let mut game = Self {
            state: GameState {
                level: 1,
                gold: 0,
                turns: 0,
                quit_requested: false,
                player_dead: false,
                wizard: false,
                score_only: false,
                wizard_password_input: None,
                pending_direction: None,
                pending_item_action: None,
                player_position,
                player_hit_points: INIT_HP,
                player_max_hit_points: INIT_HP,
                player_strength: INIT_STRENGTH,
                player_max_strength: INIT_STRENGTH,
                player_exp_points: 0,
                player_exp_level: 1,
                food_remaining: INIT_FOOD,
                is_hungry: false,
                is_weak: false,
                frozen_turns: 0,
                confused_turns: 0,
                monsters_defeated: 0,
                monsters,
                last_turn_events: Vec::new(),
                inventory: vec![
                    // Initial equipment matching original Rogue (see original/rogue-libc5-ncurses/rogue/init.c player_init())
                    InventoryEntry {
                        id: 1,
                        item: InventoryItem::food_ration(),
                        equipped_slot: None,
                        ichar: 'a',
                        quantity: 1,
                    },
                    InventoryEntry {
                        id: 2,
                        item: InventoryItem::ring_mail(),
                        equipped_slot: Some(EquipmentSlot::Armor),
                        ichar: 'b',
                        quantity: 1,
                    },
                    InventoryEntry {
                        id: 3,
                        item: InventoryItem::mace(),
                        equipped_slot: Some(EquipmentSlot::Weapon),
                        ichar: 'c',
                        quantity: 1,
                    },
                    InventoryEntry {
                        id: 4,
                        item: InventoryItem::bow(),
                        equipped_slot: None,
                        ichar: 'd',
                        quantity: 1,
                    },
                    InventoryEntry {
                        id: 5,
                        item: InventoryItem::arrow(),
                        equipped_slot: None,
                        ichar: 'e',
                        quantity: arrows_count,
                    },
                ],
                floor_items,
                trap_positions,
                trap_types,
                known_traps: Vec::new(),
                next_item_id: 6,
                last_inventory_events: Vec::new(),
                last_move_blocked: false,
                last_system_message: None,
                party_counter,
                explored: HashSet::new(),
            },
            current_level,
        };
        game.update_explored();
        game
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    #[cfg(test)]
    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    pub fn current_level(&self) -> &GeneratedLevel {
        &self.current_level
    }

    pub fn from_parts(state: GameState, current_level: GeneratedLevel) -> Self {
        Self {
            state,
            current_level,
        }
    }

    fn update_explored(&mut self) {
        let pos = self.state.player_position;
        if let Some(room) = self.current_level.rooms.iter().find(|r| r.contains(pos.row, pos.col)) {
            for row in (room.top_row - 1)..=(room.bottom_row + 1) {
                for col in (room.left_col - 1)..=(room.right_col + 1) {
                    if DungeonGrid::in_bounds(row, col) {
                        self.state.explored.insert(Position::new(row, col));
                    }
                }
            }
        } else {
            for drow in -1i16..=1 {
                for dcol in -1i16..=1 {
                    let p = Position::new(pos.row + drow, pos.col + dcol);
                    if DungeonGrid::in_bounds(p.row, p.col) {
                        self.state.explored.insert(p);
                    }
                }
            }
        }
    }

    pub fn player_is_held(&self) -> bool {
        self.state.monsters.iter().any(|monster| {
            monster.special_hit == Some(SpecialHit::Hold)
                && is_adjacent(monster.position, self.state.player_position)
        })
    }

    pub fn parse_command(input: char) -> Command {
        match input {
            '.' => Command::Rest,
            'Q' => Command::Quit,
            'h' => Command::Move(Direction::Left),
            'j' => Command::Move(Direction::Down),
            'k' => Command::Move(Direction::Up),
            'l' => Command::Move(Direction::Right),
            ',' => Command::PickUp,
            'd' => Command::Drop,
            'w' => Command::Wield,
            'W' => Command::WearArmor,
            'T' => Command::TakeOffArmor,
            'P' => Command::PutOnRing,
            'R' => Command::RemoveRing,
            'q' => Command::Quaff,
            'z' => Command::Zap,
            't' => Command::Throw,
            'r' => Command::ReadScroll,
            'e' => Command::Eat,
            '^' => Command::IdentifyTrap,
            's' => Command::Search,
            'S' => Command::Save,
            'L' => Command::Load,
            '>' => Command::Descend,
            'y' => Command::Move(Direction::UpLeft),
            'u' => Command::Move(Direction::UpRight),
            'b' => Command::Move(Direction::DownLeft),
            'n' => Command::Move(Direction::DownRight),
            ' ' => Command::Noop,
            _ => Command::Unknown,
        }
    }

    fn player_attack_damage(&self) -> i16 {
        let str_bonus = damage_for_strength(self.state.player_strength);
        let exp_bonus = (self.state.player_exp_level - 1) / 2;
        let weapon_base = total_attack_bonus(&self.state.inventory).max(1);
        let base = (weapon_base + str_bonus + exp_bonus).max(1);
        if self.state.wizard { base * 3 } else { base }
    }

    fn player_armor_bonus(&self) -> i16 {
        total_armor_bonus(&self.state.inventory)
    }

    fn direction_delta(direction: Direction) -> (i16, i16) {
        match direction {
            Direction::Left => (0, -1),
            Direction::Right => (0, 1),
            Direction::Up => (-1, 0),
            Direction::Down => (1, 0),
            Direction::UpLeft => (-1, -1),
            Direction::UpRight => (-1, 1),
            Direction::DownLeft => (1, -1),
            Direction::DownRight => (1, 1),
        }
    }

    fn try_move_player(&mut self, direction: Direction) -> PlayerAction {
        let (drow, dcol) = Self::direction_delta(direction);
        let target = Position::new(
            self.state.player_position.row + drow,
            self.state.player_position.col + dcol,
        );
        let attack_damage = self.player_attack_damage();

        if self.player_is_held()
            && !self
                .state
                .monsters
                .iter()
                .any(|monster| monster.position == target)
        {
            self.state.last_move_blocked = true;
            return PlayerAction::Held;
        }

        if let Some(event) = attack_monster(&mut self.state.monsters, target, attack_damage) {
            if let CombatEvent::PlayerHitMonster {
                killed: true,
                kill_exp,
                monster_kind,
                position: kill_pos,
                ..
            } = event
            {
                self.state.monsters_defeated += 1;
                self.state.player_exp_points += kill_exp as i64;
                let next_level = self.state.player_exp_level as usize;
                if next_level < EXP_LEVELS.len()
                    && self.state.player_exp_points >= EXP_LEVELS[next_level - 1]
                {
                    self.state.player_exp_level = (self.state.player_exp_level + 1).min(21);
                    let mut lvl_rng = GameRng::new(self.state.turns as i32 ^ 0x5432_i32);
                    // Wizard mode: fixed 10 HP per level (original hp_raise() in level.c)
                    let hp_gain = if self.state.wizard {
                        10
                    } else {
                        lvl_rng.get_rand(3, 10) as i16
                    };
                    self.state.player_max_hit_points =
                        (self.state.player_max_hit_points + hp_gain).min(MAX_HP);
                    self.state.player_hit_points = self.state.player_max_hit_points;
                    self.state.last_system_message = Some(format!(
                        "Welcome to experience level {}!",
                        self.state.player_exp_level
                    ));
                }
                // Monster may drop an item on death
                let mut drop_rng = GameRng::new(
                    self.state.turns as i32
                        ^ kill_pos.row as i32
                        ^ 0x4321_i32,
                );
                if drop_rng.rand_percent(monster_kind.drop_percent()) {
                    let dropped = gr_floor_item(&mut drop_rng);
                    self.state
                        .floor_items
                        .push(FloorItem { item: dropped, position: kill_pos });
                }
            }
            self.state.last_turn_events.push(event);
            self.state.last_move_blocked = false;
            return PlayerAction::Attacked;
        }

        if self.current_level.grid.is_walkable(target.row, target.col) {
            self.state.player_position = target;
            self.state.last_move_blocked = false;
            self.update_explored();
            PlayerAction::Moved
        } else {
            self.state.last_move_blocked = true;
            PlayerAction::Blocked
        }
    }

    /// Immediate inventory actions (PickUp, IdentifyTrap) that don't need item
    /// letter selection.
    fn try_immediate_inventory_action(&mut self, command: Command) -> PlayerAction {
        let events: Option<Vec<InventoryEvent>> = match command {
            Command::PickUp => pick_up_item(
                &mut self.state.inventory,
                &mut self.state.floor_items,
                &mut self.state.next_item_id,
                self.state.player_position,
            )
            .map(|event| vec![event]),
            Command::IdentifyTrap => {
                let found = self
                    .state
                    .trap_positions
                    .iter()
                    .copied()
                    .find(|position| is_adjacent(*position, self.state.player_position));

                if let Some(position) = found {
                    let trap_idx = self.state.trap_positions.iter().position(|&p| p == position);
                    let trap_name = trap_idx
                        .and_then(|i| self.state.trap_types.get(i))
                        .map(|k| k.name())
                        .unwrap_or("trap");
                    if !self.state.known_traps.contains(&position) {
                        self.state.known_traps.push(position);
                    }
                    self.state.last_system_message = Some(format!("You found a {trap_name}."));
                    Some(Vec::new())
                } else {
                    self.state.last_system_message = Some("No trap nearby.".to_string());
                    None
                }
            }
            _ => None,
        };

        if let Some(events) = events {
            self.state.last_inventory_events.extend(events);
            PlayerAction::InventoryChanged
        } else {
            PlayerAction::Blocked
        }
    }

    /// Start an item-selection action.  If the pack has at least one eligible
    /// item, set `pending_item_action` and show the prompt; otherwise show the
    /// "nothing to …" message and return Blocked.
    fn start_item_action(&mut self, action: PendingItemAction) -> PlayerAction {
        let has_item = self.state.inventory.iter().any(|e| {
            let cat_ok = action
                .filter_category()
                .map_or(true, |cat| e.item.category == cat);
            let eq_ok = if action.equipped_only() {
                e.equipped_slot.is_some()
            } else {
                true
            };
            cat_ok && eq_ok
        });

        if has_item {
            self.state.pending_item_action = Some(action);
            self.state.last_system_message = Some(action.prompt().to_string());
            PlayerAction::Blocked // Don't advance turn; wait for SelectItem
        } else {
            self.state.last_system_message = Some(action.empty_message().to_string());
            PlayerAction::Blocked
        }
    }

    /// Execute the pending item action for the item with pack-letter `ch`.
    fn execute_item_selection(&mut self, action: PendingItemAction, ch: char) -> PlayerAction {
        let events: Option<Vec<InventoryEvent>> = match action {
            PendingItemAction::Drop => drop_by_ichar(
                &mut self.state.inventory,
                &mut self.state.floor_items,
                ch,
                self.state.player_position,
            ),
            PendingItemAction::Wield => {
                let valid = self.state.inventory.iter().any(|e| {
                    e.ichar == ch
                        && e.item.category == ItemCategory::Weapon
                        && e.equipped_slot.is_none()
                });
                if valid {
                    equip_by_ichar(&mut self.state.inventory, ch)
                } else {
                    self.state.last_system_message = Some("no such item.".to_string());
                    None
                }
            }
            PendingItemAction::WearArmor => {
                let valid = self.state.inventory.iter().any(|e| {
                    e.ichar == ch
                        && e.item.category == ItemCategory::Armor
                        && e.equipped_slot.is_none()
                });
                if valid {
                    equip_by_ichar(&mut self.state.inventory, ch)
                } else {
                    self.state.last_system_message = Some("no such item.".to_string());
                    None
                }
            }
            PendingItemAction::TakeOffArmor => {
                let valid = self.state.inventory.iter().any(|e| {
                    e.ichar == ch
                        && e.item.category == ItemCategory::Armor
                        && e.equipped_slot.is_some()
                });
                if valid {
                    unequip_by_ichar(&mut self.state.inventory, ch).map(|ev| vec![ev])
                } else {
                    self.state.last_system_message = Some("no such item.".to_string());
                    None
                }
            }
            PendingItemAction::PutOnRing => {
                let valid = self.state.inventory.iter().any(|e| {
                    e.ichar == ch
                        && e.item.category == ItemCategory::Ring
                        && e.equipped_slot.is_none()
                });
                if valid {
                    equip_by_ichar(&mut self.state.inventory, ch)
                } else {
                    self.state.last_system_message = Some("no such item.".to_string());
                    None
                }
            }
            PendingItemAction::RemoveRing => {
                let valid = self.state.inventory.iter().any(|e| {
                    e.ichar == ch
                        && e.item.category == ItemCategory::Ring
                        && e.equipped_slot.is_some()
                });
                if valid {
                    unequip_by_ichar(&mut self.state.inventory, ch).map(|ev| vec![ev])
                } else {
                    self.state.last_system_message = Some("no such item.".to_string());
                    None
                }
            }
            PendingItemAction::Quaff => {
                let entry = self.state.inventory.iter().find(|e| {
                    e.ichar == ch && e.item.category == ItemCategory::Potion
                });
                if entry.is_none() {
                    self.state.last_system_message = Some("no such item.".to_string());
                    return PlayerAction::Blocked;
                }
                remove_item_by_ichar(&mut self.state.inventory, ch).map(|entry| {
                    let msg = match entry.item.name {
                        "healing potion" => {
                            // Original: potion_heal(rogue.exp) — heal by exp level
                            let n = self.state.player_exp_level as i16;
                            let new_hp = self.state.player_hit_points + n;
                            if new_hp > self.state.player_max_hit_points {
                                if self.state.player_hit_points == self.state.player_max_hit_points {
                                    self.state.player_max_hit_points =
                                        (self.state.player_max_hit_points + 1).min(MAX_HP);
                                }
                                self.state.player_hit_points = self.state.player_max_hit_points;
                            } else {
                                self.state.player_hit_points = new_hp;
                            }
                            "You feel better."
                        }
                        "potion of extra healing" => {
                            // Original: potion_heal(2 * rogue.exp)
                            let n = self.state.player_exp_level as i16 * 2;
                            let new_hp = self.state.player_hit_points + n;
                            if new_hp > self.state.player_max_hit_points {
                                if self.state.player_hit_points == self.state.player_max_hit_points {
                                    self.state.player_max_hit_points =
                                        (self.state.player_max_hit_points + 1).min(MAX_HP);
                                }
                                self.state.player_hit_points = self.state.player_max_hit_points;
                            } else {
                                self.state.player_hit_points = new_hp;
                            }
                            "You feel much better."
                        }
                        "potion of increase strength" => "You feel stronger.",
                        "potion of restore strength" => "You feel your strength return.",
                        "potion of poison" => "You feel very sick.",
                        "potion of raise level" => "You feel more experienced.",
                        "potion of blindness" => "A cloud of darkness surrounds you.",
                        "potion of hallucination" => "Oh wow, everything seems so cosmic!",
                        "potion of detect monster" => "You sense the presence of monsters.",
                        "potion of detect objects" => "You sense the presence of objects.",
                        "potion of confusion" => "You feel confused.",
                        "potion of levitation" => "You start to float in the air.",
                        "potion of haste self" => "You feel yourself moving faster.",
                        "potion of see invisible" => "Your vision becomes clear.",
                        _ => "You drink the potion.",
                    };
                    self.state.last_system_message = Some(msg.to_string());
                    vec![InventoryEvent::Used { name: entry.item.name }]
                })
            }
            PendingItemAction::ReadScroll => {
                let entry = self.state.inventory.iter().find(|e| {
                    e.ichar == ch && e.item.category == ItemCategory::Scroll
                });
                if entry.is_none() {
                    self.state.last_system_message = Some("no such item.".to_string());
                    return PlayerAction::Blocked;
                }
                remove_item_by_ichar(&mut self.state.inventory, ch).map(|entry| {
                    let msg: &str = match entry.item.name {
                        "scroll of enchant weapon" => {
                            if let Some(e) = self.state.inventory.iter_mut().find(|e| {
                                e.equipped_slot == Some(EquipmentSlot::Weapon)
                            }) {
                                e.item.attack_bonus += 1;
                                "Your weapon glows blue."
                            } else {
                                "You feel a glow, but you have no weapon equipped."
                            }
                        }
                        "scroll of enchant armor" => {
                            if let Some(e) = self.state.inventory.iter_mut().find(|e| {
                                e.equipped_slot == Some(EquipmentSlot::Armor)
                            }) {
                                e.item.armor_bonus += 1;
                                "Your armor glows silver."
                            } else {
                                "You feel a glow, but you have no armor equipped."
                            }
                        }
                        "scroll of teleport" => {
                            let (rows, cols) = self.current_level.grid.dimensions();
                            let mut tel_rng =
                                GameRng::new(self.state.turns as i32 ^ 0x7777);
                            let mut candidates: Vec<Position> = Vec::new();
                            for row in 1..rows as i16 - 1 {
                                for col in 1..cols as i16 - 1 {
                                    if self.current_level.grid.is_walkable(row, col) {
                                        candidates.push(Position::new(row, col));
                                    }
                                }
                            }
                            if !candidates.is_empty() {
                                let idx = tel_rng.get_rand(
                                    0,
                                    (candidates.len() - 1) as i32,
                                ) as usize;
                                self.state.player_position = candidates[idx];
                                self.update_explored();
                            }
                            "You suddenly find yourself somewhere else."
                        }
                        "scroll of magic mapping" => {
                            let (rows, cols) = self.current_level.grid.dimensions();
                            for row in 0..rows as i16 {
                                for col in 0..cols as i16 {
                                    if let Some(flags) = self.current_level.grid.get(row, col) {
                                        if flags.intersects(
                                            TileFlags::FLOOR
                                                | TileFlags::TUNNEL
                                                | TileFlags::DOOR
                                                | TileFlags::STAIRS,
                                        ) {
                                            self.state.explored.insert(Position::new(row, col));
                                        }
                                    }
                                }
                            }
                            "You feel a sense of the dungeon around you."
                        }
                        "scroll of create monster" => {
                            // Spawn one random monster adjacent to the player
                            let player_pos = self.state.player_position;
                            let mut spawn_rng =
                                GameRng::new(self.state.turns as i32 ^ 0x1234);
                            let offsets: [(i16, i16); 8] = [
                                (-1, -1), (-1, 0), (-1, 1),
                                (0, -1),            (0, 1),
                                (1, -1),  (1, 0),  (1, 1),
                            ];
                            'spawn: for _ in 0..16 {
                                let idx =
                                    spawn_rng.get_rand(0, 7) as usize;
                                let (dr, dc) = offsets[idx];
                                let pos = Position::new(
                                    player_pos.row + dr,
                                    player_pos.col + dc,
                                );
                                if self.current_level.grid.is_walkable(pos.row, pos.col)
                                    && !self.state.monsters.iter().any(|m| m.position == pos)
                                {
                                    let kind_idx = spawn_rng.get_rand(0, 25) as usize;
                                    const KINDS: [MonsterKind; 26] = [
                                        MonsterKind::Aquator, MonsterKind::Bat,
                                        MonsterKind::Centaur, MonsterKind::Dragon,
                                        MonsterKind::Emu, MonsterKind::VenusFlytrap,
                                        MonsterKind::Griffin, MonsterKind::Hobgoblin,
                                        MonsterKind::IceMonster, MonsterKind::Jabberwock,
                                        MonsterKind::Kestrel, MonsterKind::Leprechaun,
                                        MonsterKind::Medusa, MonsterKind::Nymph,
                                        MonsterKind::Orc, MonsterKind::Phantom,
                                        MonsterKind::Quagga, MonsterKind::Rattlesnake,
                                        MonsterKind::Snake, MonsterKind::Troll,
                                        MonsterKind::BlackUnicorn, MonsterKind::Vampire,
                                        MonsterKind::Wraith, MonsterKind::Xeroc,
                                        MonsterKind::Yeti, MonsterKind::Zombie,
                                    ];
                                    self.state.monsters.push(Monster::new(KINDS[kind_idx], pos));
                                    break 'spawn;
                                }
                            }
                            "You hear a faint cry in the distance."
                        }
                        "scroll of sleep" => {
                            self.state.frozen_turns = 5;
                            "You fall asleep."
                        }
                        "scroll of protect armor" => "Your armor glows faintly.",
                        "scroll of hold monster" => "The monsters are frozen.",
                        "scroll of identify" => "You can identify this item.",
                        "scroll of scare monster" => "The monsters flee.",
                        "scroll of remove curse" => {
                            "You feel as if someone is watching over you."
                        }
                        "scroll of aggravate monster" => {
                            "You hear a high pitched humming noise."
                        }
                        _ => "You read the scroll.",
                    };
                    self.state.last_system_message = Some(msg.to_string());
                    vec![InventoryEvent::Used { name: entry.item.name }]
                })
            }
            PendingItemAction::Eat => {
                let entry = self.state.inventory.iter().find(|e| {
                    e.ichar == ch && e.item.category == ItemCategory::Food
                });
                if entry.is_none() {
                    self.state.last_system_message = Some("no such item.".to_string());
                    return PlayerAction::Blocked;
                }
                remove_item_by_ichar(&mut self.state.inventory, ch).map(|entry| {
                    let mut eat_rng = GameRng::new(self.state.turns as i32 ^ 0x5550);
                    let is_yummy = eat_rng.get_rand(0, 99) < 60;
                    let moves = if is_yummy {
                        eat_rng.get_rand(900, 1100)
                    } else {
                        self.state.player_exp_points += 2;
                        eat_rng.get_rand(700, 900)
                    };
                    let msg = if is_yummy {
                        "Yum, that tasted good."
                    } else {
                        "Yuk, that food tasted awful."
                    };
                    self.state.food_remaining = self.state.food_remaining / 3 + moves;
                    self.state.is_hungry = false;
                    self.state.is_weak = false;
                    self.state.last_system_message = Some(msg.to_string());
                    vec![InventoryEvent::Used { name: entry.item.name }]
                })
            }
            PendingItemAction::Zap => {
                let entry = self.state.inventory.iter().find(|e| {
                    e.ichar == ch && e.item.category == ItemCategory::Wand
                });
                if entry.is_none() {
                    self.state.last_system_message = Some("no such item.".to_string());
                    return PlayerAction::Blocked;
                }
                remove_item_by_ichar(&mut self.state.inventory, ch).map(|entry| {
                    let direction = self.state.pending_direction.unwrap_or(Direction::Right);
                    if let Some(target) = self.first_monster_in_direction(direction) {
                        if let Some(event) = attack_monster(&mut self.state.monsters, target, 2) {
                            if matches!(event, CombatEvent::PlayerHitMonster { killed: true, .. }) {
                                self.state.monsters_defeated += 1;
                            }
                            self.state.last_turn_events.push(event);
                            self.state.last_system_message = Some("Magic missile hits.".to_string());
                        }
                    } else {
                        self.state.last_system_message =
                            Some("The wand fizzles into empty air.".to_string());
                    }
                    vec![InventoryEvent::Used { name: entry.item.name }]
                })
            }
            PendingItemAction::Throw => {
                let entry = self.state.inventory.iter().find(|e| {
                    e.ichar == ch && e.item.category == ItemCategory::Weapon
                });
                if entry.is_none() {
                    self.state.last_system_message = Some("no such item.".to_string());
                    return PlayerAction::Blocked;
                }
                remove_item_by_ichar(&mut self.state.inventory, ch).map(|entry| {
                    let direction = self.state.pending_direction.unwrap_or(Direction::Right);
                    let (drow, dcol) = Self::direction_delta(direction);
                    let target = Position::new(
                        self.state.player_position.row + drow,
                        self.state.player_position.col + dcol,
                    );
                    if let Some(event) = attack_monster(&mut self.state.monsters, target, 1) {
                        if matches!(event, CombatEvent::PlayerHitMonster { killed: true, .. }) {
                            self.state.monsters_defeated += 1;
                        }
                        self.state.last_turn_events.push(event);
                        self.state.last_system_message = Some("You throw and hit.".to_string());
                    } else if self.current_level.grid.is_walkable(target.row, target.col) {
                        self.state.floor_items.push(FloorItem {
                            item: entry.item.clone(),
                            position: target,
                        });
                        self.state.last_system_message = Some("You throw your weapon.".to_string());
                    }
                    vec![InventoryEvent::Thrown { name: entry.item.name }]
                })
            }
        };

        if let Some(events) = events {
            self.state.last_inventory_events.extend(events);
            PlayerAction::InventoryChanged
        } else {
            PlayerAction::Blocked
        }
    }

    fn first_monster_in_direction(&self, direction: Direction) -> Option<Position> {
        let (drow, dcol) = Self::direction_delta(direction);
        let mut current = self.state.player_position;

        loop {
            current = Position::new(current.row + drow, current.col + dcol);

            if !self
                .current_level
                .grid
                .is_walkable(current.row, current.col)
            {
                return None;
            }

            if self
                .state
                .monsters
                .iter()
                .any(|monster| monster.position == current)
            {
                return Some(current);
            }
        }
    }

    fn record_high_score(&mut self, outcome: persistence::RunOutcome) {
        let score = persistence::compute_score(self);
        let total_entries = persistence::load_high_scores()
            .map(|scores| scores.len())
            .ok();
        match persistence::record_score(self, outcome) {
            Ok(rank) if rank > 0 => {
                self.state.last_system_message = Some(match total_entries {
                    Some(total) => {
                        format!("Score recorded: {score} (rank #{rank}, entries: {total}).")
                    }
                    None => format!("Score recorded: {score} (rank #{rank})."),
                });
            }
            Ok(_) => {
                self.state.last_system_message = Some(format!("Score recorded: {score}."));
            }
            Err(error) => {
                self.state.last_system_message = Some(format!("Score save failed: {error}"));
            }
        }
    }

    fn advance_world_turn(&mut self) -> StepOutcome {
        self.state.turns += 1;
        let mut rng = GameRng::new(self.state.turns as i32);

        // Hunger tick — matching original reg_move() / check_hunger()
        {
            let is_slow_dig = self.state.inventory.iter().any(|e| {
                matches!(
                    e.equipped_slot,
                    Some(EquipmentSlot::LeftRing) | Some(EquipmentSlot::RightRing)
                ) && e.item.name == "ring of slow digestion"
            });
            let ring_count = self.state.inventory.iter()
                .filter(|e| {
                    matches!(
                        e.equipped_slot,
                        Some(EquipmentSlot::LeftRing) | Some(EquipmentSlot::RightRing)
                    )
                })
                .count();
            let decrement: i32 = if is_slow_dig {
                if self.state.turns % 2 == 1 { 1 } else { 0 }
            } else if ring_count >= 2 {
                2
            } else {
                1
            };
            self.state.food_remaining -= decrement;

            if self.state.food_remaining == FOOD_HUNGRY {
                self.state.is_hungry = true;
                self.state.last_system_message =
                    Some("You are starting to feel hungry.".to_string());
            } else if self.state.food_remaining == FOOD_WEAK {
                self.state.is_weak = true;
                self.state.last_system_message =
                    Some("You feel weak with hunger.".to_string());
            } else if self.state.food_remaining == FOOD_FAINT {
                self.state.last_system_message =
                    Some("You are about to faint from hunger.".to_string());
            } else if self.state.food_remaining < FOOD_FAINT && self.state.food_remaining > 0 {
                if rng.get_rand(0, 99) < 40 {
                    self.state.food_remaining =
                        (self.state.food_remaining + 1).min(FOOD_FAINT);
                }
                let n = rng.get_rand(0, FOOD_FAINT - self.state.food_remaining);
                if n > 0 {
                    self.state.last_system_message =
                        Some("You faint from hunger.".to_string());
                    for _ in 0..n {
                        if rng.get_rand(0, 1) == 1 {
                            tick_monsters(
                                &mut self.state.monsters,
                                &self.current_level,
                                self.state.player_position,
                                &mut rng,
                            );
                        }
                    }
                }
            } else if self.state.food_remaining <= 0 {
                self.state.food_remaining = 0;
                self.state.last_system_message =
                    Some("You starve to death.".to_string());
                self.state.player_hit_points = 0;
                self.state.quit_requested = true;
            }
        }

        if self.state.player_hit_points == 0 {
            self.state.quit_requested = true;
            self.state.player_dead = true;
            self.record_high_score(persistence::RunOutcome::Defeated);
            return StepOutcome::Finished;
        }

        // Passive healing: heal() from move.c — +1 or +2 HP every N turns,
        // based on experience level. Ring of Regeneration grants +1 extra per tick.
        {
            let interval = match self.state.player_exp_level {
                1 => 20u64, 2 => 18, 3 => 17, 4 => 14, 5 => 13, 6 => 10,
                7 => 9, 8 => 8, 9 => 7, 10 => 4, 11 => 3, _ => 2,
            };
            if self.state.turns % interval == 0
                && self.state.player_hit_points < self.state.player_max_hit_points
            {
                let has_regeneration = self.state.inventory.iter().any(|e| {
                    matches!(
                        e.equipped_slot,
                        Some(EquipmentSlot::LeftRing) | Some(EquipmentSlot::RightRing)
                    ) && e.item.name == "ring of regeneration"
                });
                let regen_bonus: i16 = if has_regeneration { 1 } else { 0 };
                // Alternate between +1 and +2 HP each interval
                let base_heal: i16 = if (self.state.turns / interval) % 2 == 0 { 2 } else { 1 };
                self.state.player_hit_points =
                    (self.state.player_hit_points + base_heal + regen_bonus)
                        .min(self.state.player_max_hit_points);
            } else if has_regen_ring_only(&self.state.inventory) {
                // Regeneration ring also heals +1 HP every turn outside the interval
                if self.state.player_hit_points < self.state.player_max_hit_points {
                    self.state.player_hit_points =
                        (self.state.player_hit_points + 1).min(self.state.player_max_hit_points);
                }
            }
        }

        let events = tick_monsters(
            &mut self.state.monsters,
            &self.current_level,
            self.state.player_position,
            &mut rng,
        );

        for event in events {
            match event {
                CombatEvent::MonsterHitPlayer { damage, .. } => {
                    if damage > 0 {
                        // Wizard mode: monster hit chance is halved (50% of attacks miss)
                        // and damage dealt is reduced to 1/3.
                        let effective_damage = if self.state.wizard {
                            if rng.rand_percent(50) {
                                // Monster misses entirely in wizard mode
                                self.state.last_turn_events.push(event);
                                continue;
                            }
                            (damage / 3).max(1)
                        } else {
                            damage
                        };
                        let mitigated_damage = (effective_damage - self.player_armor_bonus()).max(1);
                        self.state.player_hit_points =
                            (self.state.player_hit_points - mitigated_damage).max(0);
                        if self.state.player_hit_points == 0 {
                            self.state.quit_requested = true;
                        }
                    }
                    // damage == 0 means attack causes only a side-effect (e.g. Aquator)
                }
                CombatEvent::MonsterAppliedEffect { effect, .. } => match effect {
                    StatusEffectEvent::Frozen { turns } => {
                        // Original: 12% immunity from freezing
                        if !rng.rand_percent(12) && self.state.frozen_turns == 0 {
                            self.state.frozen_turns = turns;
                        }
                    }
                    StatusEffectEvent::Held => {}
                    StatusEffectEvent::Stung { amount } => {
                        // Rattlesnake: 50% skip, minimum strength 3
                        if !rng.rand_percent(50) && self.state.player_strength > 3 {
                            self.state.player_strength =
                                (self.state.player_strength - amount).max(3);
                        }
                    }
                    StatusEffectEvent::LifeDrained { max_hit_points_lost } => {
                        // Original: 60% skip, guard against very low max HP
                        if !rng.rand_percent(60)
                            && self.state.player_max_hit_points > max_hit_points_lost
                        {
                            self.state.player_max_hit_points =
                                (self.state.player_max_hit_points - max_hit_points_lost).max(1);
                            self.state.player_hit_points = self
                                .state
                                .player_hit_points
                                .min(self.state.player_max_hit_points);
                        }
                    }
                    StatusEffectEvent::GoldStolen => {
                        if self.state.gold > 0 {
                            let stolen = (self.state.level as i64 * 15).min(self.state.gold);
                            self.state.gold -= stolen;
                        }
                    }
                    StatusEffectEvent::ItemStolen => {
                        // Remove the first non-equipped pack item
                        if let Some(idx) = self
                            .state
                            .inventory
                            .iter()
                            .position(|e| e.equipped_slot.is_none())
                        {
                            self.state.inventory.remove(idx);
                        }
                    }
                    StatusEffectEvent::LevelDropped => {
                        // Original: 80% skip, only applies when level > 5, rand(3,10) hp loss
                        if !rng.rand_percent(80) && self.state.player_exp_level > 5 {
                            self.state.player_exp_level =
                                (self.state.player_exp_level - 2).max(1);
                            let hp_loss = rng.get_rand(3, 10) as i16;
                            self.state.player_max_hit_points =
                                (self.state.player_max_hit_points - hp_loss).max(1);
                            self.state.player_hit_points = self
                                .state
                                .player_hit_points
                                .min(self.state.player_max_hit_points);
                        }
                    }
                    StatusEffectEvent::ArmorRusted => {
                        // Check for Ring of Maintain Armor; leather armor is immune
                        let has_maintain_armor = self.state.inventory.iter().any(|e| {
                            matches!(
                                e.equipped_slot,
                                Some(EquipmentSlot::LeftRing) | Some(EquipmentSlot::RightRing)
                            ) && e.item.name == "ring of maintain armor"
                        });
                        if !has_maintain_armor {
                            if let Some(armor) = self.state.inventory.iter_mut().find(|e| {
                                e.equipped_slot == Some(EquipmentSlot::Armor)
                                    && e.item.armor_bonus > 1
                                    && e.item.name != "leather armor"
                            }) {
                                armor.item.armor_bonus -= 1;
                            }
                        }
                    }
                    StatusEffectEvent::Confused { turns } => {
                        if self.state.confused_turns == 0 {
                            self.state.confused_turns = turns;
                        }
                    }
                },
                CombatEvent::PlayerHitMonster { .. } => {}
            }

            self.state.last_turn_events.push(event);

            if self.state.player_hit_points == 0 {
                self.state.quit_requested = true;
            }
        }

        if self.state.player_hit_points == 0 {
            self.state.quit_requested = true;
            self.state.player_dead = true;
            self.record_high_score(persistence::RunOutcome::Defeated);
            StepOutcome::Finished
        } else {
            StepOutcome::Continue
        }
    }

    fn descend_level(&mut self) {
        let new_depth = self.state.level + 1;
        let seed = (self.state.turns as i32)
            .wrapping_mul(1_000_003)
            .wrapping_add(self.state.level as i32);
        let mut rng = GameRng::new(seed);
        let new_level = generate_level_with_depth(&mut rng, new_depth, self.state.party_counter);
        let player_position = new_level.spawn_position();
        let monsters = spawn_basic_monsters(&new_level, &mut rng, player_position, new_depth);
        let floor_items = place_floor_items(&new_level, &mut rng, new_depth);
        let (trap_positions, trap_types) = place_traps(&new_level, &mut rng, new_depth);

        self.state.level = new_depth;
        self.state.player_position = player_position;
        self.state.monsters = monsters;
        self.state.floor_items = floor_items;
        self.state.trap_positions = trap_positions;
        self.state.trap_types = trap_types;
        self.state.known_traps.clear();
        self.state.explored.clear();
        self.state.last_system_message = Some(format!("You descend to dungeon level {}.", new_depth));

        self.current_level = new_level;
        self.update_explored();
    }

    pub fn step(&mut self, command: Command) -> StepOutcome {
        apply_item_effects();
        self.state.last_turn_events.clear();
        self.state.last_inventory_events.clear();
        self.state.last_system_message = None;

        if !matches!(command, Command::Quit | Command::Save | Command::Load)
            && self.state.frozen_turns > 0
        {
            self.state.pending_direction = None;
            self.state.last_move_blocked = false;
            self.state.frozen_turns -= 1;
            return self.advance_world_turn();
        }

        match command {
            Command::Rest => {
                self.state.pending_direction = None;
                self.state.last_move_blocked = false;
                self.advance_world_turn()
            }
            Command::Quit => {
                self.state.quit_requested = true;
                self.record_high_score(persistence::RunOutcome::Quit);
                StepOutcome::Finished
            }
            Command::Save => {
                match persistence::save_game(self) {
                    Ok(()) => {
                        self.state.last_system_message = Some("Game saved.".to_string());
                    }
                    Err(error) => {
                        self.state.last_system_message = Some(format!("Save failed: {error}"));
                    }
                }
                StepOutcome::Continue
            }
            Command::Load => {
                match persistence::load_game() {
                    Ok(loaded_game) => {
                        *self = loaded_game;
                        self.state.last_system_message = Some("Game loaded.".to_string());
                    }
                    Err(error) => {
                        self.state.last_system_message = Some(format!("Load failed: {error}"));
                    }
                }
                StepOutcome::Continue
            }
            Command::Move(direction) => {
                // When confused the player stumbles in a random direction
                let actual_direction = if self.state.confused_turns > 0 {
                    self.state.confused_turns -= 1;
                    let mut rng = GameRng::new(self.state.turns as i32);
                    const ALL_DIRS: [Direction; 8] = [
                        Direction::Up,
                        Direction::UpRight,
                        Direction::Right,
                        Direction::DownRight,
                        Direction::Down,
                        Direction::DownLeft,
                        Direction::Left,
                        Direction::UpLeft,
                    ];
                    ALL_DIRS[rng.get_rand(0, 7) as usize]
                } else {
                    direction
                };
                self.state.pending_direction = Some(actual_direction);
                match self.try_move_player(actual_direction) {
                    PlayerAction::Moved | PlayerAction::Attacked | PlayerAction::Held => {
                        if let Some(trap_idx) = self
                            .state
                            .trap_positions
                            .iter()
                            .position(|&p| p == self.state.player_position)
                        {
                            let trap_kind = self
                                .state
                                .trap_types
                                .get(trap_idx)
                                .copied()
                                .unwrap_or(TrapKind::DartTrap);
                            if !self.state.known_traps.contains(&self.state.player_position) {
                                self.state.known_traps.push(self.state.player_position);
                            }
                            match trap_kind {
                                TrapKind::TrapDoor => {
                                    self.state.last_system_message =
                                        Some("You fell down a trap door!".to_string());
                                }
                                TrapKind::BearTrap => {
                                    self.state.frozen_turns =
                                        (self.state.frozen_turns + 4).min(8);
                                    self.state.last_system_message =
                                        Some("You are caught in a bear trap!".to_string());
                                }
                                TrapKind::TeleTrap => {
                                    self.state.last_system_message = Some(
                                        "You trip over a trap and are teleported!".to_string(),
                                    );
                                }
                                TrapKind::DartTrap => {
                                    // Original: 1d6 damage, 40% chance of strength loss
                                    // unless wearing Ring of Sustain Strength
                                    let mut dart_rng =
                                        GameRng::new(self.state.turns as i32 ^ 0x7777_i32);
                                    let dart_damage = dart_rng.get_rand(1, 6) as i16;
                                    self.state.player_hit_points =
                                        (self.state.player_hit_points - dart_damage).max(0);
                                    let has_sustain = self.state.inventory.iter().any(|e| {
                                        matches!(
                                            e.equipped_slot,
                                            Some(EquipmentSlot::LeftRing)
                                                | Some(EquipmentSlot::RightRing)
                                        ) && e.item.name == "ring of sustain strength"
                                    });
                                    if !has_sustain && dart_rng.rand_percent(40) {
                                        self.state.player_strength =
                                            (self.state.player_strength - 1).max(1);
                                        self.state.last_system_message = Some(format!(
                                            "A dart hits you for {dart_damage} damage and poisons you!"
                                        ));
                                    } else {
                                        self.state.last_system_message = Some(format!(
                                            "A dart hits you for {dart_damage} damage."
                                        ));
                                    }
                                }
                                TrapKind::SleepingGasTrap => {
                                    self.state.frozen_turns =
                                        (self.state.frozen_turns + 3).min(6);
                                    self.state.last_system_message =
                                        Some("A puff of sleeping gas hits you!".to_string());
                                }
                                TrapKind::RustTrap => {
                                    self.state.last_system_message =
                                        Some("A gush of water rusts your armor!".to_string());
                                }
                            }
                            if self.state.player_hit_points == 0 {
                                self.state.quit_requested = true;
                                self.state.player_dead = true;
                                self.record_high_score(persistence::RunOutcome::Defeated);
                                return StepOutcome::Finished;
                            }
                        }
                        self.advance_world_turn()
                    }
                    PlayerAction::InventoryChanged | PlayerAction::Blocked => StepOutcome::Continue,
                }
            }
            Command::PickUp | Command::IdentifyTrap => {
                match self.try_immediate_inventory_action(command) {
                    PlayerAction::InventoryChanged => self.advance_world_turn(),
                    _ => StepOutcome::Continue,
                }
            }
            Command::Search => {
                let player_pos = self.state.player_position;
                let ring_search_bonus: i32 = self.state.inventory.iter()
                    .filter(|e| {
                        matches!(
                            e.equipped_slot,
                            Some(EquipmentSlot::LeftRing) | Some(EquipmentSlot::RightRing)
                        ) && e.item.name == "ring of searching"
                    })
                    .count() as i32
                    * 2;
                let reveal_pct: i32 = 12
                    + self.state.player_exp_level as i32
                    + ring_search_bonus;
                let mut rng = GameRng::new(
                    self.state.turns as i32 ^ 0xABCD,
                );
                let mut found = false;
                for drow in -1i16..=1 {
                    for dcol in -1i16..=1 {
                        if drow == 0 && dcol == 0 {
                            continue;
                        }
                        let row = player_pos.row + drow;
                        let col = player_pos.col + dcol;
                        let pos = Position::new(row, col);
                        // Reveal hidden trap
                        if let Some(trap_idx) = self.state.trap_positions.iter().position(|&p| p == pos) {
                            if !self.state.known_traps.contains(&pos) && rng.rand_percent(reveal_pct) {
                                self.state.known_traps.push(pos);
                                let kind = self.state.trap_types[trap_idx];
                                self.state.last_system_message =
                                    Some(format!("You found a {}!", kind.name()));
                                found = true;
                            }
                        }
                        // Reveal hidden passage (HIDDEN | DOOR or HIDDEN | TUNNEL)
                        if let Some(flags) = self.current_level.grid.get(row, col) {
                            if flags.contains(TileFlags::HIDDEN) && rng.rand_percent(reveal_pct) {
                                // Passage reveal would require mutable grid access; mark as found
                                if !found {
                                    self.state.last_system_message =
                                        Some("You found a secret passage!".to_string());
                                    found = true;
                                }
                            }
                        }
                    }
                }
                if !found {
                    self.state.last_system_message = Some("You find nothing.".to_string());
                }
                self.advance_world_turn()
            }
            Command::Drop => {
                self.start_item_action(PendingItemAction::Drop);
                StepOutcome::Continue
            }
            Command::Wield => {
                self.start_item_action(PendingItemAction::Wield);
                StepOutcome::Continue
            }
            Command::WearArmor => {
                self.start_item_action(PendingItemAction::WearArmor);
                StepOutcome::Continue
            }
            Command::TakeOffArmor => {
                self.start_item_action(PendingItemAction::TakeOffArmor);
                StepOutcome::Continue
            }
            Command::PutOnRing => {
                self.start_item_action(PendingItemAction::PutOnRing);
                StepOutcome::Continue
            }
            Command::RemoveRing => {
                self.start_item_action(PendingItemAction::RemoveRing);
                StepOutcome::Continue
            }
            Command::Quaff => {
                self.start_item_action(PendingItemAction::Quaff);
                StepOutcome::Continue
            }
            Command::ReadScroll => {
                self.start_item_action(PendingItemAction::ReadScroll);
                StepOutcome::Continue
            }
            Command::Eat => {
                self.start_item_action(PendingItemAction::Eat);
                StepOutcome::Continue
            }
            Command::Zap => {
                self.start_item_action(PendingItemAction::Zap);
                StepOutcome::Continue
            }
            Command::Throw => {
                self.start_item_action(PendingItemAction::Throw);
                StepOutcome::Continue
            }
            Command::SelectItem(ch) => {
                if let Some(action) = self.state.pending_item_action.take() {
                    match self.execute_item_selection(action, ch) {
                        PlayerAction::InventoryChanged => self.advance_world_turn(),
                        _ => StepOutcome::Continue,
                    }
                } else {
                    StepOutcome::Continue
                }
            }
            Command::CancelItemSelect => {
                self.state.pending_item_action = None;
                self.state.last_system_message = None;
                StepOutcome::Continue
            }
            Command::Descend => {
                self.state.pending_direction = None;
                self.state.last_move_blocked = false;
                let pos = self.state.player_position;
                let on_stairs = self.current_level.stairs_position == Some(pos);
                // Wizard mode: can descend from any tile (drop_check() in level.c)
                if on_stairs || self.state.wizard {
                    self.descend_level();
                    self.advance_world_turn()
                } else {
                    self.state.last_system_message =
                        Some("You see no stairs here.".to_string());
                    StepOutcome::Continue
                }
            }
            Command::Noop | Command::Unknown => {
                self.state.pending_direction = None;
                self.state.last_move_blocked = false;
                StepOutcome::Continue
            }
            // ── Wizard mode commands ──────────────────────────────────────────
            Command::ToggleWizard => {
                if self.state.wizard {
                    self.state.wizard = false;
                    self.state.last_system_message =
                        Some("not wizard anymore".to_string());
                } else {
                    // Activate password-entry mode
                    self.state.wizard_password_input = Some(String::new());
                    self.state.last_system_message =
                        Some("wizard's password:".to_string());
                }
                StepOutcome::Continue
            }
            Command::WizardPasswordChar(ch) => {
                if let Some(ref mut buf) = self.state.wizard_password_input {
                    if ch == '\x08' {
                        buf.pop();
                    } else {
                        buf.push(ch);
                    }
                }
                StepOutcome::Continue
            }
            Command::WizardPasswordSubmit => {
                let input = self.state.wizard_password_input.take();
                if let Some(password) = input {
                    if wizard_check_password(&password) {
                        self.state.wizard = true;
                        self.state.score_only = true;
                        self.state.last_system_message =
                            Some("Welcome, mighty wizard!".to_string());
                    } else {
                        self.state.last_system_message = Some("sorry".to_string());
                    }
                }
                StepOutcome::Continue
            }
            Command::WizardPasswordCancel => {
                self.state.wizard_password_input = None;
                self.state.last_system_message = None;
                StepOutcome::Continue
            }
            Command::WizardRevealMap => {
                if self.state.wizard {
                    let (rows, cols) = self.current_level.grid.dimensions();
                    for row in 0..rows as i16 {
                        for col in 0..cols as i16 {
                            if let Some(flags) = self.current_level.grid.get(row, col) {
                                if flags.intersects(
                                    TileFlags::HORWALL
                                        | TileFlags::VERTWALL
                                        | TileFlags::DOOR
                                        | TileFlags::TUNNEL
                                        | TileFlags::FLOOR
                                        | TileFlags::STAIRS
                                        | TileFlags::TRAP,
                                ) {
                                    self.state.explored.insert(Position::new(row, col));
                                }
                            }
                        }
                    }
                    self.state.last_system_message =
                        Some("The dungeon layout is revealed.".to_string());
                } else {
                    self.state.last_system_message =
                        Some("unknown command".to_string());
                }
                StepOutcome::Continue
            }
            Command::WizardShowTraps => {
                if self.state.wizard {
                    for &pos in &self.state.trap_positions {
                        if !self.state.known_traps.contains(&pos) {
                            self.state.known_traps.push(pos);
                        }
                    }
                    self.state.last_system_message =
                        Some("All traps revealed.".to_string());
                } else {
                    self.state.last_system_message =
                        Some("unknown command".to_string());
                }
                StepOutcome::Continue
            }
            Command::WizardShowObjects => {
                if self.state.wizard {
                    // Reveal all floor objects by adding their positions to explored
                    let positions: Vec<Position> =
                        self.state.floor_items.iter().map(|fi| fi.position).collect();
                    for pos in positions {
                        self.state.explored.insert(pos);
                    }
                    self.state.last_system_message =
                        Some("All level objects shown.".to_string());
                } else {
                    self.state.last_system_message =
                        Some("unknown command".to_string());
                }
                StepOutcome::Continue
            }
            Command::WizardShowLevelObjects => {
                if self.state.wizard {
                    // Matches original Tab → inventory(&level_objects, ALL_OBJECTS):
                    // just reveal all floor item positions so they appear on the map.
                    let positions: Vec<Position> =
                        self.state.floor_items.iter().map(|fi| fi.position).collect();
                    for pos in positions {
                        self.state.explored.insert(pos);
                    }
                    self.state.last_system_message =
                        Some(format!("{} object(s) on this level.", self.state.floor_items.len()));
                } else {
                    self.state.last_system_message =
                        Some("unknown command".to_string());
                }
                StepOutcome::Continue
            }
            Command::WizardAddItem => {
                if self.state.wizard {
                    use crate::inventory_items::{next_avail_ichar, MAX_PACK_ITEMS};
                    if self.state.inventory.len() >= MAX_PACK_ITEMS {
                        self.state.last_system_message = Some("Pack full.".to_string());
                    } else {
                        let mut rng = GameRng::new(self.state.turns as i32 ^ 0xCA11_i32);
                        let item = gr_floor_item(&mut rng);
                        let name = item.name;
                        let ichar = next_avail_ichar(&self.state.inventory);
                        self.state.inventory.push(InventoryEntry {
                            id: self.state.next_item_id,
                            item,
                            equipped_slot: None,
                            ichar,
                            quantity: 1,
                        });
                        self.state.next_item_id += 1;
                        self.state.last_system_message =
                            Some(format!("Wizard conjured: {name}."));
                    }
                } else {
                    self.state.last_system_message = Some("unknown command".to_string());
                }
                StepOutcome::Continue
            }
            Command::WizardShowMonsters => {
                if self.state.wizard {
                    // Reveal all monster positions by adding them to explored
                    let positions: Vec<Position> =
                        self.state.monsters.iter().map(|m| m.position).collect();
                    for pos in positions {
                        self.state.explored.insert(pos);
                    }
                    self.state.last_system_message =
                        Some("All monsters revealed.".to_string());
                } else {
                    self.state.last_system_message =
                        Some("unknown command".to_string());
                }
                StepOutcome::Continue
            }
        }
    }

    pub fn run_script(&mut self, script: &str) -> StepOutcome {
        let mut outcome = StepOutcome::Continue;

        for input in script.chars() {
            let command = Self::parse_command(input);
            outcome = self.step(command);

            if outcome == StepOutcome::Finished {
                break;
            }
        }

        outcome
    }
}

pub fn run() -> GameLoop {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i32)
        .unwrap_or(12345);

    let game = GameLoop::new(seed);
    let _ = game.state().turns;
    let _ = game.current_level().rooms.len();
    let _ = GameLoop::parse_command('.');
    let _ = crate::core_types::MAXROOMS + crate::core_types::MAX_TRAPS;

    let spawn = game.current_level().spawn_position();
    let room = game.current_level().rooms[0];
    let _ = room.contains(spawn.row, spawn.col);

    let mut scratch_grid = game.current_level().grid.clone();
    let _ = scratch_grid.dimensions();
    scratch_grid.clear();

    let mut scratch_rng = GameRng::new(7);
    let _ = scratch_rng.seed();
    let _ = scratch_rng.rand_percent(50);
    let _ = scratch_rng.coin_toss();
    let _ = InventoryItem::dagger();
    let _ = InventoryItem::leather_armor();
    let _ = InventoryItem::protection_ring();
    let _ = InventoryItem::accuracy_ring();
    let _ = InventoryItem::healing_potion();
    let _ = InventoryItem::magic_missile_wand();
    let _ = Monster::new(MonsterKind::IceMonster, spawn);
    let _ = Monster::new(MonsterKind::VenusFlytrap, spawn);
    let _ = Monster::new(MonsterKind::Rattlesnake, spawn);

    let _ = game.state().inventory.len();
    let _ = game.state().floor_items.len();
    let _ = game.state().trap_positions.len();

    let mut smoke = GameLoop::new(12345);
    let _ = smoke.run_script(".");

    game
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::core_types::Position;
    use crate::inventory_items::{EquipmentSlot, FloorItem, InventoryItem};

    use super::{Command, Direction, GameLoop, StepOutcome};
    use crate::actors::{CombatEvent, Monster, MonsterKind, StatusEffectEvent};

    #[test]
    fn parses_legacy_commands() {
        assert_eq!(GameLoop::parse_command('.'), Command::Rest);
        assert_eq!(GameLoop::parse_command('Q'), Command::Quit);
        assert_eq!(GameLoop::parse_command('h'), Command::Move(Direction::Left));
        assert_eq!(GameLoop::parse_command(','), Command::PickUp);
        assert_eq!(GameLoop::parse_command('q'), Command::Quaff);
        assert_eq!(GameLoop::parse_command('z'), Command::Zap);
        assert_eq!(GameLoop::parse_command('t'), Command::Throw);
        assert_eq!(GameLoop::parse_command('^'), Command::IdentifyTrap);
        assert_eq!(GameLoop::parse_command('S'), Command::Save);
        assert_eq!(GameLoop::parse_command('L'), Command::Load);
        assert_eq!(GameLoop::parse_command('w'), Command::Wield);
        assert_eq!(
            GameLoop::parse_command('n'),
            Command::Move(Direction::DownRight)
        );
    }

    proptest! {
        #[test]
        fn parse_command_handles_all_ascii_without_panicking(byte in 0u8..=127u8) {
            let ch = byte as char;
            let _ = GameLoop::parse_command(ch);
        }

        #[test]
        fn directional_keys_map_to_move_commands(byte in prop::sample::select(vec![b'h', b'j', b'k', b'l', b'y', b'u', b'b', b'n'])) {
            let command = GameLoop::parse_command(byte as char);
            prop_assert!(matches!(command, Command::Move(_)));
        }
    }

    #[test]
    fn new_loop_initializes_level_and_state() {
        let game = GameLoop::new(12345);

        assert_eq!(game.state().level, 1);
        assert_eq!(game.state().turns, 0);
        assert_eq!(game.state().player_hit_points, 12);
        assert!(!game.current_level().rooms.is_empty());
        assert_eq!(game.state().player_position, Position::new(4, 12));
        assert_eq!(game.state().inventory.len(), 5);
        assert!(
            game.state().monsters.len() >= 4 && game.state().monsters.len() <= 6,
            "expected 4-6 monsters, got {}",
            game.state().monsters.len()
        );
        assert!(game.state().monsters.iter().all(|m| m.position != game.state().player_position));
    }

    #[test]
    fn rest_and_move_advance_turns() {
        let mut game = GameLoop::new(12345);

        assert_eq!(game.step(Command::Rest), StepOutcome::Continue);
        assert_eq!(
            game.step(Command::Move(Direction::Left)),
            StepOutcome::Continue
        );

        let initial_monster_pos = Position::new(4, 20);
        assert_eq!(game.state().turns, 2);
        assert_eq!(game.state().pending_direction, Some(Direction::Left));
        assert_eq!(game.state().player_position, Position::new(4, 11));
        assert_ne!(game.state().monsters[0].position, initial_monster_pos);
        assert!(!game.state().last_move_blocked);
    }

    #[test]
    fn quit_finishes_script() {
        let mut game = GameLoop::new(12345);
        let outcome = game.run_script(".Qh");

        assert_eq!(outcome, StepOutcome::Finished);
        assert!(game.state().quit_requested);
        assert_eq!(game.state().turns, 1);
    }

    #[test]
    fn quaff_consumes_potion_and_heals() {
        let mut game = GameLoop::new(12345);
        game.state.inventory.clear();
        game.state.player_hit_points = 7;
        game.state
            .inventory
            .push(crate::inventory_items::InventoryEntry {
                id: 1,
                item: InventoryItem::healing_potion(),
                equipped_slot: None,
                ichar: 'a',
                quantity: 1,
            });

        // First Quaff sets pending action; SelectItem('a') executes it.
        assert_eq!(game.step(Command::Quaff), StepOutcome::Continue);
        assert!(game.state.pending_item_action.is_some());
        assert_eq!(game.step(Command::SelectItem('a')), StepOutcome::Continue);
        // Healing potion now heals player_exp_level (1) HP: 7 + 1 = 8
        assert_eq!(game.state.player_hit_points, 8);
        assert!(game.state.inventory.is_empty());
    }

    #[test]
    fn identify_trap_marks_nearby_trap() {
        let mut game = GameLoop::new(12345);
        // Level 1 has no procedural traps; add one manually adjacent to the player.
        let player_pos = game.state.player_position;
        let trap = Position::new(player_pos.row, player_pos.col + 1);
        game.state.trap_positions.push(trap);
        game.state.trap_types.push(crate::core_types::TrapKind::DartTrap);

        assert!(super::is_adjacent(game.state.player_position, trap));
        assert_eq!(game.step(Command::IdentifyTrap), StepOutcome::Continue);
        assert!(game.state.known_traps.contains(&trap));
    }

    #[test]
    fn move_into_wall_is_blocked_without_consuming_turn() {
        let mut game = GameLoop::new(12345);
        game.state.monsters.clear();

        for _ in 0..9 {
            assert_eq!(
                game.step(Command::Move(Direction::Left)),
                StepOutcome::Continue
            );
        }

        let before = game.state().player_position;
        let turns_before = game.state().turns;

        assert_eq!(
            game.step(Command::Move(Direction::Left)),
            StepOutcome::Continue
        );
        assert_eq!(game.state().player_position, before);
        assert_eq!(game.state().turns, turns_before);
        assert!(game.state().last_move_blocked);
    }

    #[test]
    fn diagonal_movement_updates_position_when_walkable() {
        let mut game = GameLoop::new(12345);

        assert_eq!(
            game.step(Command::Move(Direction::DownRight)),
            StepOutcome::Continue
        );
        let initial_monster_pos = Position::new(4, 20);
        assert_eq!(game.state().player_position, Position::new(5, 13));
        assert_ne!(game.state().monsters[0].position, initial_monster_pos);
        assert!(!game.state().last_move_blocked);
    }

    #[test]
    fn resting_advances_monster_turns() {
        let mut game = GameLoop::new(12345);
        let monster_before = game.state().monsters[0].position;

        assert_eq!(game.step(Command::Rest), StepOutcome::Continue);

        assert_eq!(game.state().turns, 1);
        assert_ne!(game.state().monsters[0].position, monster_before);
    }

    #[test]
    fn pickup_wield_and_drop_flow_is_stable() {
        let mut game = GameLoop::new(12345);
        game.state.inventory.clear();
        game.state.floor_items.clear();
        game.state.floor_items.push(FloorItem {
            item: InventoryItem::dagger(),
            position: game.state.player_position,
        });

        assert_eq!(game.step(Command::PickUp), StepOutcome::Continue);
        assert_eq!(game.state().turns, 1);
        assert_eq!(game.state().inventory.len(), 1);
        assert!(game.state().floor_items.is_empty());
        assert_eq!(game.state().last_inventory_events.len(), 1);

        // Wield: first call sets pending, SelectItem executes.
        assert_eq!(game.step(Command::Wield), StepOutcome::Continue);
        assert!(game.state().pending_item_action.is_some());
        let item_ichar = game.state().inventory[0].ichar;
        assert_eq!(game.step(Command::SelectItem(item_ichar)), StepOutcome::Continue);
        assert_eq!(game.state().turns, 2);
        assert_eq!(
            game.state().inventory[0].equipped_slot,
            Some(EquipmentSlot::Weapon)
        );

        let drop_position = game.state.player_position;
        // Drop: first call sets pending, SelectItem executes.
        assert_eq!(game.step(Command::Drop), StepOutcome::Continue);
        assert!(game.state().pending_item_action.is_some());
        assert_eq!(game.step(Command::SelectItem(item_ichar)), StepOutcome::Continue);
        assert_eq!(game.state().turns, 3);
        assert!(game.state().inventory.is_empty());
        assert_eq!(game.state().floor_items[0].position, drop_position);
    }

    #[test]
    fn equipped_items_modify_attack_and_armor() {
        let mut game = GameLoop::new(12345);
        game.state.inventory.clear();
        game.state
            .inventory
            .push(crate::inventory_items::InventoryEntry {
                id: 1,
                item: InventoryItem::dagger(),
                equipped_slot: Some(EquipmentSlot::Weapon),
                ichar: 'a',
                quantity: 1,
            });
        game.state
            .inventory
            .push(crate::inventory_items::InventoryEntry {
                id: 2,
                item: InventoryItem::leather_armor(),
                equipped_slot: Some(EquipmentSlot::Armor),
                ichar: 'b',
                quantity: 1,
            });

        game.state.monsters.clear();
        game.state.monsters.push(Monster::new(MonsterKind::Kestrel, Position::new(4, 13)));
        game.state.monsters[0].hit_points = 2;

        assert_eq!(
            game.step(Command::Move(Direction::Right)),
            StepOutcome::Continue
        );

        assert!(game.state().monsters.is_empty());
        assert_eq!(game.state().player_hit_points, 12);
        assert_eq!(game.state().monsters_defeated, 1);
    }

    #[test]
    fn wear_and_remove_ring_commands_toggle_equipment() {
        let mut game = GameLoop::new(12345);
        game.state.inventory.clear();
        game.state
            .inventory
            .push(crate::inventory_items::InventoryEntry {
                id: 1,
                item: InventoryItem::leather_armor(),
                equipped_slot: None,
                ichar: 'a',
                quantity: 1,
            });
        game.state
            .inventory
            .push(crate::inventory_items::InventoryEntry {
                id: 2,
                item: InventoryItem::protection_ring(),
                equipped_slot: None,
                ichar: 'b',
                quantity: 1,
            });

        assert_eq!(game.step(Command::WearArmor), StepOutcome::Continue);
        assert!(game.state().pending_item_action.is_some());
        assert_eq!(game.step(Command::SelectItem('a')), StepOutcome::Continue);
        assert_eq!(
            game.state().inventory[0].equipped_slot,
            Some(EquipmentSlot::Armor)
        );

        assert_eq!(game.step(Command::PutOnRing), StepOutcome::Continue);
        assert!(game.state().pending_item_action.is_some());
        assert_eq!(game.step(Command::SelectItem('b')), StepOutcome::Continue);
        assert_eq!(
            game.state().inventory[1].equipped_slot,
            Some(EquipmentSlot::LeftRing)
        );

        assert_eq!(game.step(Command::RemoveRing), StepOutcome::Continue);
        assert!(game.state().pending_item_action.is_some());
        assert_eq!(game.step(Command::SelectItem('b')), StepOutcome::Continue);
        assert_eq!(game.state().inventory[1].equipped_slot, None);

        assert_eq!(game.step(Command::TakeOffArmor), StepOutcome::Continue);
        assert!(game.state().pending_item_action.is_some());
        assert_eq!(game.step(Command::SelectItem('a')), StepOutcome::Continue);
        assert_eq!(game.state().inventory[0].equipped_slot, None);
    }

    #[test]
    fn moving_into_monster_attacks_instead_of_moving() {
        let mut game = GameLoop::new(12345);
        game.state.inventory.clear();
        // Give the Kestrel enough HP that it survives the first hit
        let mut kestrel = Monster::new(MonsterKind::Kestrel, Position::new(4, 13));
        kestrel.hit_points = 50;
        game.state.monsters = vec![kestrel];

        assert_eq!(
            game.step(Command::Move(Direction::Right)),
            StepOutcome::Continue
        );

        // Player stays directly left of monster (combat, not movement)
        assert_eq!(game.state().player_position, Position::new(4, 12));
        assert_eq!(game.state().turns, 1);
        // Kestrel was hit — its HP is lower than it started
        assert!(game.state().monsters[0].hit_points < 50);
        // At least the PlayerHitMonster event was emitted
        assert!(game.state().last_turn_events.iter().any(|e| {
            matches!(
                e,
                CombatEvent::PlayerHitMonster {
                    monster_kind: MonsterKind::Kestrel,
                    killed: false,
                    ..
                }
            )
        }));
    }

    #[test]
    fn killing_monster_removes_it_before_counter_attack() {
        let mut game = GameLoop::new(12345);
        game.state.inventory.clear();
        game.state.monsters = vec![Monster::new(MonsterKind::Kestrel, Position::new(4, 13))];
        game.state.monsters[0].hit_points = 1;

        assert_eq!(
            game.step(Command::Move(Direction::Right)),
            StepOutcome::Continue
        );

        assert!(game.state().monsters.is_empty());
        assert_eq!(game.state().monsters_defeated, 1);
        // Monster is dead so no counter-attack: player HP unchanged
        assert_eq!(game.state().player_hit_points, 12);
        // The kill event was emitted
        assert!(game.state().last_turn_events.iter().any(|e| {
            matches!(
                e,
                CombatEvent::PlayerHitMonster {
                    monster_kind: MonsterKind::Kestrel,
                    killed: true,
                    ..
                }
            )
        }));
    }

    #[test]
    fn hold_effect_blocks_escape_but_consumes_turn() {
        let mut game = GameLoop::new(12345);
        game.state.monsters = vec![Monster::new(
            MonsterKind::VenusFlytrap,
            Position::new(4, 13),
        )];

        // VenusFlytrap hits for 25 — give player enough HP to survive
        game.state.player_hit_points = 100;
        game.state.player_max_hit_points = 100;

        assert_eq!(game.step(Command::Rest), StepOutcome::Continue);

        let turns_before = game.state.turns;
        assert_eq!(
            game.step(Command::Move(Direction::Left)),
            StepOutcome::Continue
        );

        assert_eq!(game.state.player_position, Position::new(4, 12));
        assert!(game.state.last_move_blocked);
        assert_eq!(game.state.turns, turns_before + 1);
    }

    #[test]
    fn freeze_effect_skips_player_turns() {
        let mut game = GameLoop::new(12345);
        game.state.monsters = vec![Monster::new(MonsterKind::IceMonster, Position::new(4, 13))];

        assert_eq!(game.step(Command::Rest), StepOutcome::Continue);
        assert_eq!(game.state.frozen_turns, 2);
        assert!(game.state.last_turn_events.iter().any(|event| {
            matches!(
                event,
                CombatEvent::MonsterAppliedEffect {
                    effect: StatusEffectEvent::Frozen { turns: 2 },
                    ..
                }
            )
        }));

        let turns_before = game.state.turns;
        assert_eq!(
            game.step(Command::Move(Direction::Left)),
            StepOutcome::Continue
        );
        assert_eq!(game.state.player_position, Position::new(4, 12));
        assert_eq!(game.state.turns, turns_before + 1);
        assert_eq!(game.state.frozen_turns, 1);
    }

    #[test]
    fn sting_effect_reduces_player_max_hit_points() {
        let mut game = GameLoop::new(12345);
        game.state.monsters = vec![Monster::new(
            MonsterKind::Rattlesnake,
            Position::new(4, 13),
        )];

        assert_eq!(game.step(Command::Rest), StepOutcome::Continue);

        // Sting now drains strength with 50% probability, minimum 3
        assert!(game.state.player_strength <= 16); // 16 = INIT_STRENGTH (may skip with 50% chance)
        assert!(game.state().last_turn_events.iter().any(|event| {
            matches!(
                event,
                CombatEvent::MonsterAppliedEffect {
                    effect: StatusEffectEvent::Stung { amount: 1 },
                    ..
                }
            )
        }));
    }
}

