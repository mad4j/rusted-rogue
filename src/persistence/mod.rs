use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::actors::{CombatEvent, Monster, MonsterKind, SpecialHit, StatusEffectEvent};
use crate::core_types::{Position, TileFlags, TrapKind, DCOLS, DROWS};
use crate::game_loop::{Command, Direction, GameLoop, GameState};
use crate::inventory_items::{
    next_avail_ichar, EquipmentSlot, FloorItem, InventoryEntry, InventoryEvent, InventoryItem,
};
use crate::world_gen::{DoorLink, DungeonGrid, GeneratedLevel, Room};

const SAVE_VERSION: u8 = 1;
const SAVE_FILE_NAME: &str = "rusted-rogue-save-v1.json";
const SCORE_VERSION: u8 = 1;
const SCORE_FILE_NAME: &str = "rusted-rogue-scores-v1.json";
const MAX_HIGH_SCORES: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunOutcome {
    Quit,
    Defeated,
}

impl RunOutcome {
    fn as_str(self) -> &'static str {
        match self {
            Self::Quit => "quit",
            Self::Defeated => "defeated",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighScoreEntry {
    pub score: u64,
    pub level: i16,
    pub turns: u64,
    pub monsters_defeated: u64,
    pub outcome: String,
    pub recorded_at_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HighScoreFile {
    version: u8,
    entries: Vec<HighScoreEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveFile {
    version: u8,
    state: GameStateSnapshot,
    level: LevelSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LevelSnapshot {
    rooms: Vec<RoomSnapshot>,
    grid: Vec<Vec<u16>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoomSnapshot {
    top_row: i16,
    bottom_row: i16,
    left_col: i16,
    right_col: i16,
    #[serde(default)]
    slot_index: i16,
    #[serde(default)]
    doors: [DoorLinkSnapshot; 4],
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
struct DoorLinkSnapshot {
    door_row: i16,
    door_col: i16,
    oth_room: i16,
    oth_row: i16,
    oth_col: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GameStateSnapshot {
    level: i16,
    turns: u64,
    #[serde(default)]
    gold: i64,
    quit_requested: bool,
    pending_direction: Option<String>,
    player_position: PositionSnapshot,
    player_hit_points: i16,
    player_max_hit_points: i16,
    player_strength: i16,
    player_max_strength: i16,
    player_exp_points: i64,
    player_exp_level: i16,
    food_remaining: i32,
    is_hungry: bool,
    is_weak: bool,
    frozen_turns: u8,
    #[serde(default)]
    confused_turns: u8,
    monsters_defeated: u64,
    monsters: Vec<MonsterSnapshot>,
    last_turn_events: Vec<CombatEventSnapshot>,
    inventory: Vec<InventoryEntrySnapshot>,
    floor_items: Vec<FloorItemSnapshot>,
    trap_positions: Vec<PositionSnapshot>,
    trap_types: Vec<String>,
    known_traps: Vec<PositionSnapshot>,
    next_item_id: u64,
    last_inventory_events: Vec<InventoryEventSnapshot>,
    last_move_blocked: bool,
    last_system_message: Option<String>,
    #[serde(default)]
    party_counter: i16,
    #[serde(default)]
    explored: Vec<PositionSnapshot>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct PositionSnapshot {
    row: i16,
    col: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonsterSnapshot {
    kind: String,
    position: PositionSnapshot,
    hit_points: i16,
    attack_damage: i16,
    special_hit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum StatusEffectSnapshot {
    Frozen { turns: u8 },
    Held,
    Stung { amount: i16 },
    ArmorRusted,
    GoldStolen,
    ItemStolen,
    LifeDrained { max_hit_points_lost: i16 },
    LevelDropped,
    Confused { turns: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CombatEventSnapshot {
    PlayerHitMonster {
        monster_kind: String,
        position: PositionSnapshot,
        damage: i16,
        killed: bool,
    },
    MonsterHitPlayer {
        monster_kind: String,
        position: PositionSnapshot,
        damage: i16,
    },
    MonsterAppliedEffect {
        monster_kind: String,
        position: PositionSnapshot,
        effect: StatusEffectSnapshot,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InventoryEntrySnapshot {
    id: u64,
    item_name: String,
    equipped_slot: Option<String>,
    /// Pack letter ('a'–'z').  None for saves written before this field was added.
    #[serde(default)]
    ichar: Option<char>,
    /// Stack size.  Defaults to 1 for saves written before this field was added.
    #[serde(default = "quantity_one")]
    quantity: u16,
}

fn quantity_one() -> u16 { 1 }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FloorItemSnapshot {
    item_name: String,
    position: PositionSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum InventoryEventSnapshot {
    PickedUp {
        name: String,
    },
    Dropped {
        name: String,
        position: PositionSnapshot,
    },
    Equipped {
        name: String,
        slot: String,
    },
    Unequipped {
        name: String,
        slot: String,
    },
    Used {
        name: String,
    },
    Thrown {
        name: String,
    },
    PackFull,
}

pub fn default_save_path() -> PathBuf {
    if let Ok(mut current_dir) = std::env::current_dir() {
        current_dir.push("saves");
        current_dir.push(SAVE_FILE_NAME);
        current_dir
    } else {
        let mut temp = std::env::temp_dir();
        temp.push(SAVE_FILE_NAME);
        temp
    }
}

pub fn default_score_path() -> PathBuf {
    if let Ok(mut current_dir) = std::env::current_dir() {
        current_dir.push("saves");
        current_dir.push(SCORE_FILE_NAME);
        current_dir
    } else {
        let mut temp = std::env::temp_dir();
        temp.push(SCORE_FILE_NAME);
        temp
    }
}

pub fn compute_score(game: &GameLoop) -> u64 {
    let level_component = (game.state().level.max(1) as u64) * 1_000;
    let monster_component = game.state().monsters_defeated * 250;
    let turn_component = game.state().turns;
    level_component + monster_component + turn_component
}

pub fn record_score(game: &GameLoop, outcome: RunOutcome) -> io::Result<usize> {
    record_score_to_path(game, outcome, &default_score_path())
}

pub fn record_score_to_path(
    game: &GameLoop,
    outcome: RunOutcome,
    path: &Path,
) -> io::Result<usize> {
    let mut entries = match load_high_scores_from_path(path) {
        Ok(existing) => existing,
        Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error),
    };

    let new_entry = HighScoreEntry {
        score: compute_score(game),
        level: game.state().level,
        turns: game.state().turns,
        monsters_defeated: game.state().monsters_defeated,
        outcome: outcome.as_str().to_string(),
        recorded_at_unix: current_unix_seconds(),
    };

    entries.push(new_entry.clone());
    normalize_high_scores(&mut entries);

    let rank = entries
        .iter()
        .position(|entry| entry == &new_entry)
        .map(|index| index + 1)
        .unwrap_or(0);

    write_high_scores_to_path(path, &entries)?;
    Ok(rank)
}

pub fn load_high_scores() -> io::Result<Vec<HighScoreEntry>> {
    load_high_scores_from_path(&default_score_path())
}

pub fn load_high_scores_from_path(path: &Path) -> io::Result<Vec<HighScoreEntry>> {
    let content = fs::read_to_string(path)?;
    let file: HighScoreFile = serde_json::from_str(&content).map_err(io::Error::other)?;

    if file.version != SCORE_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported score version {} (expected {})",
                file.version, SCORE_VERSION
            ),
        ));
    }

    let mut entries = file.entries;
    normalize_high_scores(&mut entries);
    Ok(entries)
}

fn write_high_scores_to_path(path: &Path, entries: &[HighScoreEntry]) -> io::Result<()> {
    let file = HighScoreFile {
        version: SCORE_VERSION,
        entries: entries.to_vec(),
    };

    let content = serde_json::to_string_pretty(&file).map_err(io::Error::other)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)
}

fn normalize_high_scores(entries: &mut Vec<HighScoreEntry>) {
    entries.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.turns.cmp(&right.turns))
            .then_with(|| right.level.cmp(&left.level))
            .then_with(|| left.recorded_at_unix.cmp(&right.recorded_at_unix))
    });
    entries.truncate(MAX_HIGH_SCORES);
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub fn save_game(game: &GameLoop) -> io::Result<()> {
    save_game_to_path(game, &default_save_path())
}

pub fn load_game() -> io::Result<GameLoop> {
    load_game_from_path(&default_save_path())
}

pub fn save_game_to_path(game: &GameLoop, path: &Path) -> io::Result<()> {
    let save_file = SaveFile {
        version: SAVE_VERSION,
        state: GameStateSnapshot::from_game_state(game.state()),
        level: LevelSnapshot::from_level(game.current_level()),
    };

    let content = serde_json::to_string_pretty(&save_file).map_err(io::Error::other)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)
}

pub fn load_game_from_path(path: &Path) -> io::Result<GameLoop> {
    let content = fs::read_to_string(path)?;
    let save_file: SaveFile = serde_json::from_str(&content).map_err(io::Error::other)?;

    if save_file.version != SAVE_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported save version {} (expected {})",
                save_file.version, SAVE_VERSION
            ),
        ));
    }

    let state = save_file.state.into_game_state()?;
    let level = save_file.level.into_level()?;
    Ok(GameLoop::from_parts(state, level))
}

impl LevelSnapshot {
    fn from_level(level: &GeneratedLevel) -> Self {
        let mut grid = vec![vec![0_u16; DCOLS]; DROWS];

        for row in 0..(DROWS as i16) {
            for col in 0..(DCOLS as i16) {
                let tile = level.grid.get(row, col).unwrap_or(TileFlags::NOTHING);
                grid[row as usize][col as usize] = tile.bits();
            }
        }

        Self {
            rooms: level
                .rooms
                .iter()
                .map(|room| RoomSnapshot {
                    top_row: room.top_row,
                    bottom_row: room.bottom_row,
                    left_col: room.left_col,
                    right_col: room.right_col,
                    slot_index: room.slot_index,
                    doors: room.doors.map(|door| DoorLinkSnapshot {
                        door_row: door.door_row,
                        door_col: door.door_col,
                        oth_room: door.oth_room,
                        oth_row: door.oth_row,
                        oth_col: door.oth_col,
                    }),
                })
                .collect(),
            grid,
        }
    }

    fn into_level(self) -> io::Result<GeneratedLevel> {
        if self.grid.len() != DROWS || self.grid.iter().any(|row| row.len() != DCOLS) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid dungeon grid dimensions in save file",
            ));
        }

        let mut grid = DungeonGrid::new();
        for row in 0..(DROWS as i16) {
            for col in 0..(DCOLS as i16) {
                let bits = self.grid[row as usize][col as usize];
                let tile = TileFlags::from_bits_retain(bits);
                let _ = grid.set(row, col, tile);
            }
        }

        let rooms = self
            .rooms
            .into_iter()
            .map(|room| {
                let doors = room.doors.map(|door| DoorLink {
                    door_row: door.door_row,
                    door_col: door.door_col,
                    oth_room: door.oth_room,
                    oth_row: door.oth_row,
                    oth_col: door.oth_col,
                });
                Room::with_metadata(
                    room.top_row,
                    room.bottom_row,
                    room.left_col,
                    room.right_col,
                    room.slot_index,
                    doors,
                )
            })
            .collect();

        let stairs_position = (0..DROWS as i16)
            .flat_map(|row| (0..DCOLS as i16).map(move |col| (row, col)))
            .find(|&(row, col)| {
                grid.get(row, col)
                    .is_some_and(|f| f.contains(TileFlags::STAIRS))
            })
            .map(|(row, col)| crate::core_types::Position::new(row, col));

        Ok(GeneratedLevel { grid, rooms, stairs_position })
    }
}

