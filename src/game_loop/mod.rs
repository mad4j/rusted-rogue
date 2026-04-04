use std::collections::HashSet;

use crate::actors::{
    attack_monster, spawn_basic_monsters, tick_monsters, CombatEvent, Monster, MonsterKind,
    SpecialHit, StatusEffectEvent,
};
use crate::core_types::{EXP_LEVELS, INIT_FOOD, INIT_STRENGTH, Position, TrapKind};
use crate::inventory_items::{
    apply_item_effects, drop_first_item, equip_first_armor, equip_first_weapon, pick_up_item,
    put_on_first_ring, remove_first_item_by_category, remove_ring, total_armor_bonus,
    total_attack_bonus, unequip_armor, FloorItem, InventoryEntry, InventoryEvent, InventoryItem,
    ItemCategory,
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
    Save,
    Load,
    Noop,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub level: i16,
    pub turns: u64,
    pub quit_requested: bool,
    pub pending_direction: Option<Direction>,
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

impl GameLoop {
    pub fn new(seed: i32) -> Self {
        let mut rng = GameRng::new(seed);
        let party_counter = rng.get_rand(1, 10) as i16;
        let current_level = generate_level_with_depth(&mut rng, 1, party_counter);
        let player_position = current_level.spawn_position();
        let monsters = spawn_basic_monsters(&current_level, &mut rng, player_position);

        let mut game = Self {
            state: GameState {
                level: 1,
                turns: 0,
                quit_requested: false,
                pending_direction: None,
                player_position,
                player_hit_points: 12,
                player_max_hit_points: 12,
                player_strength: INIT_STRENGTH,
                player_max_strength: INIT_STRENGTH,
                player_exp_points: 0,
                player_exp_level: 1,
                food_remaining: INIT_FOOD,
                is_hungry: false,
                is_weak: false,
                frozen_turns: 0,
                monsters_defeated: 0,
                monsters,
                last_turn_events: Vec::new(),
                inventory: Vec::new(),
                floor_items: vec![
                    FloorItem {
                        item: InventoryItem::healing_potion(),
                        position: Position::new(player_position.row, player_position.col + 1),
                    },
                    FloorItem {
                        item: InventoryItem::magic_missile_wand(),
                        position: Position::new(player_position.row + 1, player_position.col),
                    },
                ],
                trap_positions: vec![Position::new(player_position.row - 1, player_position.col)],
                trap_types: vec![TrapKind::DartTrap],
                known_traps: Vec::new(),
                next_item_id: 1,
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
            'S' => Command::Save,
            'L' => Command::Load,
            'y' => Command::Move(Direction::UpLeft),
            'u' => Command::Move(Direction::UpRight),
            'b' => Command::Move(Direction::DownLeft),
            'n' => Command::Move(Direction::DownRight),
            ' ' => Command::Noop,
            _ => Command::Unknown,
        }
    }

    fn player_attack_damage(&self) -> i16 {
        let str_bonus = (self.state.player_strength - INIT_STRENGTH) / 4;
        1 + str_bonus.max(0) + total_attack_bonus(&self.state.inventory)
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
            if matches!(event, CombatEvent::PlayerHitMonster { killed: true, .. }) {
                self.state.monsters_defeated += 1;
                self.state.player_exp_points += 5;
                let next_level = self.state.player_exp_level as usize;
                if next_level < EXP_LEVELS.len()
                    && self.state.player_exp_points >= EXP_LEVELS[next_level - 1]
                {
                    self.state.player_exp_level = (self.state.player_exp_level + 1).min(21);
                    self.state.player_max_hit_points += 4;
                    self.state.player_hit_points = self.state.player_max_hit_points;
                    self.state.last_system_message = Some(format!(
                        "Welcome to experience level {}!",
                        self.state.player_exp_level
                    ));
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

    fn try_inventory_action(&mut self, command: Command) -> PlayerAction {
        let events = match command {
            Command::PickUp => pick_up_item(
                &mut self.state.inventory,
                &mut self.state.floor_items,
                &mut self.state.next_item_id,
                self.state.player_position,
            )
            .map(|event| vec![event]),
            Command::Drop => drop_first_item(
                &mut self.state.inventory,
                &mut self.state.floor_items,
                self.state.player_position,
            ),
            Command::Wield => equip_first_weapon(&mut self.state.inventory),
            Command::WearArmor => equip_first_armor(&mut self.state.inventory),
            Command::TakeOffArmor => {
                unequip_armor(&mut self.state.inventory).map(|event| vec![event])
            }
            Command::PutOnRing => put_on_first_ring(&mut self.state.inventory),
            Command::RemoveRing => remove_ring(&mut self.state.inventory).map(|event| vec![event]),
            Command::Quaff => {
                remove_first_item_by_category(&mut self.state.inventory, ItemCategory::Potion).map(
                    |entry| {
                        let msg = match entry.item.name {
                            "healing potion" => {
                                self.state.player_hit_points = (self.state.player_hit_points + 4)
                                    .min(self.state.player_max_hit_points);
                                "You feel better."
                            }
                            "potion of extra healing" => {
                                self.state.player_hit_points = (self.state.player_hit_points + 8)
                                    .min(self.state.player_max_hit_points);
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
                        vec![InventoryEvent::Used {
                            name: entry.item.name,
                        }]
                    },
                )
            }
            Command::ReadScroll => {
                remove_first_item_by_category(&mut self.state.inventory, ItemCategory::Scroll).map(
                    |entry| {
                        let msg = match entry.item.name {
                            "scroll of protect armor" => "Your armor glows faintly.",
                            "scroll of hold monster" => "The monsters are frozen.",
                            "scroll of enchant weapon" => "Your weapon glows blue.",
                            "scroll of enchant armor" => "Your armor glows silver.",
                            "scroll of identify" => "You can identify this item.",
                            "scroll of teleport" => "You suddenly find yourself somewhere else.",
                            "scroll of sleep" => "You fall asleep.",
                            "scroll of scare monster" => "The monsters flee.",
                            "scroll of remove curse" => {
                                "You feel as if someone is watching over you."
                            }
                            "scroll of create monster" => "You hear a faint cry in the distance.",
                            "scroll of aggravate monster" => {
                                "You hear a high pitched humming noise."
                            }
                            "scroll of magic mapping" => {
                                "You feel a sense of the dungeon around you."
                            }
                            _ => "You read the scroll.",
                        };
                        self.state.last_system_message = Some(msg.to_string());
                        vec![InventoryEvent::Used {
                            name: entry.item.name,
                        }]
                    },
                )
            }
            Command::Eat => {
                remove_first_item_by_category(&mut self.state.inventory, ItemCategory::Food).map(
                    |entry| {
                        self.state.food_remaining = INIT_FOOD;
                        self.state.is_hungry = false;
                        self.state.is_weak = false;
                        self.state.last_system_message =
                            Some("Yum, that tasted good.".to_string());
                        vec![InventoryEvent::Used {
                            name: entry.item.name,
                        }]
                    },
                )
            }
            Command::Zap => remove_first_item_by_category(
                &mut self.state.inventory,
                ItemCategory::Wand,
            )
            .map(|entry| {
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
                vec![InventoryEvent::Used {
                    name: entry.item.name,
                }]
            }),
            Command::Throw => remove_first_item_by_category(
                &mut self.state.inventory,
                ItemCategory::Weapon,
            )
            .map(|entry| {
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

                vec![InventoryEvent::Thrown {
                    name: entry.item.name,
                }]
            }),
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
                    self.state.last_system_message =
                        Some(format!("You found a {trap_name}."));
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

        // Hunger tick — reduce food every 2 turns
        if self.state.turns % 2 == 0 {
            self.state.food_remaining -= 1;
            if self.state.food_remaining <= 0 {
                self.state.player_hit_points = (self.state.player_hit_points - 1).max(0);
                self.state.last_system_message = Some("You are starving!".to_string());
                if self.state.player_hit_points == 0 {
                    self.state.quit_requested = true;
                }
            } else if self.state.food_remaining <= 20 && !self.state.is_weak {
                self.state.is_weak = true;
                self.state.is_hungry = true;
                self.state.last_system_message =
                    Some("You feel weak with hunger.".to_string());
            } else if self.state.food_remaining <= 150 && !self.state.is_hungry {
                self.state.is_hungry = true;
                self.state.last_system_message =
                    Some("You are starting to feel hungry.".to_string());
            }
        }

        if self.state.player_hit_points == 0 {
            self.state.quit_requested = true;
            self.record_high_score(persistence::RunOutcome::Defeated);
            return StepOutcome::Finished;
        }

        let events = tick_monsters(
            &mut self.state.monsters,
            &self.current_level,
            self.state.player_position,
        );

        for event in events {
            match event {
                CombatEvent::MonsterHitPlayer { damage, .. } => {
                    let mitigated_damage = (damage - self.player_armor_bonus()).max(1);
                    self.state.player_hit_points =
                        (self.state.player_hit_points - mitigated_damage).max(0);
                    if self.state.player_hit_points == 0 {
                        self.state.quit_requested = true;
                    }
                }
                CombatEvent::MonsterAppliedEffect { effect, .. } => match effect {
                    StatusEffectEvent::Frozen { turns } => {
                        if self.state.frozen_turns == 0 {
                            self.state.frozen_turns = turns;
                        }
                    }
                    StatusEffectEvent::Held => {}
                    StatusEffectEvent::Stung {
                        max_hit_points_lost,
                    } => {
                        self.state.player_max_hit_points =
                            (self.state.player_max_hit_points - max_hit_points_lost).max(1);
                        self.state.player_hit_points = self
                            .state
                            .player_hit_points
                            .min(self.state.player_max_hit_points);
                    }
                    StatusEffectEvent::LifeDrained { max_hit_points_lost } => {
                        self.state.player_max_hit_points =
                            (self.state.player_max_hit_points - max_hit_points_lost).max(1);
                        self.state.player_hit_points = self
                            .state
                            .player_hit_points
                            .min(self.state.player_max_hit_points);
                    }
                    StatusEffectEvent::GoldStolen => {}
                    StatusEffectEvent::ItemStolen => {}
                    StatusEffectEvent::LevelDropped => {}
                    StatusEffectEvent::ArmorRusted => {}
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
            self.record_high_score(persistence::RunOutcome::Defeated);
            StepOutcome::Finished
        } else {
            StepOutcome::Continue
        }
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
                self.state.pending_direction = Some(direction);
                match self.try_move_player(direction) {
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
                                    self.state.player_hit_points =
                                        (self.state.player_hit_points - 2).max(0);
                                    self.state.player_strength =
                                        (self.state.player_strength - 1).max(1);
                                    self.state.last_system_message = Some(
                                        "A dart hits you for 2 damage and poisons you!"
                                            .to_string(),
                                    );
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
                                self.record_high_score(persistence::RunOutcome::Defeated);
                                return StepOutcome::Finished;
                            }
                        }
                        self.advance_world_turn()
                    }
                    PlayerAction::InventoryChanged | PlayerAction::Blocked => StepOutcome::Continue,
                }
            }
            Command::PickUp
            | Command::Drop
            | Command::Wield
            | Command::WearArmor
            | Command::TakeOffArmor
            | Command::PutOnRing
            | Command::RemoveRing
            | Command::Quaff
            | Command::Zap
            | Command::Throw
            | Command::ReadScroll
            | Command::Eat
            | Command::IdentifyTrap => match self.try_inventory_action(command) {
                PlayerAction::InventoryChanged => self.advance_world_turn(),
                PlayerAction::Blocked
                | PlayerAction::Held
                | PlayerAction::Moved
                | PlayerAction::Attacked => StepOutcome::Continue,
            },
            Command::Noop | Command::Unknown => {
                self.state.pending_direction = None;
                self.state.last_move_blocked = false;
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
        assert_eq!(game.state().player_position, Position::new(3, 18));
        assert!(game.state().inventory.is_empty());
        assert_eq!(game.state().monsters.len(), 1);
        assert_ne!(
            game.state().monsters[0].position,
            game.state().player_position
        );
    }

    #[test]
    fn rest_and_move_advance_turns() {
        let mut game = GameLoop::new(12345);

        assert_eq!(game.step(Command::Rest), StepOutcome::Continue);
        assert_eq!(
            game.step(Command::Move(Direction::Left)),
            StepOutcome::Continue
        );

        assert_eq!(game.state().turns, 2);
        assert_eq!(game.state().pending_direction, Some(Direction::Left));
        assert_eq!(game.state().player_position, Position::new(3, 17));
        assert_eq!(game.state().monsters[0].position, Position::new(3, 14));
        assert!(game.state().last_turn_events.is_empty());
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
        game.state.player_hit_points = 7;
        game.state
            .inventory
            .push(crate::inventory_items::InventoryEntry {
                id: 1,
                item: InventoryItem::healing_potion(),
                equipped_slot: None,
            });

        assert_eq!(game.step(Command::Quaff), StepOutcome::Continue);
        assert_eq!(game.state.player_hit_points, 11);
        assert!(game.state.inventory.is_empty());
    }

    #[test]
    fn identify_trap_marks_nearby_trap() {
        let mut game = GameLoop::new(12345);
        let trap = game.state.trap_positions[0];

        assert!(super::is_adjacent(game.state.player_position, trap));
        assert_eq!(game.step(Command::IdentifyTrap), StepOutcome::Continue);
        assert!(game.state.known_traps.contains(&trap));
    }

    #[test]
    fn move_into_wall_is_blocked_without_consuming_turn() {
        let mut game = GameLoop::new(12345);
        game.state.monsters.clear();

        for _ in 0..6 {
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
            game.step(Command::Move(Direction::UpLeft)),
            StepOutcome::Continue
        );
        assert_eq!(game.state().player_position, Position::new(2, 17));
        assert_eq!(game.state().monsters[0].position, Position::new(2, 13));
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

        assert_eq!(game.step(Command::Wield), StepOutcome::Continue);
        assert_eq!(game.state().turns, 2);
        assert_eq!(
            game.state().inventory[0].equipped_slot,
            Some(EquipmentSlot::Weapon)
        );

        let drop_position = game.state.player_position;
        assert_eq!(game.step(Command::Drop), StepOutcome::Continue);
        assert_eq!(game.state().turns, 3);
        assert!(game.state().inventory.is_empty());
        assert_eq!(game.state().floor_items[0].position, drop_position);
    }

    #[test]
    fn equipped_items_modify_attack_and_armor() {
        let mut game = GameLoop::new(12345);
        game.state
            .inventory
            .push(crate::inventory_items::InventoryEntry {
                id: 1,
                item: InventoryItem::dagger(),
                equipped_slot: Some(EquipmentSlot::Weapon),
            });
        game.state
            .inventory
            .push(crate::inventory_items::InventoryEntry {
                id: 2,
                item: InventoryItem::leather_armor(),
                equipped_slot: Some(EquipmentSlot::Armor),
            });

        game.state.monsters[0].position = Position::new(3, 19);
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
        game.state
            .inventory
            .push(crate::inventory_items::InventoryEntry {
                id: 1,
                item: InventoryItem::leather_armor(),
                equipped_slot: None,
            });
        game.state
            .inventory
            .push(crate::inventory_items::InventoryEntry {
                id: 2,
                item: InventoryItem::protection_ring(),
                equipped_slot: None,
            });

        assert_eq!(game.step(Command::WearArmor), StepOutcome::Continue);
        assert_eq!(
            game.state().inventory[0].equipped_slot,
            Some(EquipmentSlot::Armor)
        );

        assert_eq!(game.step(Command::PutOnRing), StepOutcome::Continue);
        assert_eq!(
            game.state().inventory[1].equipped_slot,
            Some(EquipmentSlot::LeftRing)
        );

        assert_eq!(game.step(Command::RemoveRing), StepOutcome::Continue);
        assert_eq!(game.state().inventory[1].equipped_slot, None);

        assert_eq!(game.step(Command::TakeOffArmor), StepOutcome::Continue);
        assert_eq!(game.state().inventory[0].equipped_slot, None);
    }

    #[test]
    fn moving_into_monster_attacks_instead_of_moving() {
        let mut game = GameLoop::new(12345);
        game.state.monsters = vec![Monster::new(MonsterKind::Kestrel, Position::new(3, 19))];
        game.state.monsters[0].hit_points = 2;

        assert_eq!(
            game.step(Command::Move(Direction::Right)),
            StepOutcome::Continue
        );

        assert_eq!(game.state().player_position, Position::new(3, 18));
        assert_eq!(game.state().turns, 1);
        assert_eq!(game.state().player_hit_points, 11);
        assert_eq!(game.state().monsters[0].hit_points, 1);
        assert_eq!(
            game.state().last_turn_events,
            vec![
                CombatEvent::PlayerHitMonster {
                    monster_kind: game.state().monsters[0].kind,
                    position: Position::new(3, 19),
                    damage: 1,
                    killed: false,
                },
                CombatEvent::MonsterHitPlayer {
                    monster_kind: game.state().monsters[0].kind,
                    position: Position::new(3, 19),
                    damage: 1,
                },
            ]
        );
    }

    #[test]
    fn killing_monster_removes_it_before_counter_attack() {
        let mut game = GameLoop::new(12345);
        game.state.monsters = vec![Monster::new(MonsterKind::Kestrel, Position::new(3, 19))];
        game.state.monsters[0].hit_points = 1;

        assert_eq!(
            game.step(Command::Move(Direction::Right)),
            StepOutcome::Continue
        );

        assert!(game.state().monsters.is_empty());
        assert_eq!(game.state().monsters_defeated, 1);
        assert_eq!(game.state().player_hit_points, 12);
        assert_eq!(
            game.state().last_turn_events,
            vec![CombatEvent::PlayerHitMonster {
                monster_kind: crate::actors::MonsterKind::Kestrel,
                position: Position::new(3, 19),
                damage: 1,
                killed: true,
            }]
        );
    }

    #[test]
    fn hold_effect_blocks_escape_but_consumes_turn() {
        let mut game = GameLoop::new(12345);
        game.state.monsters = vec![Monster::new(
            MonsterKind::VenusFlytrap,
            Position::new(3, 19),
        )];

        assert_eq!(game.step(Command::Rest), StepOutcome::Continue);
        assert!(game.player_is_held());

        let turns_before = game.state.turns;
        assert_eq!(
            game.step(Command::Move(Direction::Left)),
            StepOutcome::Continue
        );

        assert_eq!(game.state.player_position, Position::new(3, 18));
        assert!(game.state.last_move_blocked);
        assert_eq!(game.state.turns, turns_before + 1);
    }

    #[test]
    fn freeze_effect_skips_player_turns() {
        let mut game = GameLoop::new(12345);
        game.state.monsters = vec![Monster::new(MonsterKind::IceMonster, Position::new(3, 19))];

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
        assert_eq!(game.state.player_position, Position::new(3, 18));
        assert_eq!(game.state.turns, turns_before + 1);
        assert_eq!(game.state.frozen_turns, 1);
    }

    #[test]
    fn sting_effect_reduces_player_max_hit_points() {
        let mut game = GameLoop::new(12345);
        game.state.monsters = vec![Monster::new(
            MonsterKind::Rattlesnake,
            Position::new(3, 19),
        )];

        assert_eq!(game.step(Command::Rest), StepOutcome::Continue);

        assert_eq!(game.state.player_max_hit_points, 11);
        assert_eq!(game.state.player_hit_points, 11);
        assert!(game.state.last_turn_events.iter().any(|event| {
            matches!(
                event,
                CombatEvent::MonsterAppliedEffect {
                    effect: StatusEffectEvent::Stung {
                        max_hit_points_lost: 1
                    },
                    ..
                }
            )
        }));
    }
}

