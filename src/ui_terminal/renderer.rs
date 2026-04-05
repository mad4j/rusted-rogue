use std::collections::{HashMap, HashSet};

use iced::widget::canvas;
use iced::{Color, Font, Point};

use crate::core_types::{Position, TileFlags, DCOLS, DROWS};
use crate::game_loop::GameLoop;
use crate::inventory_items::ItemCategory;

use super::messages::render_last_message;

// ---------------------------------------------------------------------------
// Shared text helper
// ---------------------------------------------------------------------------

pub(super) fn cell_text(
    content: impl Into<String>,
    col: usize,
    row: usize,
    color: Color,
) -> canvas::Text {
    canvas::Text {
        content: content.into(),
        position: Point::new(col as f32 * super::CELL_W, row as f32 * super::CELL_H),
        color,
        size: iced::Pixels(super::FONT_SIZE),
        line_height: iced::widget::text::LineHeight::Absolute(iced::Pixels(super::CELL_H)),
        font: Font::MONOSPACE,
        horizontal_alignment: iced::alignment::Horizontal::Left,
        vertical_alignment: iced::alignment::Vertical::Top,
        shaping: iced::widget::text::Shaping::Basic,
    }
}

// ---------------------------------------------------------------------------
// Color mapping
// ---------------------------------------------------------------------------

pub(super) fn cell_color(ch: char) -> Color {
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

// ---------------------------------------------------------------------------
// Game screen rendering
// ---------------------------------------------------------------------------

pub(super) fn render_game(frame: &mut canvas::Frame, game: &GameLoop) {
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
    frame.fill_text(cell_text(
        message,
        0,
        DROWS + 1,
        Color::from_rgb(1.0, 0.78, 0.59),
    ));
}

// ---------------------------------------------------------------------------
// Cell rendering
// ---------------------------------------------------------------------------

pub(super) fn render_cell(
    game: &GameLoop,
    position: Position,
    lookups: &RenderLookups,
) -> char {
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

pub(super) struct RenderLookups {
    pub(super) monsters: HashMap<Position, char>,
    pub(super) floor_items: HashMap<Position, char>,
    pub(super) known_traps: HashSet<Position>,
}

impl RenderLookups {
    pub(super) fn from_game(game: &GameLoop) -> Self {
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
