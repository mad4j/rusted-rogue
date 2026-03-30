use std::collections::{HashMap, HashSet};
use std::io::{self, stdout, Stdout, Write};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};

use crate::actors::{CombatEvent, MonsterKind, StatusEffectEvent};
use crate::core_types::{Position, TileFlags, DCOLS, DROWS};
use crate::game_loop::{Command, Direction, GameLoop, StepOutcome};
use crate::inventory_items::{InventoryEvent, ItemCategory};

pub fn run(mut game: GameLoop) -> io::Result<()> {
    let mut terminal = stdout();
    let _guard = TerminalGuard::enter(&mut terminal)?;

    render(&mut terminal, &game)?;

    loop {
        if game.state().quit_requested {
            break;
        }

        let Event::Key(key_event) = read()? else {
            continue;
        };

        if key_event.kind != KeyEventKind::Press {
            continue;
        }

        let Some(command) = map_key_to_command(key_event) else {
            continue;
        };

        let outcome = game.step(command);
        render(&mut terminal, &game)?;

        if outcome == StepOutcome::Finished {
            break;
        }
    }

    Ok(())
}

fn render(stdout: &mut Stdout, game: &GameLoop) -> io::Result<()> {
    queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

    let lookups = RenderLookups::from_game(game);

    for row in 0..(DROWS as i16) {
        let mut line = String::with_capacity(DCOLS);

        for col in 0..(DCOLS as i16) {
            line.push(render_cell(game, Position::new(row, col), &lookups));
        }

        queue!(stdout, MoveTo(0, row as u16), Print(line))?;
    }

    queue!(
        stdout,
        MoveTo(0, DROWS as u16),
        Clear(ClearType::CurrentLine),
        Print(render_status(game)),
        MoveTo(0, (DROWS + 1) as u16),
        Clear(ClearType::CurrentLine),
        Print(render_last_message(game)),
        MoveTo(0, (DROWS + 2) as u16),
        Clear(ClearType::CurrentLine),
        Print("Keys: hjkl yubn/arrows move, . rest, , pick up, d drop, w wield, W wear, T take off, P put ring, R remove ring, q quaff, z zap, t throw, ^ identify trap, Q quit"),
    )?;

    stdout.flush()
}

fn render_cell(game: &GameLoop, position: Position, lookups: &RenderLookups) -> char {
    if game.state().player_position == position {
        return '@';
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
    format!(
        "Level:{} Turns:{} HP:{}/{} Defeated:{} Inv:{} Monsters:{} Frozen:{}{}",
        game.state().level,
        game.state().turns,
        game.state().player_hit_points,
        game.state().player_max_hit_points,
        game.state().monsters_defeated,
        game.state().inventory.len(),
        game.state().monsters.len(),
        game.state().frozen_turns,
        if game.player_is_held() { " held" } else { "" },
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

fn map_key_to_command(key_event: KeyEvent) -> Option<Command> {
    if key_event.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key_event.code, KeyCode::Char('c'))
    {
        return Some(Command::Quit);
    }

    let command = match key_event.code {
        KeyCode::Left => Command::Move(Direction::Left),
        KeyCode::Right => Command::Move(Direction::Right),
        KeyCode::Up => Command::Move(Direction::Up),
        KeyCode::Down => Command::Move(Direction::Down),
        KeyCode::Esc => Command::Quit,
        KeyCode::Char(character) => GameLoop::parse_command(character),
        _ => Command::Unknown,
    };

    match command {
        Command::Unknown => None,
        _ => Some(command),
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter(stdout: &mut Stdout) -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = stdout();
        let _ = execute!(stdout, Show, LeaveAlternateScreen);
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEventState, KeyModifiers};

    use super::{map_key_to_command, render_cell, RenderLookups};
    use crate::core_types::Position;
    use crate::game_loop::{Command, Direction, GameLoop};
    use crate::inventory_items::{FloorItem, InventoryItem};

    #[test]
    fn arrow_keys_map_to_movement_commands() {
        let left = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert_eq!(
            map_key_to_command(left),
            Some(Command::Move(Direction::Left))
        );
    }

    #[test]
    fn legacy_keys_map_through_game_parser() {
        let wield = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('w'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert_eq!(map_key_to_command(wield), Some(Command::Wield));
    }

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
