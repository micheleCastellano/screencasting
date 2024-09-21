use std::fs::File;
use std::thread;
use std::io::Read;
use std::io::Write;
use std::time::Duration;
use std::net::{TcpListener};
use std::sync::mpsc::Sender;
use eframe::egui::{Context, TextureHandle};

pub fn start(channel_s: Sender<TextureHandle>, ctx: Context) {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server in ascolto su 127.0.0.1:8080");
    let (mut stream, _) = listener.accept().unwrap();

    let chunk_size = 64 * 1024;
    let mut buffer = vec![0u8; chunk_size];
    let mut frame_vec = Vec::new();
    thread::sleep(Duration::from_millis(1000));

    for _ in 0..100 {
        match stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read == 0 => {
                println!("Connection closed");
                break;
            }
            Ok(bytes_read) => {
                println!("Received chunk of {} bytes", bytes_read);
                frame_vec.extend_from_slice(&buffer);
            }
            Err(e) => panic!("Error reading from socket: {:?}", e),
        }
    }
    let mut file = File::create("frame_receiver.txt").unwrap();
    for item in &frame_vec {
        writeln!(file, "{}", item).unwrap();
    }
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
        eframe::egui::ColorImage::from_rgba_premultiplied(size, &pixels),
        eframe::egui::TextureOptions::default(),
    );
    channel_s.send(texture).unwrap();
    println!("receiver finished");
}