use std::collections::VecDeque;

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
// Game-over screen PNG embedded at compile time
const GAMEOVER_BYTES: &[u8] = include_bytes!("../../resources/gameover.png");

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

    let game = std::sync::Arc::new(std::sync::Mutex::new(Some(game)));
    iced::application(
        move || {
            let game = game.lock().unwrap().take().expect("boot called only once");
            let splash_handle = img_widget::Handle::from_bytes(SPLASH_BYTES);
            let gameover_handle = img_widget::Handle::from_bytes(GAMEOVER_BYTES);
            (RogueApp { game, show_help: false, help_page: 0, screen: Screen::Splash, splash_handle, gameover_handle, show_inventory: false, show_stats: false, blink_on: false, message_queue: VecDeque::new() }, Task::none())
        },
        RogueApp::update,
        RogueApp::view,
    )
        .title("Rusted Rogue")
        .subscription(RogueApp::subscription)
        .window(iced::window::Settings {
            size: Size::new(win_w, win_h),
            resizable: false,
            icon: icon::window_icon(),
            ..Default::default()
        })
        .run()
        .unwrap();
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

enum Screen {
    Splash,
    Game,
    GameOver,
}

struct RogueApp {
    game: GameLoop,
    show_help: bool,
    help_page: usize,
    screen: Screen,
    splash_handle: img_widget::Handle,
    gameover_handle: img_widget::Handle,
    show_inventory: bool,
    show_stats: bool,
    blink_on: bool,
    /// Messages pending player acknowledgement.  When this has more than one
    /// entry, the front is shown with a `--More--` prompt and input is blocked
    /// until the player presses Space to advance through the queue.
    message_queue: VecDeque<String>,
}

#[derive(Debug, Clone)]
enum Message {
    KeyPressed(Key, Modifiers),
    Tick,
}

