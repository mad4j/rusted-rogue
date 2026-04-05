use std::collections::{HashMap, HashSet};

use iced::widget::canvas;
use iced::{Color, Font, Point};

use crate::core_types::{Position, TileFlags, DCOLS, DROWS};
use crate::game_loop::GameLoop;
use crate::inventory_items::{total_armor_bonus, ItemCategory};

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
        position: Point::new(col as f32 * super::CELL_W + super::PADDING, row as f32 * super::CELL_H + super::PADDING),
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

// Column at which the side panel starts (keep in sync with overlay constants)
const PANEL_COL: usize = 52;

pub(super) fn render_game(frame: &mut canvas::Frame, game: &GameLoop, show_inventory: bool) {
    let lookups = RenderLookups::from_game(game);

    let message = render_last_message(game);
    frame.fill_text(cell_text(
        message,
        0,
        0,
        Color::from_rgb(1.0, 0.78, 0.59),
    ));

    let show_overlay = show_inventory || game.state().pending_item_action.is_some();

    for row in 0..DROWS {
        for col in 0..DCOLS {
            let ch = render_cell(game, Position::new(row as i16, col as i16), &lookups);
            // Dim cells that fall under the side panel so their colour does not
            // bleed through the semi-transparent overlay.
            let color = if show_overlay && col >= PANEL_COL {
                dim_color(cell_color(ch))
            } else {
                cell_color(ch)
            };
            frame.fill_text(cell_text(ch.to_string(), col, row + 1, color));
        }
    }

    let status = render_status(game);
    frame.fill_text(cell_text(status, 0, DROWS + 1, Color::from_rgb(1.0, 1.0, 0.39)));

    // Overlay the inventory panel when 'i' is pressed or an item action is pending.
    if show_overlay {
        render_inventory_overlay(frame, game);
    }
}

/// Reduce brightness of a colour to indicate it is behind the panel overlay.
fn dim_color(c: Color) -> Color {
    Color { r: c.r * 0.25, g: c.g * 0.25, b: c.b * 0.25, a: c.a }
}

/// Draw the inventory list (or item-selection prompt) overlaid on the right side of the screen.
fn render_inventory_overlay(frame: &mut canvas::Frame, game: &GameLoop) {
    use crate::inventory_items::EquipmentSlot;

    const PANEL_WIDTH: usize = 28; // cols 52-79; flush to the right edge

    let state = game.state();
    let pending = &state.pending_item_action;

    // Dark background rectangle.
    let bg_x = PANEL_COL as f32 * super::CELL_W + super::PADDING;
    let bg_y = super::PADDING;
    let bg_w = PANEL_WIDTH as f32 * super::CELL_W;
    let bg_h = (DROWS + 2) as f32 * super::CELL_H;
    frame.fill_rectangle(
        Point::new(bg_x, bg_y),
        iced::Size::new(bg_w, bg_h),
        Color::from_rgba(0.04, 0.04, 0.14, 0.95),
    );

    // Header line – arrows are dim since inventory has no pagination
    frame.fill_text(cell_text("  INVENTORY", PANEL_COL, 0, Color::from_rgb(1.0, 1.0, 0.4)));

    // Determine which items to list.
    let filter_cat = pending.as_ref().and_then(|a| a.filter_category());
    let equipped_only = pending.as_ref().map(|a| a.equipped_only()).unwrap_or(false);

    let items: Vec<&crate::inventory_items::InventoryEntry> = state
        .inventory
        .iter()
        .filter(|e| {
            if let Some(cat) = filter_cat {
                if e.item.category != cat {
                    return false;
                }
            }
            if equipped_only && e.equipped_slot.is_none() {
                return false;
            }
            true
        })
        .collect();

    if items.is_empty() {
        let msg = if let Some(action) = pending {
            action.empty_message()
        } else {
            "your pack is empty"
        };
        frame.fill_text(cell_text(
            format!("  {}", msg),
            PANEL_COL,
            2,
            Color::from_rgb(0.7, 0.7, 0.7),
        ));
    } else {
        for (idx, entry) in items.iter().enumerate() {
            let row = 2 + idx;
            if row >= DROWS { break; } // clamp to panel height
            let slot_label = match entry.equipped_slot {
                Some(EquipmentSlot::Weapon) => " (weapon in hand)",
                Some(EquipmentSlot::Armor) => " (being worn)",
                Some(EquipmentSlot::LeftRing) => " (on left finger)",
                Some(EquipmentSlot::RightRing) => " (on right finger)",
                None => "",
            };
            let name = if entry.quantity > 1 {
                format!("{} {}", entry.quantity, entry.item.name)
            } else {
                entry.item.name.to_string()
            };
            let line = format!(" {}) {}{}", entry.ichar, name, slot_label);
            // Truncate to panel width.
            let line: String = line.chars().take(PANEL_WIDTH - 1).collect();
            frame.fill_text(cell_text(
                line,
                PANEL_COL,
                row,
                Color::from_rgb(0.85, 0.85, 0.85),
            ));
        }
    }

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
        "  Weak"
    } else if game.state().is_hungry {
        "  Hungry"
    } else {
        ""
    };
    let arm = total_armor_bonus(&game.state().inventory);
    format!(
        "Level: {}  Gold: {}  Hp: {}({})  Str: {}({})  Arm: {}  Exp: {}/{}{}",
        game.state().level,
        game.state().gold,
        game.state().player_hit_points,
        game.state().player_max_hit_points,
        game.state().player_strength,
        game.state().player_max_strength,
        arm,
        game.state().player_exp_level,
        game.state().player_exp_points,
        hunger,
    )
}
