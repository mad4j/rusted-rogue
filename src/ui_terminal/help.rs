use iced::widget::canvas;
use iced::{Color, Point};

use crate::core_types::DROWS;

use super::renderer::{cell_color, cell_text};

// Panel constants – mirror the inventory overlay
const PANEL_COL: usize = 42;
const PANEL_WIDTH: usize = 36;

// ---------------------------------------------------------------------------
// Help content
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub(super) enum HelpLine {
    Section(&'static str),
    Binding(&'static str, &'static str), // key  →  description
    Symbol(char, &'static str),          // glyph →  description
    Empty,
}

// Page 1 – movement and item actions
const HELP_PAGE_1: &[HelpLine] = &[
    HelpLine::Section("MOVEMENT"),
    HelpLine::Binding("h", "left"),
    HelpLine::Binding("l", "right"),
    HelpLine::Binding("k", "up"),
    HelpLine::Binding("j", "down"),
    HelpLine::Binding("y", "up-left"),
    HelpLine::Binding("u", "up-right"),
    HelpLine::Binding("b", "dn-left"),
    HelpLine::Binding("n", "dn-right"),
    HelpLine::Binding("H/J/K/L", "run"),
    HelpLine::Binding("Y/U/B/N", "run diag"),
    HelpLine::Empty,
    HelpLine::Section("ACTIONS"),
    HelpLine::Binding(",", "pick up"),
    HelpLine::Binding("d", "drop"),
    HelpLine::Binding("e", "eat"),
    HelpLine::Binding("q", "quaff"),
    HelpLine::Binding("r", "read"),
    HelpLine::Binding("z", "zap wand"),
    HelpLine::Binding("t", "throw"),
    HelpLine::Binding("w", "wield"),
    HelpLine::Empty,
    HelpLine::Section("ARMOR & RINGS"),
    HelpLine::Binding("W", "wear"),
    HelpLine::Binding("T", "take off"),
    HelpLine::Binding("P", "ring on"),
    HelpLine::Binding("R", "ring off"),
];

// Page 2 – game controls and map legend
const HELP_PAGE_2: &[HelpLine] = &[
    HelpLine::Section("GAME"),
    HelpLine::Binding(".", "rest"),
    HelpLine::Binding(">", "descend"),
    HelpLine::Binding("^", "id trap"),
    HelpLine::Binding("i", "inventory"),
    HelpLine::Binding("S", "save"),
    HelpLine::Binding("L", "load"),
    HelpLine::Binding("Q/Esc", "quit"),
    HelpLine::Empty,
    HelpLine::Section("TERRAIN"),
    HelpLine::Symbol('.', "floor"),
    HelpLine::Symbol('#', "tunnel"),
    HelpLine::Symbol('+', "door"),
    HelpLine::Symbol('-', "horiz. wall"),
    HelpLine::Symbol('|', "vert. wall"),
    HelpLine::Symbol('>', "stairs"),
    HelpLine::Symbol('^', "trap"),
    HelpLine::Empty,
    HelpLine::Section("ITEMS"),
    HelpLine::Symbol(')', "weapon"),
    HelpLine::Symbol(']', "armor"),
    HelpLine::Symbol('!', "potion"),
    HelpLine::Symbol('?', "scroll"),
    HelpLine::Symbol('/', "wand"),
    HelpLine::Symbol('=', "ring"),
    HelpLine::Symbol('%', "food"),
    HelpLine::Empty,
    HelpLine::Section("ENTITIES"),
    HelpLine::Symbol('@', "player"),
    HelpLine::Symbol('k', "monsters a-z/A-Z"),
];

pub(super) const HELP_PAGES: &[&[HelpLine]] = &[HELP_PAGE_1, HELP_PAGE_2];

// ---------------------------------------------------------------------------
// Help overlay rendering  (same panel style as inventory)
// ---------------------------------------------------------------------------

pub(super) fn render_help_overlay(frame: &mut canvas::Frame, page: usize) {
    const GOLD: Color = Color { r: 1.0, g: 0.78, b: 0.20, a: 1.0 };
    const CYAN: Color = Color { r: 0.39, g: 0.86, b: 1.0, a: 1.0 };
    const YELLOW: Color = Color { r: 1.0, g: 0.86, b: 0.31, a: 1.0 };
    const WHITE: Color = Color { r: 0.86, g: 0.86, b: 0.86, a: 1.0 };
    const DIM: Color = Color { r: 0.43, g: 0.43, b: 0.43, a: 1.0 };

    let total = HELP_PAGES.len();

    // Dark background rectangle – same dimensions as the inventory panel
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
    frame.fill_text(cell_text("  HELP", PANEL_COL, 0, GOLD));

    // Page indicator
    let indicator = format!("  -- {}/{} --", page + 1, total);
    frame.fill_text(cell_text(indicator, PANEL_COL, 1, DIM));

    // Content lines
    for (i, line) in HELP_PAGES[page].iter().enumerate() {
        let row = i + 2;
        match line {
            HelpLine::Section(text) => {
                frame.fill_text(cell_text(*text, PANEL_COL + 1, row, CYAN));
            }
            HelpLine::Binding(key, desc) => {
                frame.fill_text(cell_text(*key, PANEL_COL + 2, row, YELLOW));
                frame.fill_text(cell_text(*desc, PANEL_COL + 10, row, WHITE));
            }
            HelpLine::Symbol(ch, desc) => {
                frame.fill_text(cell_text(ch.to_string(), PANEL_COL + 2, row, cell_color(*ch)));
                frame.fill_text(cell_text(*desc, PANEL_COL + 5, row, WHITE));
            }
            HelpLine::Empty => {}
        }
    }

    // Navigation footer – same row as inventory footer
    let nav = if page == 0 && total > 1 {
        "-> next  |  any key=close"
    } else if page + 1 == total && page > 0 {
        "<- back  |  any key=close"
    } else {
        "any key to close"
    };
    frame.fill_text(cell_text(nav, PANEL_COL, DROWS, DIM));
}