impl PositionSnapshot {
    fn from_position(position: Position) -> Self {
        Self {
            row: position.row,
            col: position.col,
        }
    }

    fn into_position(self) -> Position {
        Position::new(self.row, self.col)
    }
}

impl GameStateSnapshot {
    fn from_game_state(state: &GameState) -> Self {
        Self {
            level: state.level,
            turns: state.turns,
            gold: state.gold,
            quit_requested: state.quit_requested,
            pending_direction: state.pending_direction.map(direction_to_string),
            player_position: PositionSnapshot::from_position(state.player_position),
            player_hit_points: state.player_hit_points,
            player_max_hit_points: state.player_max_hit_points,
            player_strength: state.player_strength,
            player_max_strength: state.player_max_strength,
            player_exp_points: state.player_exp_points,
            player_exp_level: state.player_exp_level,
            food_remaining: state.food_remaining,
            is_hungry: state.is_hungry,
            is_weak: state.is_weak,
            frozen_turns: state.frozen_turns,
            confused_turns: state.confused_turns,
            monsters_defeated: state.monsters_defeated,
            monsters: state
                .monsters
                .iter()
                .map(MonsterSnapshot::from_monster)
                .collect(),
            last_turn_events: state
                .last_turn_events
                .iter()
                .map(CombatEventSnapshot::from_event)
                .collect(),
            inventory: state
                .inventory
                .iter()
                .map(InventoryEntrySnapshot::from_entry)
                .collect(),
            floor_items: state
                .floor_items
                .iter()
                .map(FloorItemSnapshot::from_floor_item)
                .collect(),
            trap_positions: state
                .trap_positions
                .iter()
                .map(|p| PositionSnapshot::from_position(*p))
                .collect(),
            trap_types: state
                .trap_types
                .iter()
                .map(|k| trap_kind_to_string(*k).to_string())
                .collect(),
            known_traps: state
                .known_traps
                .iter()
                .map(|p| PositionSnapshot::from_position(*p))
                .collect(),
            next_item_id: state.next_item_id,
            last_inventory_events: state
                .last_inventory_events
                .iter()
                .map(InventoryEventSnapshot::from_event)
                .collect(),
            last_move_blocked: state.last_move_blocked,
            last_system_message: state.last_system_message.clone(),
            party_counter: state.party_counter,
            explored: state
                .explored
                .iter()
                .map(|p| PositionSnapshot::from_position(*p))
                .collect(),
        }
    }

