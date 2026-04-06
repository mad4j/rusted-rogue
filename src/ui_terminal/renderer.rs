use std::collections::{HashMap, HashSet};

use iced::widget::canvas;
use iced::{Color, Font, Point};

use crate::core_types::{Position, TileFlags, DCOLS, DROWS};
use crate::game_loop::GameLoop;
use crate::inventory_items::{total_armor_bonus, ItemCategory};


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
        '*' => Color::from_rgb(1.0, 0.84, 0.0),
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

pub(super) fn render_game(frame: &mut canvas::Frame, game: &GameLoop, show_inventory: bool, dim_panel: bool, blink_on: bool, message: &str, has_more: bool) {
    let lookups = RenderLookups::from_game(game);

    const MSG_COLOR: Color = Color { r: 1.0, g: 0.78, b: 0.59, a: 1.0 };
    frame.fill_text(cell_text(
        message,
        0,
        0,
        MSG_COLOR,
    ));

    if has_more {
        // Draw "--More--" in inverted colours (amber bg, black text) immediately
        // after the message text, matching the original prompt position.
        const MORE_TEXT: &str = "--More--";
        let more_col = (message.chars().count() + 1).min(DCOLS - MORE_TEXT.len());
        let more_x = more_col as f32 * super::CELL_W + super::PADDING;
        let more_y = super::PADDING; // row 0
        frame.fill_rectangle(
            iced::Point::new(more_x, more_y),
            iced::Size::new(MORE_TEXT.len() as f32 * super::CELL_W, super::CELL_H),
            MSG_COLOR,
        );
        frame.fill_text(cell_text(MORE_TEXT, more_col, 0, Color::BLACK));
    }

    let show_inventory_overlay = show_inventory || game.state().pending_item_action.is_some();
    // Dim map cells under the panel area whenever any side panel is visible.
    let should_dim = dim_panel || show_inventory_overlay;

    for row in 0..DROWS {
        for col in 0..DCOLS {
            let ch = render_cell(game, Position::new(row as i16, col as i16), &lookups);
            // Dim cells that fall under the side panel so their colour does not
            // bleed through the semi-transparent overlay.
            let color = if should_dim && col >= PANEL_COL {
                dim_color(cell_color(ch))
            } else {
                cell_color(ch)
            };
            frame.fill_text(cell_text(ch.to_string(), col, row + 1, color));
        }
    }

    render_status_bar(frame, game, blink_on);

    // Overlay the inventory panel when 'i' is pressed or an item action is pending.
    if show_inventory_overlay {
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

        let mut floor_items: HashMap<Position, char> = game
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

        // Gold piles display as '*'.
        for gold_pile in &game.state().floor_gold {
            floor_items.entry(gold_pile.position).or_insert('*');
        }

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

// Fixed-width status bar layout (80 cols):
// Col  0: "Level:" label  Col  6: value (3)
// Col  9: "Gold:"  label  Col 14: value (6)
// Col 20: " Hp:"   label  Col 24: value (8, e.g. "999(999)")
// Col 32: " Str:"  label  Col 37: value (7, e.g. "31(31) ")
// Col 44: " Arm:"  label  Col 49: value (3)
// Col 52: " Exp:"  label  Col 57: value (9, e.g. "21/999999")
// Col 66-67: spacing
// Col 68: hunger (6)  Col 74: wizard (6)
fn render_status_bar(frame: &mut canvas::Frame, game: &GameLoop, blink_on: bool) {
    let s = game.state();
    let arm = total_armor_bonus(&s.inventory);
    let row = DROWS + 1;
    let lbl = Color::from_rgb(0.45, 0.45, 0.5);

    // Level
    frame.fill_text(cell_text("Level:", 0, row, lbl));
    frame.fill_text(cell_text(format!("{:<3}", s.level), 6, row, Color::WHITE));

    // Gold
    frame.fill_text(cell_text("Gold:", 9, row, lbl));
    frame.fill_text(cell_text(format!("{:<6}", s.gold), 14, row, Color::from_rgb(1.0, 0.82, 0.24)));

    // Hp — blink bright-red / dark-red when ≤25% max
    let hp_critical = s.player_hit_points * 4 <= s.player_max_hit_points;
    let hp_color = if hp_critical {
        if blink_on {
            Color::from_rgb(1.0, 0.15, 0.15)  // bright red (blink phase ON)
        } else {
            Color::from_rgb(0.55, 0.05, 0.05) // dark red   (blink phase OFF)
        }
    } else {
        Color::from_rgb(0.2, 0.9, 0.2)
    };
    frame.fill_text(cell_text(" Hp:", 20, row, lbl));
    let hp_str = format!("{}({})", s.player_hit_points, s.player_max_hit_points);
    frame.fill_text(cell_text(format!("{:<8}", hp_str), 24, row, hp_color));

    // Str
    frame.fill_text(cell_text(" Str:", 32, row, lbl));
    let str_str = format!("{}({})", s.player_strength, s.player_max_strength);
    frame.fill_text(cell_text(format!("{:<7}", str_str), 37, row, Color::from_rgb(0.39, 0.78, 1.0)));

    // Arm
    frame.fill_text(cell_text(" Arm:", 44, row, lbl));
    frame.fill_text(cell_text(format!("{:<3}", arm), 49, row, Color::from_rgb(0.39, 1.0, 0.78)));

    // Exp
    frame.fill_text(cell_text(" Exp:", 52, row, lbl));
    let exp_str = format!("{}/{}", s.player_exp_level, s.player_exp_points);
    frame.fill_text(cell_text(format!("{:<9}", exp_str), 57, row, Color::from_rgb(0.78, 0.39, 1.0)));

    // Hunger — Weak blinks red/dark-red; Hungry stays orange
    if s.is_weak || s.is_hungry {
        let hunger_color = if s.is_weak {
            if blink_on {
                Color::from_rgb(1.0, 0.15, 0.15)
            } else {
                Color::from_rgb(0.55, 0.05, 0.05)
            }
        } else {
            // Hungry: static orange
            Color::from_rgb(1.0, 0.55, 0.1)
        };
        let hunger_str = if s.is_weak { "Weak  " } else { "Hungry" };
        frame.fill_text(cell_text(hunger_str, 68, row, hunger_color));
    }

    // Wizard tag (col 74, 6 chars)
    if s.wizard {
        frame.fill_text(cell_text("[WIZ] ", 74, row, Color::from_rgb(0.2, 1.0, 0.4)));
    }
}

// ---------------------------------------------------------------------------
// Stats panel overlay
// ---------------------------------------------------------------------------

/// Render the end-of-run statistics as a side panel overlaid on the right side
/// of the map, matching the layout of the help and inventory panels.
pub(super) fn render_stats(frame: &mut canvas::Frame, game: &GameLoop) {
    const PANEL_WIDTH: usize = 28; // cols 52-79; flush to the right edge
    // Interior usable width: PANEL_WIDTH - 2 border cols = 26 chars.
    // Label column is fixed at 17 chars so ": " + value fits in 26.
    const LABEL_W: usize = 17;

    let s = game.state();

    // Outcome-based title and colour.
    let (title, title_color) = if s.player_dead {
        ("  DEFEATED", Color::from_rgb(1.0, 0.25, 0.25))
    } else if s.quit_requested {
        ("  QUIT", Color::from_rgb(0.7, 0.7, 0.7))
    } else {
        ("  STATISTICS", Color::from_rgb(1.0, 0.82, 0.24))
    };

    let label_color  = Color::from_rgb(0.6, 0.8, 0.6);
    let value_color  = Color::WHITE;
    let footer_color = Color::from_rgb(0.45, 0.45, 0.5);

    // Dark background rectangle — same as inventory/help panel.
    let bg_x = PANEL_COL as f32 * super::CELL_W + super::PADDING;
    let bg_y = super::PADDING;
    let bg_w = PANEL_WIDTH as f32 * super::CELL_W;
    let bg_h = (DROWS + 2) as f32 * super::CELL_H;
    frame.fill_rectangle(
        Point::new(bg_x, bg_y),
        iced::Size::new(bg_w, bg_h),
        Color::from_rgba(0.04, 0.04, 0.14, 0.95),
    );

    // Header
    frame.fill_text(cell_text(title, PANEL_COL, 0, title_color));

    // Stat rows — label and value on the same line.
    // Format: "{label:<LABEL_W}: {value}"
    let rows: &[(&str, String)] = &[
        ("Monsters defeated", format!("{}", s.stats.monsters_defeated)),
        ("Gold collected",    format!("{}", s.stats.gold_collected)),
        ("Food eaten",        format!("{}", s.stats.food_eaten)),
        ("Time (turns)",      format!("{}", s.turns)),
        ("Steps taken",       format!("{}", s.stats.steps_taken)),
        ("Damage dealt",      format!("{}", s.stats.damage_dealt)),
        ("Health recovered",  format!("{}", s.stats.health_recovered)),
    ];

    for (i, (label, value)) in rows.iter().enumerate() {
        let row = i + 2;
        if row >= DROWS { break; }
        frame.fill_text(cell_text(
            format!("{:<LABEL_W$}", label),
            PANEL_COL + 1, row, label_color,
        ));
        frame.fill_text(cell_text(
            format!(": {}", value),
            PANEL_COL + 1 + LABEL_W, row, value_color,
        ));
    }

    // Footer pinned to the last row of the panel background.
    let footer_row = DROWS + 1;
    frame.fill_text(cell_text("SPACE to continue", PANEL_COL + 1, footer_row, footer_color));
}
