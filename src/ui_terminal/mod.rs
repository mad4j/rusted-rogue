use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use doryen_rs::{App, AppOptions, DoryenApi, Engine, InputApi, TextAlign, UpdateEvent};

use crate::actors::{CombatEvent, MonsterKind, StatusEffectEvent};
use crate::core_types::{Position, TileFlags, DCOLS, DROWS};
use crate::game_loop::{Command, Direction, GameLoop, StepOutcome};
use crate::inventory_items::{InventoryEvent, ItemCategory};

// Console dimensions: 60 cols x 35 rows (32 map + 3 UI lines)
const UI_ROWS: u32 = 3;
// Pixel size of each font glyph in terminal_8x8.png
const FONT_W: u32 = 8;
const FONT_H: u32 = 8;
// Scale factor for the window (2x makes it easier to read)
const SCALE: u32 = 2;

pub fn run(game: GameLoop) {
    let con_w = DCOLS as u32;
    let con_h = DROWS as u32 + UI_ROWS;
    let font_path = resolve_font_path();
    let mut app = App::new(AppOptions {
        console_width: con_w,
        console_height: con_h,
        screen_width: con_w * FONT_W * SCALE,
        screen_height: con_h * FONT_H * SCALE,
        window_title: "Rusted Rogue".to_string(),
        // Use an absolute path so native doryen-rs does not prepend "www/".
        font_path,
        vsync: true,
        fullscreen: false,
        show_cursor: false,
        resizable: false,
        intercept_close_request: false,
        max_fps: 60,
    });
    app.set_engine(Box::new(RogueEngine { game }));
    app.run();
}

fn resolve_font_path() -> String {
    if let Ok(path) = std::env::var("RUSTED_ROGUE_FONT") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return candidate.to_string_lossy().into_owned();
        }
    }

    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("terminal_8x8.png"));
        candidates.push(cwd.join("www").join("terminal_8x8.png"));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest_dir.join("terminal_8x8.png"));
    candidates.push(manifest_dir.join("www").join("terminal_8x8.png"));

    if let Some(path) = candidates.into_iter().find(|p| p.is_file()) {
        return path.to_string_lossy().into_owned();
    }

    panic!(
        "Missing terminal font. Place terminal_8x8.png in project root or www/, or set RUSTED_ROGUE_FONT to an absolute file path."
    );
}

struct RogueEngine {
    game: GameLoop,
}

impl Engine for RogueEngine {
    fn update(&mut self, api: &mut dyn DoryenApi) -> Option<UpdateEvent> {
        let input = api.input();
        if input.close_requested() || self.game.state().quit_requested {
            return Some(UpdateEvent::Exit);
        }
        if let Some(cmd) = read_command(input) {
            let outcome = self.game.step(cmd);
            if outcome == StepOutcome::Finished {
                return Some(UpdateEvent::Exit);
            }
        }
        None
    }

    fn render(&mut self, api: &mut dyn DoryenApi) {
        let con = api.con();
        con.clear(None, Some((0, 0, 0, 255)), Some(b' ' as u16));

        let lookups = RenderLookups::from_game(&self.game);

        for row in 0..(DROWS as i32) {
            for col in 0..(DCOLS as i32) {
                let ch =
                    render_cell(&self.game, Position::new(row as i16, col as i16), &lookups);
                con.ascii(col, row, ch as u16);
                con.fore(col, row, cell_color(ch));
            }
        }

        let status = render_status(&self.game);
        con.print(
            0,
            DROWS as i32,
            &status,
            TextAlign::Left,
            Some((255, 255, 100, 255)),
            None,
        );

        let message = render_last_message(&self.game);
        con.print(
            0,
            DROWS as i32 + 1,
            &message,
            TextAlign::Left,
            Some((255, 200, 150, 255)),
            None,
        );

        con.print(
            0,
            DROWS as i32 + 2,
            "hjkl yubn/arrows move  . rest  > stairs  , pick  d drop  w wield  W wear  T off  P/R ring  q quaff  z zap  t throw  r read  e eat  ^ trap  Q quit",
            TextAlign::Left,
            Some((120, 120, 120, 255)),
            None,
        );
    }
}

fn read_command(input: &mut dyn InputApi) -> Option<Command> {
    if input.key_pressed("Escape") {
        return Some(Command::Quit);
    }
    if input.key_pressed("ArrowLeft") {
        return Some(Command::Move(Direction::Left));
    }
    if input.key_pressed("ArrowRight") {
        return Some(Command::Move(Direction::Right));
    }
    if input.key_pressed("ArrowUp") {
        return Some(Command::Move(Direction::Up));
    }
    if input.key_pressed("ArrowDown") {
        return Some(Command::Move(Direction::Down));
    }
    // text() returns characters typed this frame (respects shift for uppercase)
    let text = input.text();
    if let Some(ch) = text.chars().next() {
        let cmd = GameLoop::parse_command(ch);
        if cmd != Command::Unknown {
            return Some(cmd);
        }
    }
    None
}

