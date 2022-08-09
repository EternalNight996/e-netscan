// Hide terminal window on windows
#![windows_subsystem = "windows"]
extern crate e_libscanner;
mod configs;
#[path = "./datas.rs"]
mod datas;
mod ui;

use iced::{window, Application, Settings};
use ui::App;

const APP_NAME: &'static str = "e-netscan";
/// main app
pub fn main() -> Result<(), String> {
    #[cfg(debug_assertions)]
    let verbose = 5;
    #[cfg(not(debug_assertions))]
    let verbose = 3;
    configs::logger::init_logger(APP_NAME, verbose);
    gui_run()
}

/// create window icon
fn window_icon(ico_bytes: &[u8]) -> Option<window::Icon> {
    let raw_image = image::load_from_memory(ico_bytes).ok()?;
    let loaded_image = raw_image.as_rgba8().map(|rgba_image| {
        (
            rgba_image.width(),
            rgba_image.height(),
            rgba_image
                .pixels()
                .fold(Vec::<u8>::new(), |mut pixels, next| {
                    pixels.extend_from_slice(&next.0);
                    pixels
                }),
        )
    })?;
    window::Icon::from_rgba(loaded_image.2, loaded_image.0, loaded_image.1).ok()
}

/// run gui;
pub fn gui_run() -> Result<(), String> {
    // gui interface
    let mut settings = Settings::default();
    // window config
    settings.window = window::Settings {
        // window size
        size: (700, 500),
        // The initial position of the window.
        position: window::Position::Centered,
        // The minimum size of the window.
        min_size: None,
        // The maximum size of the window.
        max_size: None,
        // Whether the window should be resizable or not.
        resizable: true,
        // Whether the window should have a border, a title bar, etc. or not.
        decorations: true,
        // Whether the window should be transparent.
        transparent: false,
        // Always keep top
        always_on_top: false,
        // Set icon
        icon: window_icon(include_bytes!("../static/icon.ico")),
    };
    // Font size
    settings.default_text_size = 20u16;
    // The bytes of the font that will be used by default.
    settings.default_font = Some(include_bytes!("../static/ttfs/font.ttf"));
    // Whether the [`Application`] should exit when the user requests the
    settings.exit_on_close_request = false;
    // If enabled, spread text workload in multiple threads when multiple cores
    settings.text_multithreading = true;
    // Enabling it can produce a smoother result in some widgets, like the [`Canvas`]
    settings.antialiasing = true;
    match App::run(settings) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
