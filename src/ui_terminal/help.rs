use iced::widget::canvas;
use iced::{Color, Point};

use crate::core_types::DROWS;

use super::renderer::{cell_color, cell_text};

// Panel constants – mirror the inventory overlay
const PANEL_COL: usize = 52;
const PANEL_WIDTH: usize = 28; // cols 52-79; flush to the right edge

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

// Page 1 – movement and actions (22 lines; content rows 2-23 within the 24-row panel)
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
    HelpLine::Binding("s", "search"),
    HelpLine::Empty,
];

// Page 2 – armor, rings and game controls
const HELP_PAGE_2: &[HelpLine] = &[
    HelpLine::Section("ARMOR & RINGS"),
    HelpLine::Binding("W", "wear"),
    HelpLine::Binding("T", "take off"),
    HelpLine::Binding("P", "ring on"),
    HelpLine::Binding("R", "ring off"),
    HelpLine::Empty,
    HelpLine::Section("GAME"),
    HelpLine::Binding(".", "rest"),
    HelpLine::Binding(">", "descend"),
    HelpLine::Binding("^", "id trap"),
    HelpLine::Binding("i", "inventory"),
    HelpLine::Binding("?", "help"),
    HelpLine::Binding("S", "save"),
    HelpLine::Binding("L", "load"),
    HelpLine::Binding("Q/Esc", "quit"),
    HelpLine::Binding("Ctrl+A", "statistics"),
    HelpLine::Binding("Ctrl+P", "recall msg"),
    HelpLine::Empty,
];

// Wizard page – only shown when wizard mode is active
const HELP_PAGE_WIZARD: &[HelpLine] = &[
    HelpLine::Section("WIZARD MODE"),
    HelpLine::Binding("Ctrl+W", "toggle wizard"),
    HelpLine::Empty,
    HelpLine::Section("WIZARD COMMANDS"),
    HelpLine::Binding("Ctrl+S", "reveal map"),
    HelpLine::Binding("Ctrl+T", "show traps"),
    HelpLine::Binding("Ctrl+O", "show objects"),
    HelpLine::Binding("Ctrl+C", "conjure item"),
    HelpLine::Binding("Ctrl+M", "show monsters"),
    HelpLine::Binding("Tab", "list objects"),
    HelpLine::Empty,
];

// Page 3 – terrain, items and entities (22 lines)
const HELP_PAGE_3: &[HelpLine] = &[
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
    HelpLine::Empty,
];

// Page 4 – monsters A-M
const HELP_PAGE_4: &[HelpLine] = &[
    HelpLine::Section("MONSTERS (A-M)"),
    HelpLine::Symbol('A', "Aquator"),
    HelpLine::Symbol('B', "Bat"),
    HelpLine::Symbol('C', "Centaur"),
    HelpLine::Symbol('D', "Dragon"),
    HelpLine::Symbol('E', "Emu"),
    HelpLine::Symbol('F', "Venus fly-trap"),
    HelpLine::Symbol('G', "Griffin"),
    HelpLine::Symbol('H', "Hobgoblin"),
    HelpLine::Symbol('I', "Ice monster"),
    HelpLine::Symbol('J', "Jabberwock"),
    HelpLine::Symbol('K', "Kestrel"),
    HelpLine::Symbol('L', "Leprechaun"),
    HelpLine::Symbol('M', "Medusa"),
    HelpLine::Empty,
];

// Page 5 – monsters N-Z
const HELP_PAGE_5: &[HelpLine] = &[
    HelpLine::Section("MONSTERS (N-Z)"),
    HelpLine::Symbol('N', "Nymph"),
    HelpLine::Symbol('O', "Orc"),
    HelpLine::Symbol('P', "Phantom"),
    HelpLine::Symbol('Q', "Quagga"),
    HelpLine::Symbol('R', "Rattlesnake"),
    HelpLine::Symbol('S', "Snake"),
    HelpLine::Symbol('T', "Troll"),
    HelpLine::Symbol('U', "Black unicorn"),
    HelpLine::Symbol('V', "Vampire"),
    HelpLine::Symbol('W', "Wraith"),
    HelpLine::Symbol('X', "Xeroc"),
    HelpLine::Symbol('Y', "Yeti"),
    HelpLine::Symbol('Z', "Zombie"),
    HelpLine::Empty,
];

const HELP_PAGES_NORMAL: &[&[HelpLine]] = &[HELP_PAGE_1, HELP_PAGE_2, HELP_PAGE_3, HELP_PAGE_4, HELP_PAGE_5];
const HELP_PAGES_WIZARD: &[&[HelpLine]] = &[HELP_PAGE_1, HELP_PAGE_2, HELP_PAGE_3, HELP_PAGE_4, HELP_PAGE_5, HELP_PAGE_WIZARD];

pub(super) fn pages(wizard: bool) -> &'static [&'static [HelpLine]] {
    if wizard { HELP_PAGES_WIZARD } else { HELP_PAGES_NORMAL }
}

// ---------------------------------------------------------------------------
// Help overlay rendering  (same panel style as inventory)
// ---------------------------------------------------------------------------

pub(super) fn render_help_overlay(frame: &mut canvas::Frame, page: usize, wizard: bool) {
    const GOLD: Color = Color { r: 1.0, g: 0.78, b: 0.20, a: 1.0 };
    const CYAN: Color = Color { r: 0.39, g: 0.86, b: 1.0, a: 1.0 };
    const YELLOW: Color = Color { r: 1.0, g: 0.86, b: 0.31, a: 1.0 };
    const WHITE: Color = Color { r: 0.86, g: 0.86, b: 0.86, a: 1.0 };

    let all_pages = pages(wizard);
    let total = all_pages.len();

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

    // Header – title on the left, page-navigation arrows on the right
    frame.fill_text(cell_text("  HELP", PANEL_COL, 0, GOLD));
    if page > 0 {
        frame.fill_text(cell_text("<", PANEL_COL + PANEL_WIDTH - 4, 0, GOLD));
    }
    if page + 1 < total {
        frame.fill_text(cell_text(">", PANEL_COL + PANEL_WIDTH - 2, 0, GOLD));
    }

    // Content lines – stop before the last rows so nothing overflows the panel
    for (i, line) in all_pages[page].iter().enumerate() {
        let row = i + 1;
        if row >= DROWS { break; }
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
}