    fn into_game_state(self) -> io::Result<GameState> {
        Ok(GameState {
            level: self.level,
            turns: self.turns,
            gold: self.gold,
            quit_requested: self.quit_requested,
            pending_direction: self
                .pending_direction
                .as_deref()
                .map(direction_from_string)
                .transpose()?,
            player_position: self.player_position.into_position(),
            player_hit_points: self.player_hit_points,
            player_max_hit_points: self.player_max_hit_points,
            player_strength: self.player_strength,
            player_max_strength: self.player_max_strength,
            player_exp_points: self.player_exp_points,
            player_exp_level: self.player_exp_level,
            food_remaining: self.food_remaining,
            is_hungry: self.is_hungry,
            is_weak: self.is_weak,
            frozen_turns: self.frozen_turns,
            confused_turns: self.confused_turns,
            monsters_defeated: self.monsters_defeated,
            monsters: self
                .monsters
                .into_iter()
                .map(MonsterSnapshot::into_monster)
                .collect::<io::Result<Vec<_>>>()?,
            last_turn_events: self
                .last_turn_events
                .into_iter()
                .map(CombatEventSnapshot::into_event)
                .collect::<io::Result<Vec<_>>>()?,
            inventory: {
                let mut inv = self
                    .inventory
                    .into_iter()
                    .map(InventoryEntrySnapshot::into_entry)
                    .collect::<io::Result<Vec<_>>>()?;
                // Reassign pack letters missing in older save files.
                for i in 0..inv.len() {
                    if inv[i].ichar == '\0' {
                        inv[i].ichar = next_avail_ichar(&inv);
                    }
                }
                inv
            },
            floor_items: self
                .floor_items
                .into_iter()
                .map(FloorItemSnapshot::into_floor_item)
                .collect::<io::Result<Vec<_>>>()?,
            trap_positions: self
                .trap_positions
                .into_iter()
                .map(PositionSnapshot::into_position)
                .collect(),
            trap_types: self
                .trap_types
                .iter()
                .map(|s| trap_kind_from_string(s))
                .collect::<io::Result<Vec<_>>>()?,
            known_traps: self
                .known_traps
                .into_iter()
                .map(PositionSnapshot::into_position)
                .collect(),
            next_item_id: self.next_item_id,
            last_inventory_events: self
                .last_inventory_events
                .into_iter()
                .map(InventoryEventSnapshot::into_event)
                .collect::<io::Result<Vec<_>>>()?,
            last_move_blocked: self.last_move_blocked,
            last_system_message: self.last_system_message,
            party_counter: self.party_counter,
            pending_item_action: None,
            explored: self
                .explored
                .into_iter()
                .map(PositionSnapshot::into_position)
                .collect(),
        })
    }
}

