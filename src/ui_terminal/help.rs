use iced::widget::canvas;
use iced::Color;

use crate::core_types::{DCOLS, DROWS};

use super::renderer::{cell_color, cell_text};

// ---------------------------------------------------------------------------
// Help content data
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub(super) enum HelpLine {
    Section(&'static str),
    Section2(&'static str, &'static str),
    Binding2(&'static str, &'static str, &'static str, &'static str),
    Symbol(char, &'static str),
    Symbol2(char, &'static str, char, &'static str),
    Empty,
}

// Page 1: all key bindings in a 2-column layout
// left col: key at 2, desc at 17  |  right col: key at 40, desc at 43
const HELP_PAGE_1: &[HelpLine] = &[
    HelpLine::Section2("Movement", "Actions"),
    HelpLine::Binding2("h / ArrowLeft", "left", ",", "pick up"),
    HelpLine::Binding2("l / ArrowRight", "right", "d", "drop"),
    HelpLine::Binding2("k / ArrowUp", "up", "e", "eat"),
    HelpLine::Binding2("j / ArrowDown", "down", "q", "quaff"),
    HelpLine::Binding2("y", "up-left", "r", "read scroll"),
    HelpLine::Binding2("u", "up-right", "z", "zap wand"),
    HelpLine::Binding2("b", "down-left", "t", "throw"),
    HelpLine::Binding2("n", "down-right", "w", "wield weapon"),
    HelpLine::Binding2("H / J / K / L", "run straight", "W", "wear armor"),
    HelpLine::Binding2("Y / U / B / N", "run diagonal", "T", "take off armor"),
    HelpLine::Binding2("", "", "P", "put on ring"),
    HelpLine::Binding2("", "", "R", "remove ring"),
    HelpLine::Empty,
    HelpLine::Section2("Game", ""),
    HelpLine::Binding2(".", "rest", "S", "save"),
    HelpLine::Binding2(">", "descend", "L", "load"),
    HelpLine::Binding2("^", "ident. trap", "?", "this help"),
    HelpLine::Binding2("Q / Esc", "quit", "", ""),
];

// Page 2: map symbols with actual game colours
const HELP_PAGE_2: &[HelpLine] = &[
    HelpLine::Section2("Terrain", "Items"),
    HelpLine::Symbol2('.', "floor", ')', "weapon"),
    HelpLine::Symbol2('#', "tunnel / passage", ']', "armor"),
    HelpLine::Symbol2('+', "door", '!', "potion"),
    HelpLine::Symbol2('-', "horiz. wall", '?', "scroll"),
    HelpLine::Symbol2('|', "vert. wall", '/', "wand"),
    HelpLine::Symbol2('>', "stairs down", '=', "ring"),
    HelpLine::Symbol2('^', "trap", '%', "food"),
    HelpLine::Empty,
    HelpLine::Section("Entities"),
    HelpLine::Symbol('@', "you (player)"),
    HelpLine::Symbol('k', "monster  (a-z / A-Z)"),
];

pub(super) const HELP_PAGES: &[&[HelpLine]] = &[HELP_PAGE_1, HELP_PAGE_2];

// ---------------------------------------------------------------------------
// Help screen rendering
// ---------------------------------------------------------------------------

pub(super) fn render_help_page(frame: &mut canvas::Frame, page: usize) {
    const GOLD: Color = Color { r: 1.0, g: 0.78, b: 0.20, a: 1.0 };
    const CYAN: Color = Color { r: 0.39, g: 0.86, b: 1.0, a: 1.0 };
    const YELLOW: Color = Color { r: 1.0, g: 0.86, b: 0.31, a: 1.0 };
    const WHITE: Color = Color { r: 0.86, g: 0.86, b: 0.86, a: 1.0 };
    const DIM: Color = Color { r: 0.43, g: 0.43, b: 0.43, a: 1.0 };

    let total = HELP_PAGES.len();

    frame.fill_text(cell_text("RUSTED ROGUE  -  HELP", DCOLS / 2 - 10, 0, GOLD));
    let indicator = format!("-- page {} of {} --", page + 1, total);
    frame.fill_text(cell_text(indicator, DCOLS / 2 - 8, 1, DIM));

    for (i, line) in HELP_PAGES[page].iter().enumerate() {
        let row = i + 3;
        match line {
            HelpLine::Section(text) => {
                frame.fill_text(cell_text(*text, 2, row, CYAN));
            }
            HelpLine::Section2(left, right) => {
                frame.fill_text(cell_text(*left, 2, row, CYAN));
                if !right.is_empty() {
                    frame.fill_text(cell_text(*right, 40, row, CYAN));
                }
            }
            HelpLine::Binding2(key1, desc1, key2, desc2) => {
                if !key1.is_empty() {
                    frame.fill_text(cell_text(*key1, 2, row, YELLOW));
                    frame.fill_text(cell_text(*desc1, 17, row, WHITE));
                }
                if !key2.is_empty() {
                    frame.fill_text(cell_text(*key2, 40, row, YELLOW));
                    frame.fill_text(cell_text(*desc2, 43, row, WHITE));
                }
            }
            HelpLine::Symbol(ch, desc) => {
                frame.fill_text(cell_text(ch.to_string(), 4, row, cell_color(*ch)));
                frame.fill_text(cell_text(*desc, 8, row, WHITE));
            }
            HelpLine::Symbol2(ch1, desc1, ch2, desc2) => {
                frame.fill_text(cell_text(ch1.to_string(), 4, row, cell_color(*ch1)));
                frame.fill_text(cell_text(*desc1, 8, row, WHITE));
                frame.fill_text(cell_text(ch2.to_string(), 44, row, cell_color(*ch2)));
                frame.fill_text(cell_text(*desc2, 47, row, WHITE));
            }
            HelpLine::Empty => {}
        }
    }

    let nav = match (page == 0, page + 1 == total) {
        (_, true) => "<- ArrowLeft: prev page   |   any other key: close",
        (true, _) => "any other key: close   |   ArrowRight: next page ->",
        _ => "<- ArrowLeft: prev   |   ArrowRight: next ->   |   any key: close",
    };
    frame.fill_text(cell_text(nav, 2, DROWS + super::UI_ROWS - 1, DIM));
}
