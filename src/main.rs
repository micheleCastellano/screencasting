mod gui;
mod sender;
mod receiver;
// mod gui_example;

use gui::EframeApp;
// use gui_example::UserInterface;

#[tokio::main]
async fn main() {

    let native_options = eframe::NativeOptions::default();

    let _ = eframe::run_native(
        "Screencastin app",
        native_options,
        Box::new(|cc| Ok(Box::new(EframeApp::new(cc)))));
}