impl MonsterSnapshot {
    fn from_monster(monster: &Monster) -> Self {
        Self {
            kind: monster_kind_to_string(monster.kind).to_string(),
            position: PositionSnapshot::from_position(monster.position),
            hit_points: monster.hit_points,
            attack_damage: monster.attack_damage,
            special_hit: monster
                .special_hit
                .map(|hit| special_hit_to_string(hit).to_string()),
        }
    }

    fn into_monster(self) -> io::Result<Monster> {
        Ok(Monster {
            kind: monster_kind_from_string(&self.kind)?,
            position: self.position.into_position(),
            hit_points: self.hit_points,
            attack_damage: self.attack_damage,
            special_hit: self
                .special_hit
                .as_deref()
                .map(special_hit_from_string)
                .transpose()?,
        })
    }
}

impl CombatEventSnapshot {
    fn from_event(event: &CombatEvent) -> Self {
        match event {
            CombatEvent::PlayerHitMonster {
                monster_kind,
                position,
                damage,
                killed,
            } => Self::PlayerHitMonster {
                monster_kind: monster_kind_to_string(*monster_kind).to_string(),
                position: PositionSnapshot::from_position(*position),
                damage: *damage,
                killed: *killed,
            },
            CombatEvent::MonsterHitPlayer {
                monster_kind,
                position,
                damage,
            } => Self::MonsterHitPlayer {
                monster_kind: monster_kind_to_string(*monster_kind).to_string(),
                position: PositionSnapshot::from_position(*position),
                damage: *damage,
            },
            CombatEvent::MonsterAppliedEffect {
                monster_kind,
                position,
                effect,
            } => Self::MonsterAppliedEffect {
                monster_kind: monster_kind_to_string(*monster_kind).to_string(),
                position: PositionSnapshot::from_position(*position),
                effect: match effect {
                    StatusEffectEvent::Frozen { turns } => {
                        StatusEffectSnapshot::Frozen { turns: *turns }
                    }
                    StatusEffectEvent::Held => StatusEffectSnapshot::Held,
                    StatusEffectEvent::Stung { amount } => {
                        StatusEffectSnapshot::Stung { amount: *amount }
                    }
                    StatusEffectEvent::ArmorRusted => StatusEffectSnapshot::ArmorRusted,
                    StatusEffectEvent::GoldStolen => StatusEffectSnapshot::GoldStolen,
                    StatusEffectEvent::ItemStolen => StatusEffectSnapshot::ItemStolen,
                    StatusEffectEvent::LifeDrained { max_hit_points_lost } => {
                        StatusEffectSnapshot::LifeDrained {
                            max_hit_points_lost: *max_hit_points_lost,
                        }
                    }
                    StatusEffectEvent::LevelDropped => StatusEffectSnapshot::LevelDropped,
                    StatusEffectEvent::Confused { turns } => {
                        StatusEffectSnapshot::Confused { turns: *turns }
                    }
                },
            },
        }
    }

