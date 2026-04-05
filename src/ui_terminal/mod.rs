use std::collections::{HashMap, HashSet};

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::widget::canvas;
use iced::widget::image as img_widget;
use iced::{Color, ContentFit, Element, Font, Length, Point, Size, Subscription, Task, Theme};

use crate::actors::{CombatEvent, MonsterKind, StatusEffectEvent};
use crate::core_types::{Position, TileFlags, DCOLS, DROWS};
use crate::game_loop::{Command, Direction, GameLoop, StepOutcome};
use crate::inventory_items::{InventoryEvent, ItemCategory};

// Splash screen PNG embedded at compile time
const SPLASH_BYTES: &[u8] = include_bytes!("../../resources/splash.png");

// Pixel width/height of each console cell (8-pixel glyph × 2× scale)
const CELL_W: f32 = 16.0;
const CELL_H: f32 = 16.0;
// Font size used to render each glyph inside a cell
const FONT_SIZE: f32 = 14.0;
// Extra rows below the map for status and message lines
const UI_ROWS: usize = 3;

pub fn run(game: GameLoop) {
    let win_w = DCOLS as f32 * CELL_W;
    let win_h = (DROWS + UI_ROWS) as f32 * CELL_H;

    iced::application("Rusted Rogue", RogueApp::update, RogueApp::view)
        .subscription(RogueApp::subscription)
        .window(iced::window::Settings {
            size: Size::new(win_w, win_h),
            resizable: false,
            ..Default::default()
        })
        .run_with(move || {
            let splash_handle = img_widget::Handle::from_bytes(SPLASH_BYTES);
            (RogueApp { game, show_help: false, help_page: 0, screen: Screen::Splash, splash_handle }, Task::none())
        })
        .unwrap();
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

enum Screen {
    Splash,
    Game,
}

struct RogueApp {
    game: GameLoop,
    show_help: bool,
    help_page: usize,
    screen: Screen,
    splash_handle: img_widget::Handle,
}

#[derive(Debug, Clone)]
enum Message {
    KeyPressed(Key, Modifiers),
}

impl RogueApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        let Message::KeyPressed(key, _modifiers) = message;

        if matches!(self.screen, Screen::Splash) {
            self.screen = Screen::Game;
            return Task::none();
        }

        if self.game.state().quit_requested {
            return iced::exit();
        }

        if self.show_help {
            match &key {
                Key::Named(Named::ArrowLeft) => {
                    self.help_page = self.help_page.saturating_sub(1);
                }
                Key::Named(Named::ArrowRight) => {
                    if self.help_page + 1 < HELP_PAGES.len() {
                        self.help_page += 1;
                    }
                }
                _ => {
                    self.show_help = false;
                }
            }
            return Task::none();
        }

        if let Key::Character(s) = &key {
            if s.as_str() == "?" {
                self.show_help = true;
                self.help_page = 0;
                return Task::none();
            }
        }

        if let Some(cmd) = key_to_command(&key) {
            let outcome = self.game.step(cmd);
            if outcome == StepOutcome::Finished {
                return iced::exit();
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        match self.screen {
            Screen::Splash => img_widget::Image::new(self.splash_handle.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Contain)
                .into(),
            Screen::Game => canvas::Canvas::new(GameCanvas {
                game: &self.game,
                show_help: self.show_help,
                help_page: self.help_page,
            })
            .width(Length::Fixed(DCOLS as f32 * CELL_W))
            .height(Length::Fixed((DROWS + UI_ROWS) as f32 * CELL_H))
            .into(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::event::listen_with(|event, status, _window| {
            if status == iced::event::Status::Captured {
                return None;
            }
            if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                modified_key,
                modifiers,
                ..
            }) = event
            {
                Some(Message::KeyPressed(modified_key, modifiers))
            } else {
                None
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Canvas renderer
// ---------------------------------------------------------------------------

struct GameCanvas<'a> {
    game: &'a GameLoop,
    show_help: bool,
    help_page: usize,
}

impl<'a> canvas::Program<Message> for GameCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<iced::Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        frame.fill(
            &canvas::Path::rectangle(Point::ORIGIN, bounds.size()),
            Color::BLACK,
        );

        if self.show_help {
            render_help_page(&mut frame, self.help_page);
        } else {
            render_game(&mut frame, self.game);
        }

        vec![frame.into_geometry()]
    }
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

fn cell_text(content: impl Into<String>, col: usize, row: usize, color: Color) -> canvas::Text {
    canvas::Text {
        content: content.into(),
        position: Point::new(col as f32 * CELL_W, row as f32 * CELL_H),
        color,
        size: iced::Pixels(FONT_SIZE),
        line_height: iced::widget::text::LineHeight::Absolute(iced::Pixels(CELL_H)),
        font: Font::MONOSPACE,
        horizontal_alignment: iced::alignment::Horizontal::Left,
        vertical_alignment: iced::alignment::Vertical::Top,
        shaping: iced::widget::text::Shaping::Basic,
    }
}

fn render_game(frame: &mut canvas::Frame, game: &GameLoop) {
    let lookups = RenderLookups::from_game(game);

    for row in 0..DROWS {
        for col in 0..DCOLS {
            let ch = render_cell(game, Position::new(row as i16, col as i16), &lookups);
            let color = cell_color(ch);
            frame.fill_text(cell_text(ch.to_string(), col, row, color));
        }
    }

    let status = render_status(game);
    frame.fill_text(cell_text(status, 0, DROWS, Color::from_rgb(1.0, 1.0, 0.39)));

    let message = render_last_message(game);
    frame.fill_text(cell_text(message, 0, DROWS + 1, Color::from_rgb(1.0, 0.78, 0.59)));
}

fn render_help_page(frame: &mut canvas::Frame, page: usize) {
    const GOLD:   Color = Color { r: 1.0,  g: 0.78, b: 0.20, a: 1.0 };
    const CYAN:   Color = Color { r: 0.39, g: 0.86, b: 1.0,  a: 1.0 };
    const YELLOW: Color = Color { r: 1.0,  g: 0.86, b: 0.31, a: 1.0 };
    const WHITE:  Color = Color { r: 0.86, g: 0.86, b: 0.86, a: 1.0 };
    const DIM:    Color = Color { r: 0.43, g: 0.43, b: 0.43, a: 1.0 };

    let total = HELP_PAGES.len();

    frame.fill_text(cell_text("RUSTED ROGUE  -  KEY BINDINGS", DCOLS / 2 - 14, 0, GOLD));
    let indicator = format!("-- page {} of {} --", page + 1, total);
    frame.fill_text(cell_text(indicator, DCOLS / 2 - 9, 1, DIM));

    for (i, line) in HELP_PAGES[page].iter().enumerate() {
        let row = i + 3;
        match line {
            HelpLine::Section(text) => {
                frame.fill_text(cell_text(*text, 2, row, CYAN));
            }
            HelpLine::Binding(key, desc) => {
                frame.fill_text(cell_text(*key, 4, row, YELLOW));
                frame.fill_text(cell_text(*desc, 26, row, WHITE));
            }
            HelpLine::Empty => {}
        }
    }

    let nav = match (page == 0, page + 1 == total) {
        (_, true) => "<- ArrowLeft: prev page   |   any other key: close",
        (true, _) => "any other key: close   |   ArrowRight: next page ->",
        _ => "<- ArrowLeft: prev   |   ArrowRight: next ->   |   any key: close",
    };
    frame.fill_text(cell_text(nav, 2, DROWS + UI_ROWS - 1, DIM));
}

// ---------------------------------------------------------------------------
// Input mapping
// ---------------------------------------------------------------------------

fn key_to_command(key: &Key) -> Option<Command> {
    match key {
        Key::Named(Named::Escape) => Some(Command::Quit),
        Key::Named(Named::ArrowLeft) => Some(Command::Move(Direction::Left)),
        Key::Named(Named::ArrowRight) => Some(Command::Move(Direction::Right)),
        Key::Named(Named::ArrowUp) => Some(Command::Move(Direction::Up)),
        Key::Named(Named::ArrowDown) => Some(Command::Move(Direction::Down)),
        Key::Character(s) => {
            if let Some(ch) = s.chars().next() {
                let cmd = GameLoop::parse_command(ch);
                if cmd != Command::Unknown {
                    return Some(cmd);
                }
            }
            None
        }
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum HelpLine {
    Section(&'static str),
    Binding(&'static str, &'static str),
    Empty,
}

const HELP_PAGE_1: &[HelpLine] = &[
    HelpLine::Section("Movement"),
    HelpLine::Binding("h / ArrowLeft",  "move left"),
    HelpLine::Binding("j / ArrowDown",  "move down"),
    HelpLine::Binding("k / ArrowUp",    "move up"),
    HelpLine::Binding("l / ArrowRight", "move right"),
    HelpLine::Empty,
    HelpLine::Section("Diagonal Movement"),
    HelpLine::Binding("y", "move up-left"),
    HelpLine::Binding("u", "move up-right"),
    HelpLine::Binding("b", "move down-left"),
    HelpLine::Binding("n", "move down-right"),
    HelpLine::Empty,
    HelpLine::Section("Running  (uppercase)"),
    HelpLine::Binding("H / J / K / L", "run in cardinal direction"),
    HelpLine::Binding("Y / U / B / N", "run in diagonal direction"),
];

const HELP_PAGE_2: &[HelpLine] = &[
    HelpLine::Section("Items"),
    HelpLine::Binding(",", "pick up item"),
    HelpLine::Binding("d", "drop item"),
    HelpLine::Binding("e", "eat food"),
    HelpLine::Binding("q", "quaff potion"),
    HelpLine::Binding("r", "read scroll"),
    HelpLine::Binding("z", "zap wand"),
    HelpLine::Binding("t", "throw item"),
    HelpLine::Empty,
    HelpLine::Section("Equipment"),
    HelpLine::Binding("w", "wield weapon"),
    HelpLine::Binding("W", "wear armor"),
    HelpLine::Binding("T", "take off armor"),
    HelpLine::Binding("P", "put on ring"),
    HelpLine::Binding("R", "remove ring"),
    HelpLine::Empty,
    HelpLine::Section("Other"),
    HelpLine::Binding(".", "rest one turn"),
    HelpLine::Binding(">", "descend stairs"),
    HelpLine::Binding("^", "identify trap"),
    HelpLine::Binding("?", "show this help screen"),
    HelpLine::Empty,
    HelpLine::Section("Game"),
    HelpLine::Binding("S", "save game"),
    HelpLine::Binding("L", "load game"),
    HelpLine::Binding("Q", "quit"),
];

const HELP_PAGES: &[&[HelpLine]] = &[HELP_PAGE_1, HELP_PAGE_2];

fn cell_color(ch: char) -> Color {
    match ch {
        '@' => Color::WHITE,
        'A'..='Z' | 'a'..='z' => Color::from_rgb(0.86, 0.31, 0.31),
        ')' | ']' => Color::from_rgb(0.39, 0.78, 1.0),
        '=' => Color::from_rgb(1.0, 0.82, 0.24),
        '!' => Color::from_rgb(0.78, 0.39, 1.0),
        '/' => Color::from_rgb(0.39, 1.0, 0.78),
        '?' => Color::from_rgb(0.90, 0.90, 0.39),
        '%' => Color::from_rgb(0.39, 0.78, 0.39),
        '-' | '|' => Color::from_rgb(0.63, 0.63, 0.63),
        '.' => Color::from_rgb(0.27, 0.27, 0.35),
        '#' => Color::from_rgb(0.43, 0.31, 0.20),
        '+' => Color::from_rgb(0.71, 0.51, 0.24),
        '>' => Color::from_rgb(1.0, 0.82, 0.20),
        '^' => Color::from_rgb(1.0, 0.31, 0.31),
        _ => Color::from_rgb(0.71, 0.71, 0.71),
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
