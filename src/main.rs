mod gui;
mod sender;
mod receiver;
mod util;
mod capturer;

use std::default::Default;
use eframe::egui::ViewportBuilder;
use gui::EframeApp;

fn main() {
    let viewport = ViewportBuilder {
        transparent:Some(true),
        // maximized: Some(true),
        ..Default::default()
    };
    let native_options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Screencastin app",
        native_options,
        Box::new(|cc| Ok(Box::new(EframeApp::new(cc)))));
}