    fn into_event(self) -> io::Result<CombatEvent> {
        Ok(match self {
            Self::PlayerHitMonster {
                monster_kind,
                position,
                damage,
                killed,
            } => CombatEvent::PlayerHitMonster {
                monster_kind: monster_kind_from_string(&monster_kind)?,
                position: position.into_position(),
                damage,
                killed,
            },
            Self::MonsterHitPlayer {
                monster_kind,
                position,
                damage,
            } => CombatEvent::MonsterHitPlayer {
                monster_kind: monster_kind_from_string(&monster_kind)?,
                position: position.into_position(),
                damage,
            },
            Self::MonsterAppliedEffect {
                monster_kind,
                position,
                effect,
            } => CombatEvent::MonsterAppliedEffect {
                monster_kind: monster_kind_from_string(&monster_kind)?,
                position: position.into_position(),
                effect: match effect {
                    StatusEffectSnapshot::Frozen { turns } => StatusEffectEvent::Frozen { turns },
                    StatusEffectSnapshot::Held => StatusEffectEvent::Held,
                    StatusEffectSnapshot::Stung { amount } => StatusEffectEvent::Stung { amount },
                    StatusEffectSnapshot::ArmorRusted => StatusEffectEvent::ArmorRusted,
                    StatusEffectSnapshot::GoldStolen => StatusEffectEvent::GoldStolen,
                    StatusEffectSnapshot::ItemStolen => StatusEffectEvent::ItemStolen,
                    StatusEffectSnapshot::LifeDrained { max_hit_points_lost } => {
                        StatusEffectEvent::LifeDrained { max_hit_points_lost }
                    }
                    StatusEffectSnapshot::LevelDropped => StatusEffectEvent::LevelDropped,
                    StatusEffectSnapshot::Confused { turns } => {
                        StatusEffectEvent::Confused { turns }
                    }
                },
            },
        })
    }
}

impl InventoryEntrySnapshot {
    fn from_entry(entry: &InventoryEntry) -> Self {
        Self {
            id: entry.id,
            item_name: entry.item.name.to_string(),
            equipped_slot: entry
                .equipped_slot
                .map(|slot| equipment_slot_to_string(slot).to_string()),
            ichar: Some(entry.ichar),
            quantity: entry.quantity,
        }
    }

