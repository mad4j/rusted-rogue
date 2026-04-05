mod canvas;
mod help;
mod icon;
mod input;
mod messages;
mod renderer;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::widget::canvas as iced_canvas;
use iced::widget::image as img_widget;
use iced::{ContentFit, Element, Length, Size, Subscription, Task};

use crate::core_types::{DCOLS, DROWS};
use crate::game_loop::{Command, GameLoop, StepOutcome};

use canvas::GameCanvas;

// Splash screen PNG embedded at compile time
const SPLASH_BYTES: &[u8] = include_bytes!("../../resources/splash.png");

// Pixel width/height of each console cell (8-pixel glyph × 2× scale)
const CELL_W: f32 = 10.0;
const CELL_H: f32 = 16.0;
// Font size used to render each glyph inside a cell
const FONT_SIZE: f32 = 14.0;
// Extra rows below the map for status and message lines
const UI_ROWS: usize = 2;
// Empty border around the game area (pixels on each side)
pub(super) const PADDING: f32 = 8.0;

pub fn run(game: GameLoop) {
    let win_w = DCOLS as f32 * CELL_W + 2.0 * PADDING;
    let win_h = (DROWS + UI_ROWS) as f32 * CELL_H + 2.0 * PADDING;

    iced::application("Rusted Rogue", RogueApp::update, RogueApp::view)
        .subscription(RogueApp::subscription)
        .window(iced::window::Settings {
            size: Size::new(win_w, win_h),
            resizable: false,
            icon: icon::window_icon(),
            ..Default::default()
        })
        .run_with(move || {
            let splash_handle = img_widget::Handle::from_bytes(SPLASH_BYTES);
            (RogueApp { game, show_help: false, help_page: 0, screen: Screen::Splash, splash_handle, show_inventory: false }, Task::none())
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
    show_inventory: bool,
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
                    if self.help_page + 1 < help::HELP_PAGES.len() {
                        self.help_page += 1;
                    }
                }
                _ => {
                    self.show_help = false;
                }
            }
            return Task::none();
        }

        // Dismiss plain inventory browse.
        if self.show_inventory {
            self.show_inventory = false;
            return Task::none();
        }

        // Item-selection mode: route letter keys or Escape to the game loop.
        if self.game.state().pending_item_action.is_some() {
            match &key {
                Key::Named(Named::Escape) => {
                    let _ = self.game.step(Command::CancelItemSelect);
                }
                Key::Character(s) => {
                    let ch = s.chars().next().unwrap_or('\0');
                    if ch.is_ascii_lowercase() {
                        let outcome = self.game.step(Command::SelectItem(ch));
                        if outcome == StepOutcome::Finished {
                            return iced::exit();
                        }
                    }
                }
                _ => {}
            }
            return Task::none();
        }

        if let Key::Character(s) = &key {
            if s.as_str() == "?" {
                self.show_help = true;
                self.help_page = 0;
                return Task::none();
            }
            if s.as_str() == "i" {
                self.show_inventory = true;
                return Task::none();
            }
        }

        if let Some(cmd) = input::key_to_command(&key) {
            let outcome = self.game.step(cmd);
            if outcome == StepOutcome::Finished {
                return iced::exit();
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Splash => img_widget::Image::new(self.splash_handle.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Contain)
                .into(),
            Screen::Game => iced_canvas::Canvas::new(GameCanvas {
                game: &self.game,
                show_help: self.show_help,
                help_page: self.help_page,
                show_inventory: self.show_inventory,
            })
            .width(Length::Fixed(DCOLS as f32 * CELL_W + 2.0 * PADDING))
            .height(Length::Fixed((DROWS + UI_ROWS) as f32 * CELL_H + 2.0 * PADDING))
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



#[cfg(test)]
mod tests {
    use super::renderer::{render_cell, RenderLookups};
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

        // Ensure the monster's tile is in the explored set so the renderer shows it.
        game.state_mut().explored.insert(monster);

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