fn cell_color(ch: char) -> (u8, u8, u8, u8) {
    match ch {
        '@' => (255, 255, 255, 255),          // player: white
        'A'..='Z' | 'a'..='z' => (220, 80, 80, 255), // monsters: red
        ')' | ']' => (100, 200, 255, 255),    // weapons / armor: cyan
        '=' => (255, 210, 60, 255),           // rings: gold
        '!' => (200, 100, 255, 255),          // potions: purple
        '/' => (100, 255, 200, 255),          // wands: teal
        '?' => (230, 230, 100, 255),          // scrolls: yellow
        '%' => (100, 200, 100, 255),          // food: green
        '-' | '|' => (160, 160, 160, 255),   // walls: grey
        '.' => (70, 70, 90, 255),             // floor: dark blue-grey
        '#' => (110, 80, 50, 255),            // tunnel: brown
        '+' => (180, 130, 60, 255),           // door: tan
        '>' => (255, 210, 50, 255),           // stairs: gold
        '^' => (255, 80, 80, 255),            // trap: bright red
        _ => (180, 180, 180, 255),
    }
}

fn render_cell(game: &GameLoop, position: Position, lookups: &RenderLookups) -> char {
    if game.state().player_position == position {
        return '@';
    }

    if !game.state().explored.contains(&position) {
        return ' ';
    }

    if let Some(monster_char) = lookups.monsters.get(&position) {
        return *monster_char;
    }

    if let Some(item_char) = lookups.floor_items.get(&position) {
        return *item_char;
    }

    if lookups.known_traps.contains(&position) {
        return '^';
    }

    game.current_level()
        .grid
        .get(position.row, position.col)
        .map(render_tile)
        .unwrap_or(' ')
}

struct RenderLookups {
    monsters: HashMap<Position, char>,
    floor_items: HashMap<Position, char>,
    known_traps: HashSet<Position>,
}

impl RenderLookups {
    fn from_game(game: &GameLoop) -> Self {
        let monsters = game
            .state()
            .monsters
            .iter()
            .map(|monster| (monster.position, monster.display_char()))
            .collect();

        let floor_items = game
            .state()
            .floor_items
            .iter()
            .map(|floor_item| {
                let ch = match floor_item.item.category {
                    ItemCategory::Weapon => ')',
                    ItemCategory::Armor => ']',
                    ItemCategory::Ring => '=',
                    ItemCategory::Potion => '!',
                    ItemCategory::Wand => '/',
                    ItemCategory::Scroll => '?',
                    ItemCategory::Food => '%',
                };
                (floor_item.position, ch)
            })
            .collect();

        let known_traps = game.state().known_traps.iter().copied().collect();

        Self {
            monsters,
            floor_items,
            known_traps,
        }
    }
}

fn render_tile(tile: TileFlags) -> char {
    if tile.contains(TileFlags::TRAP) {
        '^'
    } else if tile.contains(TileFlags::STAIRS) {
        '>'
    } else if tile.contains(TileFlags::DOOR) {
        '+'
    } else if tile.contains(TileFlags::TUNNEL) {
        '#'
    } else if tile.contains(TileFlags::FLOOR) {
        '.'
    } else if tile.contains(TileFlags::HORWALL) {
        '-'
    } else if tile.contains(TileFlags::VERTWALL) {
        '|'
    } else {
        ' '
    }
}

fn render_status(game: &GameLoop) -> String {
    let hunger = if game.state().is_weak {
        " [WEAK]"
    } else if game.state().is_hungry {
        " [HUNGRY]"
    } else {
        ""
    };
    format!(
        "Level:{} Exp:{}({}) HP:{}/{} Str:{}{} Inv:{} Turns:{}",
        game.state().level,
        game.state().player_exp_points,
        game.state().player_exp_level,
        game.state().player_hit_points,
        game.state().player_max_hit_points,
        game.state().player_strength,
        hunger,
        game.state().inventory.len(),
        game.state().turns,
    )
}

fn render_last_message(game: &GameLoop) -> String {
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

fn equipment_slot_name(slot: crate::inventory_items::EquipmentSlot) -> &'static str {
    match slot {
        crate::inventory_items::EquipmentSlot::Weapon => "weapon hand",
        crate::inventory_items::EquipmentSlot::Armor => "armor slot",
        crate::inventory_items::EquipmentSlot::LeftRing => "left hand",
        crate::inventory_items::EquipmentSlot::RightRing => "right hand",
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

#[cfg(test)]
mod tests {
    use super::{render_cell, RenderLookups};
    use crate::core_types::Position;
    use crate::game_loop::GameLoop;
    use crate::inventory_items::{FloorItem, InventoryItem};

    #[test]
    fn rendering_prioritizes_player_monster_and_floor_items() {
        let mut game = GameLoop::new(12345);
        game.state_mut().floor_items.clear();
        let player = game.state().player_position;
        let monster = game.state().monsters[0].position;
        let item_position = Position::new(player.row, player.col + 1);

        game.state_mut().floor_items.push(FloorItem {
            item: InventoryItem::dagger(),
            position: item_position,
        });

        let lookups = RenderLookups::from_game(&game);

        assert_eq!(render_cell(&game, player, &lookups), '@');
        assert_eq!(
            render_cell(&game, monster, &lookups),
            game.state().monsters[0].display_char()
        );
        assert_eq!(render_cell(&game, item_position, &lookups), ')');
    }
}