    fn into_entry(self) -> io::Result<InventoryEntry> {
        let item = InventoryItem::from_name(&self.item_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown item name in save: {}", self.item_name),
            )
        })?;

        Ok(InventoryEntry {
            id: self.id,
            item,
            equipped_slot: self
                .equipped_slot
                .as_deref()
                .map(equipment_slot_from_string)
                .transpose()?,
            // '\0' signals "needs reassignment" done after all entries are loaded.
            ichar: self.ichar.unwrap_or('\0'),
            quantity: self.quantity,
        })
    }
}

impl FloorItemSnapshot {
    fn from_floor_item(item: &FloorItem) -> Self {
        Self {
            item_name: item.item.name.to_string(),
            position: PositionSnapshot::from_position(item.position),
        }
    }

    fn into_floor_item(self) -> io::Result<FloorItem> {
        let item = InventoryItem::from_name(&self.item_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown floor item name in save: {}", self.item_name),
            )
        })?;

        Ok(FloorItem {
            item,
            position: self.position.into_position(),
        })
    }
}

impl InventoryEventSnapshot {
    fn from_event(event: &InventoryEvent) -> Self {
        match event {
            InventoryEvent::PickedUp { name } => Self::PickedUp {
                name: (*name).to_string(),
            },
            InventoryEvent::Dropped { name, position } => Self::Dropped {
                name: (*name).to_string(),
                position: PositionSnapshot::from_position(*position),
            },
            InventoryEvent::Equipped { name, slot } => Self::Equipped {
                name: (*name).to_string(),
                slot: equipment_slot_to_string(*slot).to_string(),
            },
            InventoryEvent::Unequipped { name, slot } => Self::Unequipped {
                name: (*name).to_string(),
                slot: equipment_slot_to_string(*slot).to_string(),
            },
            InventoryEvent::Used { name } => Self::Used {
                name: (*name).to_string(),
            },
            InventoryEvent::Thrown { name } => Self::Thrown {
                name: (*name).to_string(),
            },
            InventoryEvent::PackFull => Self::PackFull,
        }
    }

    fn into_event(self) -> io::Result<InventoryEvent> {
        Ok(match self {
            Self::PickedUp { name } => InventoryEvent::PickedUp {
                name: item_name_to_static(&name)?,
            },
            Self::Dropped { name, position } => InventoryEvent::Dropped {
                name: item_name_to_static(&name)?,
                position: position.into_position(),
            },
            Self::Equipped { name, slot } => InventoryEvent::Equipped {
                name: item_name_to_static(&name)?,
                slot: equipment_slot_from_string(&slot)?,
            },
            Self::Unequipped { name, slot } => InventoryEvent::Unequipped {
                name: item_name_to_static(&name)?,
                slot: equipment_slot_from_string(&slot)?,
            },
            Self::Used { name } => InventoryEvent::Used {
                name: item_name_to_static(&name)?,
            },
            Self::Thrown { name } => InventoryEvent::Thrown {
                name: item_name_to_static(&name)?,
            },
            Self::PackFull => InventoryEvent::PackFull,
        })
    }
}

fn direction_to_string(direction: Direction) -> String {
    match direction {
        Direction::Left => "Left",
        Direction::Right => "Right",
        Direction::Up => "Up",
        Direction::Down => "Down",
        Direction::UpLeft => "UpLeft",
        Direction::UpRight => "UpRight",
        Direction::DownLeft => "DownLeft",
        Direction::DownRight => "DownRight",
    }
    .to_string()
}

fn direction_from_string(direction: &str) -> io::Result<Direction> {
    match direction {
        "Left" => Ok(Direction::Left),
        "Right" => Ok(Direction::Right),
        "Up" => Ok(Direction::Up),
        "Down" => Ok(Direction::Down),
        "UpLeft" => Ok(Direction::UpLeft),
        "UpRight" => Ok(Direction::UpRight),
        "DownLeft" => Ok(Direction::DownLeft),
        "DownRight" => Ok(Direction::DownRight),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown direction in save: {direction}"),
        )),
    }
}

