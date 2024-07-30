use std::sync::mpsc::Receiver;
use eframe::{egui};
use eframe::egui::load::SizedTexture;
use egui_extras::{install_image_loaders};
use image::{ImageFormat};

//#[derive(Default)]
struct MyEguiApp {
    receiver: Receiver<Vec<u8>>,
    texture: Option<egui::TextureHandle>,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>, receiver: Receiver<Vec<u8>>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        MyEguiApp { receiver, texture: None }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(image_data) = self.receiver.recv() {
            let image = image::load_from_memory_with_format(image_data.as_slice(), ImageFormat::Png).expect("Failed to load image");
            let size = [image.width() as usize, image.height() as usize];
            let pixels = image.to_rgba8().into_raw();
            let texture = ctx.load_texture(
                "current_image",
                egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
                egui::TextureOptions::default(),
            );
            self.texture = Some(texture);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("This is an image:");
            if let Some(texture) = &self.texture {
                ui.image(SizedTexture::new(texture, texture.size_vec2()));
            }
        });
    }
}
pub fn launch(receiver: Receiver<Vec<u8>>) {
    let mut vp = egui::ViewportBuilder::default();
    let vp = vp.with_fullscreen(true);
    let native_options = eframe::NativeOptions {
        viewport: vp,
        ..Default::default()
    };
    let _ = eframe::run_native("My egui App", native_options, Box::new(|cc| {
        install_image_loaders(&cc.egui_ctx);
        Ok(Box::new(MyEguiApp::new(cc, receiver)))
    }
    ));
}
