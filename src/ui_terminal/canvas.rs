use iced::widget::canvas;
use iced::{Color, Point, Theme};

use crate::game_loop::GameLoop;

use super::help::render_help_overlay;
use super::renderer::{render_game, render_stats};
use super::Message;

pub(super) struct GameCanvas<'a> {
    pub(super) game: &'a GameLoop,
    pub(super) show_help: bool,
    pub(super) help_page: usize,
    pub(super) show_inventory: bool,
    pub(super) blink_on: bool,
    pub(super) show_stats: bool,
}

impl<'a> canvas::Program<Message> for GameCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<iced::Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        frame.fill(
            &canvas::Path::rectangle(Point::ORIGIN, bounds.size()),
            Color::BLACK,
        );

        if self.show_stats {
            render_game(&mut frame, self.game, self.show_inventory, self.blink_on);
            render_stats(&mut frame, self.game);
        } else if self.show_help {
            render_game(&mut frame, self.game, self.show_inventory, self.blink_on);
            render_help_overlay(&mut frame, self.help_page);
        } else {
            render_game(&mut frame, self.game, self.show_inventory, self.blink_on);
        }

        vec![frame.into_geometry()]
    }
}
