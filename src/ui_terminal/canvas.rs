use iced::widget::canvas;
use iced::{Color, Point, Theme};

use crate::game_loop::GameLoop;

use super::help::render_help_page;
use super::renderer::render_game;
use super::Message;

pub(super) struct GameCanvas<'a> {
    pub(super) game: &'a GameLoop,
    pub(super) show_help: bool,
    pub(super) help_page: usize,
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

        if self.show_help {
            render_help_page(&mut frame, self.help_page);
        } else {
            render_game(&mut frame, self.game);
        }

        vec![frame.into_geometry()]
    }
}