fn monster_kind_to_string(kind: MonsterKind) -> &'static str {
    match kind {
        MonsterKind::Aquator => "Aquator",
        MonsterKind::Bat => "Bat",
        MonsterKind::Centaur => "Centaur",
        MonsterKind::Dragon => "Dragon",
        MonsterKind::Emu => "Emu",
        MonsterKind::VenusFlytrap => "VenusFlytrap",
        MonsterKind::Griffin => "Griffin",
        MonsterKind::Hobgoblin => "Hobgoblin",
        MonsterKind::IceMonster => "IceMonster",
        MonsterKind::Jabberwock => "Jabberwock",
        MonsterKind::Kestrel => "Kestrel",
        MonsterKind::Leprechaun => "Leprechaun",
        MonsterKind::Medusa => "Medusa",
        MonsterKind::Nymph => "Nymph",
        MonsterKind::Orc => "Orc",
        MonsterKind::Phantom => "Phantom",
        MonsterKind::Quagga => "Quagga",
        MonsterKind::Rattlesnake => "Rattlesnake",
        MonsterKind::Snake => "Snake",
        MonsterKind::Troll => "Troll",
        MonsterKind::BlackUnicorn => "BlackUnicorn",
        MonsterKind::Vampire => "Vampire",
        MonsterKind::Wraith => "Wraith",
        MonsterKind::Xeroc => "Xeroc",
        MonsterKind::Yeti => "Yeti",
        MonsterKind::Zombie => "Zombie",
    }
}

fn monster_kind_from_string(kind: &str) -> io::Result<MonsterKind> {
    match kind {
        "Aquator" => Ok(MonsterKind::Aquator),
        "Bat" => Ok(MonsterKind::Bat),
        "Centaur" => Ok(MonsterKind::Centaur),
        "Dragon" => Ok(MonsterKind::Dragon),
        "Emu" => Ok(MonsterKind::Emu),
        "VenusFlytrap" => Ok(MonsterKind::VenusFlytrap),
        "Griffin" => Ok(MonsterKind::Griffin),
        "Hobgoblin" => Ok(MonsterKind::Hobgoblin),
        "IceMonster" => Ok(MonsterKind::IceMonster),
        "Jabberwock" => Ok(MonsterKind::Jabberwock),
        "Kestrel" => Ok(MonsterKind::Kestrel),
        "Leprechaun" => Ok(MonsterKind::Leprechaun),
        "Medusa" => Ok(MonsterKind::Medusa),
        "Nymph" => Ok(MonsterKind::Nymph),
        "Orc" => Ok(MonsterKind::Orc),
        "Phantom" => Ok(MonsterKind::Phantom),
        "Quagga" => Ok(MonsterKind::Quagga),
        "Rattlesnake" => Ok(MonsterKind::Rattlesnake),
        "Snake" => Ok(MonsterKind::Snake),
        "Troll" => Ok(MonsterKind::Troll),
        "BlackUnicorn" => Ok(MonsterKind::BlackUnicorn),
        "Vampire" => Ok(MonsterKind::Vampire),
        "Wraith" => Ok(MonsterKind::Wraith),
        "Xeroc" => Ok(MonsterKind::Xeroc),
        "Yeti" => Ok(MonsterKind::Yeti),
        "Zombie" => Ok(MonsterKind::Zombie),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown monster kind in save: {kind}"),
        )),
    }
}

fn special_hit_to_string(hit: SpecialHit) -> &'static str {
    match hit {
        SpecialHit::Freeze => "Freeze",
        SpecialHit::Hold => "Hold",
        SpecialHit::Sting => "Sting",
        SpecialHit::Rusts => "Rusts",
        SpecialHit::StealsGold => "StealsGold",
        SpecialHit::StealsItem => "StealsItem",
        SpecialHit::DrainsLife => "DrainsLife",
        SpecialHit::DropsLevel => "DropsLevel",
        SpecialHit::Confuse => "Confuse",
        SpecialHit::Flames => "Flames",
    }
}

fn special_hit_from_string(hit: &str) -> io::Result<SpecialHit> {
    match hit {
        "Freeze" => Ok(SpecialHit::Freeze),
        "Hold" => Ok(SpecialHit::Hold),
        "Sting" => Ok(SpecialHit::Sting),
        "Rusts" => Ok(SpecialHit::Rusts),
        "StealsGold" => Ok(SpecialHit::StealsGold),
        "StealsItem" => Ok(SpecialHit::StealsItem),
        "DrainsLife" => Ok(SpecialHit::DrainsLife),
        "DropsLevel" => Ok(SpecialHit::DropsLevel),
        "Confuse" => Ok(SpecialHit::Confuse),
        "Flames" => Ok(SpecialHit::Flames),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown special hit in save: {hit}"),
        )),
    }
}