impl RogueApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        if matches!(message, Message::Tick) {
            self.blink_on = !self.blink_on;
            return Task::none();
        }
        let Message::KeyPressed(key, modifiers) = message else { return Task::none(); };

        if matches!(self.screen, Screen::Splash) {
            self.screen = Screen::Game;
            return Task::none();
        }

        // Stats overlay: SPACE or Escape dismisses it.
        // End-of-run: advances to GameOver (if dead) or exits (if quit).
        // Mid-game (Ctrl+A): simply closes the overlay.
        if self.show_stats {
            let dismiss = matches!(key, Key::Named(Named::Space))
                || matches!(key, Key::Named(Named::Escape));
            if dismiss {
                self.show_stats = false;
                if self.game.state().player_dead {
                    self.screen = Screen::GameOver;
                } else if self.game.state().quit_requested {
                    println!("Grazie per aver giocato a Rusted Rogue! A presto, avventuriero... se hai il coraggio di tornare.");
                    return iced::exit();
                }
                // Mid-game stats: just close the overlay and continue playing.
            }
            return Task::none();
        }

        // Game-over screen: n/N/ESC exits, any other key restarts to splash.
        if matches!(self.screen, Screen::GameOver) {
            let exit_requested = match &key {
                Key::Named(Named::Escape) => true,
                Key::Character(s) if s.as_str() == "n" || s.as_str() == "N" => true,
                _ => false,
            };
            if exit_requested {
                println!("Grazie per aver giocato a Rusted Rogue! A presto, avventuriero... se hai il coraggio di tornare.");
                return iced::exit();
            }
            // Any other key: start a fresh game and return to the splash screen.
            self.game = crate::game_loop::run();
            self.screen = Screen::Splash;
            self.show_help = false;
            self.help_page = 0;
            self.show_inventory = false;
            self.message_queue.clear();
            return Task::none();
        }

        if self.game.state().quit_requested {
            self.show_stats = true;
            return Task::none();
        }

        // Message paging: when multiple messages are queued the player MUST
        // acknowledge each with Space before input is accepted again.
        if self.message_queue.len() > 1 {
            if matches!(key, Key::Named(Named::Space)) {
                self.message_queue.pop_front();
            }
            return Task::none();
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
                Key::Named(Named::Space) => {
                    self.show_help = false;
                }
                _ => {}
            }
            return Task::none();
        }

        // Dismiss plain inventory browse.
        if self.show_inventory {
            if matches!(key, Key::Named(Named::Space)) {
                self.show_inventory = false;
            }
            return Task::none();
        }

        // Wizard password input mode: collect characters until Enter or Escape.
        if self.game.state().wizard_password_input.is_some() {
            match &key {
                Key::Named(Named::Enter) => {
                    self.step_and_collect(Command::WizardPasswordSubmit);
                }
                Key::Named(Named::Escape) => {
                    self.step_and_collect(Command::WizardPasswordCancel);
                }
                Key::Named(Named::Backspace) => {
                    self.step_and_collect(Command::WizardPasswordChar('\x08'));
                }
                Key::Character(s) => {
                    if let Some(ch) = s.chars().next() {
                        self.step_and_collect(Command::WizardPasswordChar(ch));
                    }
                }
                _ => {}
            }
            return Task::none();
        }

        // Item-selection mode: route letter keys or Escape to the game loop.
        if self.game.state().pending_item_action.is_some() {
            match &key {
                Key::Named(Named::Escape) => {
                    self.step_and_collect(Command::CancelItemSelect);
                }
                Key::Character(s) => {
                    let ch = s.chars().next().unwrap_or('\0');
                    if ch.is_ascii_lowercase() {
                        let outcome = self.step_and_collect(Command::SelectItem(ch));
                        if outcome == StepOutcome::Finished {
                            return self.handle_finished();
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

        // Ctrl+key bindings for wizard mode and other control commands.
        if modifiers.control() {
            if let Key::Character(s) = &key {
                // Ctrl+A: show statistics overlay in any mode.
                if matches!(s.as_str(), "a" | "A") {
                    self.show_stats = true;
                    return Task::none();
                }
                let cmd = match s.as_str() {
                    "w" | "W" => Some(Command::ToggleWizard),
                    "s" | "S" => Some(Command::WizardRevealMap),
                    "t" | "T" => Some(Command::WizardShowTraps),
                    "o" | "O" => Some(Command::WizardShowObjects),
                    "c" | "C" => Some(Command::WizardAddItem),
                    "m" | "M" => Some(Command::WizardShowMonsters),
                    _ => None,
                };
                if let Some(cmd) = cmd {
                    let outcome = self.step_and_collect(cmd);
                    if outcome == StepOutcome::Finished {
                        return self.handle_finished();
                    }
                    return Task::none();
                }
            }
            return Task::none();
        }

        if let Some(cmd) = input::key_to_command(&key) {
            let outcome = self.step_and_collect(cmd);
            if outcome == StepOutcome::Finished {
                return self.handle_finished();
            }
        }

        Task::none()
    }

    /// Execute a game command and refresh the message queue from the resulting
    /// game state.  Returns the step outcome.
    fn step_and_collect(&mut self, cmd: Command) -> StepOutcome {
        let outcome = self.game.step(cmd);
        let new_msgs = messages::collect_messages(&self.game);
        self.message_queue.clear();
        // Wizard password prompt is shown as the message while typing; skip
        // pushing turn messages in that mode.
        if self.game.state().wizard_password_input.is_none() {
            for msg in new_msgs {
                self.message_queue.push_back(msg);
            }
        }
        outcome
    }

    fn handle_finished(&mut self) -> Task<Message> {
        self.show_stats = true;
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Splash => img_widget::Image::new(self.splash_handle.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Contain)
                .into(),
            Screen::GameOver => img_widget::Image::new(self.gameover_handle.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Contain)
                .into(),
            Screen::Game => {
                // Wizard-password prompt preempts the normal message queue.
                let message: String = if let Some(prompt) = messages::wizard_prompt(&self.game) {
                    prompt
                } else {
                    self.message_queue.front().cloned().unwrap_or_default()
                };
                let has_more = self.message_queue.len() > 1;
                iced_canvas::Canvas::new(GameCanvas {
                    game: &self.game,
                    show_help: self.show_help,
                    help_page: self.help_page,
                    show_inventory: self.show_inventory,
                    blink_on: self.blink_on,
                    show_stats: self.show_stats,
                    message,
                    has_more,
                })
                .width(Length::Fixed(DCOLS as f32 * CELL_W + 2.0 * PADDING))
                .height(Length::Fixed((DROWS + UI_ROWS) as f32 * CELL_H + 2.0 * PADDING))
                .into()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let keyboard = iced::event::listen_with(|event, status, _window| {
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
        });
        let tick = iced::time::every(std::time::Duration::from_millis(500))
            .map(|_| Message::Tick);
        Subscription::batch([keyboard, tick])
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
