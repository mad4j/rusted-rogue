const FAVICON_BYTES: &[u8] = include_bytes!("../../resources/icon.ico");

pub fn window_icon() -> Option<iced::window::Icon> {
    iced::window::icon::from_file_data(FAVICON_BYTES, None).ok()
}
