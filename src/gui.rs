use std::collections::VecDeque;
use eframe::egui;
use egui_extras::{install_image_loaders};

#[derive(Default)]
struct MyEguiApp {
    image_paths : VecDeque<String>
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            // let mut path = PathBuf::new();
            // path. push(r"C:\Users\Michele\progetti\screencasting_app\images5");
            // let image_paths = get_image_paths_from_directory(path);
            // for image_path in image_paths.into_iter(){
            //     ui.image("file://{image_path}");
            //     thread::sleep(Duration::from_secs(2));
            //     println!("cambio");
            // }
            //     ui.image("file://C:/Users/Michele/progetti/screencasting_app/images5/1.png");



        });
    }
}
pub fn launch() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native("My egui App", native_options, Box::new(|cc| {
        install_image_loaders(&cc.egui_ctx);
        Ok(Box::new(MyEguiApp::new(cc)))
    }
    ));
}
