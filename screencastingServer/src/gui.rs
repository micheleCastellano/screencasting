use std::fs::File;
use eframe::egui;
use std::thread;
use eframe::egui::load::SizedTexture;
use egui_extras::install_image_loaders;
use image::{ImageFormat, ImageResult};
use std::io::Read;
use std::io::Write;
//use std::sync::mpsc::Receiver;
use tokio::io::AsyncReadExt;
use std::net::TcpStream;
use std::time::Duration;
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
        ctx.request_repaint();
        let chunk_size = 64 * 1024;
        let mut buffer = vec![0u8; chunk_size];

        let mut frame_vec = Vec::new();


        thread::sleep(Duration::from_millis(1000));
//loop {
        for _ in 0..100 {
            println!("prima del match");
            match self.socket.read(&mut buffer) {
                Ok(bytes_read) if bytes_read == 0 => {
                    println!("Connection closed");
                    break;
                }
                Ok(bytes_read) => {
                    // Process the chunk of data here
                    println!("Received chunk of {} bytes", bytes_read);
                    // Append only the portion of the buffer that was filled
                    //frame_vec.extend_from_slice(&buffer[..bytes_read]);
                    frame_vec.extend_from_slice(&buffer);
                }
                Err(e) => panic!("Error reading from socket: {:?}", e),
            }
            println!("dopo il match")
        }
        //}
        println!("frame vec size: {}", frame_vec.len());

        let mut file = File::create("frame_receiver.txt").unwrap();
        for item in &frame_vec {
            writeln!(file, "{}", item).unwrap();
        }

        /*
        let n_bytes_filled = self.socket.read_exact(&mut self.buffer_image);
        println!("Buffer len: {}", self.buffer_image.len());
        println!("Image as Slice: {:?}", self.buffer_image.as_slice());
        */
        //.expect("something went wrong reading socket");
        //if let Ok(image_data) = self.receiver.recv() {
        //let image = image::load_from_memory_with_format(image_data.as_slice(), ImageFormat::Png)
        let image = image::load_from_memory(frame_vec.as_slice()).expect("Errore nel caricare l'immagine!");
        let size = [image.width() as usize, image.height() as usize];

// Converti l'immagine da BGRA a RGBA se necessario
        let mut pixels = image.to_rgba8().into_raw();

// Scambia i canali di colore (rosso e blu)
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2); // Scambia il canale rosso (0) con quello blu (2)
        }

// Ricrea l'immagine con i colori corretti e salvala
        let corrected_image = image::RgbaImage::from_raw(image.width(), image.height(), pixels.clone())
            .expect("Errore nella creazione dell'immagine corretta!");
        corrected_image.save("immagine.png").expect("failed to save corrected image");

// Carica la texture per la visualizzazione su egui
        let texture = ctx.load_texture(
            "current_image",
            egui::ColorImage::from_rgba_premultiplied(size, &pixels),
            egui::TextureOptions::default(),
        );

        self.texture = Some(texture);

// Visualizzazione su egui
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Screenshot appena fatto:");
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