fn trap_kind_to_string(kind: TrapKind) -> &'static str {
    match kind {
        TrapKind::TrapDoor => "trap_door",
        TrapKind::BearTrap => "bear_trap",
        TrapKind::TeleTrap => "tele_trap",
        TrapKind::DartTrap => "dart_trap",
        TrapKind::SleepingGasTrap => "sleeping_gas_trap",
        TrapKind::RustTrap => "rust_trap",
    }
}

fn trap_kind_from_string(kind: &str) -> io::Result<TrapKind> {
    match kind {
        "trap_door" => Ok(TrapKind::TrapDoor),
        "bear_trap" => Ok(TrapKind::BearTrap),
        "tele_trap" => Ok(TrapKind::TeleTrap),
        "dart_trap" => Ok(TrapKind::DartTrap),
        "sleeping_gas_trap" => Ok(TrapKind::SleepingGasTrap),
        "rust_trap" => Ok(TrapKind::RustTrap),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown trap kind in save: {kind}"),
        )),
    }
}

fn equipment_slot_to_string(slot: EquipmentSlot) -> &'static str {
    match slot {
        EquipmentSlot::Weapon => "Weapon",
        EquipmentSlot::Armor => "Armor",
        EquipmentSlot::LeftRing => "LeftRing",
        EquipmentSlot::RightRing => "RightRing",
    }
}

fn equipment_slot_from_string(slot: &str) -> io::Result<EquipmentSlot> {
    match slot {
        "Weapon" => Ok(EquipmentSlot::Weapon),
        "Armor" => Ok(EquipmentSlot::Armor),
        "LeftRing" => Ok(EquipmentSlot::LeftRing),
        "RightRing" => Ok(EquipmentSlot::RightRing),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown equipment slot in save: {slot}"),
        )),
    }
}

fn item_name_to_static(name: &str) -> io::Result<&'static str> {
    InventoryItem::from_name(name)
        .map(|item| item.name)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown item name in save: {name}"),
            )
        })
}

pub fn save() {
    let _ = Command::Save;
}

#[cfg(test)]
mod tests {
    use super::{
        load_game_from_path, load_high_scores_from_path, record_score_to_path, save_game_to_path,
        RunOutcome,
    };
    use crate::game_loop::{Command, GameLoop};

    #[test]
    fn save_load_round_trip_preserves_state() {
        let mut game = GameLoop::new(12345);
        let _ = game.step(Command::Rest);
        let _ = game.step(Command::Move(crate::game_loop::Direction::Left));

        let save_path = std::env::temp_dir().join("rusted-rogue-roundtrip-test.json");

        save_game_to_path(&game, &save_path).expect("save should succeed");
        let loaded = load_game_from_path(&save_path).expect("load should succeed");

        assert_eq!(game.state(), loaded.state());
        assert_eq!(game.current_level().rooms, loaded.current_level().rooms);
        assert_eq!(
            game.current_level().grid.get(18, 12),
            loaded.current_level().grid.get(18, 12)
        );

        let _ = std::fs::remove_file(save_path);
    }

    #[test]
    fn high_scores_are_written_read_and_sorted() {
        let score_path = std::env::temp_dir().join("rusted-rogue-highscores-test.json");
        let _ = std::fs::remove_file(&score_path);

        let mut game_a = GameLoop::new(12345);
        game_a.state_mut().level = 3;
        game_a.state_mut().monsters_defeated = 1;
        game_a.state_mut().turns = 10;

        let mut game_b = GameLoop::new(12345);
        game_b.state_mut().level = 4;
        game_b.state_mut().monsters_defeated = 5;
        game_b.state_mut().turns = 5;

        let rank_a = record_score_to_path(&game_a, RunOutcome::Quit, &score_path)
            .expect("first score write should succeed");
        let rank_b = record_score_to_path(&game_b, RunOutcome::Defeated, &score_path)
            .expect("second score write should succeed");

        let scores = load_high_scores_from_path(&score_path).expect("scores should be readable");

        assert_eq!(scores.len(), 2);
        assert!(scores[0].score >= scores[1].score);
        assert_eq!(scores[0].outcome, "defeated");
        assert!(rank_a >= 1);
        assert_eq!(rank_b, 1);

        let _ = std::fs::remove_file(score_path);
    }
}
