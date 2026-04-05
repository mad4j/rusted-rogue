use iced::keyboard::key::Named;
use iced::keyboard::Key;

use crate::game_loop::{Command, Direction, GameLoop};

pub(super) fn key_to_command(key: &Key) -> Option<Command> {
    match key {
        Key::Named(Named::Escape) => Some(Command::Quit),
        Key::Named(Named::ArrowLeft) => Some(Command::Move(Direction::Left)),
        Key::Named(Named::ArrowRight) => Some(Command::Move(Direction::Right)),
        Key::Named(Named::ArrowUp) => Some(Command::Move(Direction::Up)),
        Key::Named(Named::ArrowDown) => Some(Command::Move(Direction::Down)),
        Key::Character(s) => {
            if let Some(ch) = s.chars().next() {
                let cmd = GameLoop::parse_command(ch);
                if cmd != Command::Unknown {
                    return Some(cmd);
                }
            }
            None
        }
        _ => None,
    }
}
