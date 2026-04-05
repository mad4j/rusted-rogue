const FAVICON_BYTES: &[u8] = include_bytes!("../../resources/favicon-32x32.png");

pub fn window_icon() -> Option<iced::window::Icon> {
    iced::window::icon::from_file_data(FAVICON_BYTES, None).ok()
}
