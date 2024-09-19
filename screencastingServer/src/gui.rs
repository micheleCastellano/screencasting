use eframe::egui;
use eframe::egui::load::SizedTexture;
use egui_extras::install_image_loaders;
use image::ImageFormat;
use std::io::Read;
//use std::sync::mpsc::Receiver;
use tokio::io::AsyncReadExt;
use std::net::TcpStream;
//use tokio::net::TcpStream;

//#[derive(Default)]
struct MyEguiApp {
    //receiver: Receiver<Vec<u8>>,
    socket: TcpStream,
    texture: Option<egui::TextureHandle>,
    //buffer_image: &'static mut [u8],
    buffer_image: Vec<u8>,
}

impl MyEguiApp {
    //fn new(cc: &eframe::CreationContext<'>, receiver: Receiver<Vec<u8>>) -> Self {
    fn new(cc: &eframe::CreationContext, socket: TcpStream) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        MyEguiApp {
            //receiver,
            socket,
            texture: None,
            buffer_image: Vec::<u8>::with_capacity(873310),
        }
    }
}

impl eframe::App for MyEguiApp{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let chunk_size = 64 * 1024;
        let mut buffer = vec![0u8; chunk_size];

        loop {
            match self.socket.read_exact(&mut buffer) {
                Ok(_) => {
                    // Process the chunk of data here
                    //println!("Received chunk of {} bytes", buffer.len());
                    println!("{:?}", buffer);
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    println!("Connection closed");
                    break;
                }
                Err(e) => panic!(":(")
            }
        }


        let n_bytes_filled = self.socket.read_exact(&mut self.buffer_image);
        println!("Buffer len: {}", self.buffer_image.len());
        println!("Image as Slice: {:?}", self.buffer_image.as_slice());
        //.expect("something went wrong reading socket");
        //if let Ok(image_data) = self.receiver.recv() {
        //let image = image::load_from_memory_with_format(image_data.as_slice(), ImageFormat::Png)
        let image = image::load_from_memory_with_format(self.buffer_image.as_slice(), ImageFormat::Png)
            .expect("Failed to load image");
        let size = [image.width() as usize, image.height() as usize];
        let pixels = image.to_rgba8().into_raw();
        let texture = ctx.load_texture(
            "current_image",
            egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
            egui::TextureOptions::default(),
        );
        self.texture = Some(texture);
        //}

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("This is an image:");
            if let Some(texture) = &self.texture {
                ui.image(SizedTexture::new(texture, texture.size_vec2()));
            }
        });
    }
}

//pub fn launch(receiver: Receiver<Vec<u8>>) {
pub fn launch(socket: TcpStream) {
    let vp = egui::ViewportBuilder::default();
    let vp = vp.with_fullscreen(true);
    let native_options = eframe::NativeOptions {
        viewport: vp,
        ..Default::default()
    };
    let app = eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| {
            install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MyEguiApp::new(cc, socket)))
        }),
    );
}
