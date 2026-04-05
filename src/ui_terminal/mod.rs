mod canvas;
mod help;
mod input;
mod messages;
mod renderer;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::widget::canvas as iced_canvas;
use iced::widget::image as img_widget;
use iced::{ContentFit, Element, Length, Size, Subscription, Task};

use crate::core_types::{DCOLS, DROWS};
use crate::game_loop::{GameLoop, StepOutcome};

use canvas::GameCanvas;

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

        if let Key::Character(s) = &key {
            if s.as_str() == "?" {
                self.show_help = true;
                self.help_page = 0;
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

    fn view(&self) -> Element<Message> {
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
