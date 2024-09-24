use std::io::{Read};
use std::net::{TcpListener};
use std::sync::mpsc::Sender;
use eframe::egui::Context;
use image::{ImageFormat};
use crate::CHUNK_SIZE;
use crate::util::{Header, ChannelFRAME};

pub fn start(channel_s: Sender<ChannelFRAME>, _ctx: Context) {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server listening to 127.0.0.1:8080");
    let (mut stream, _) = listener.accept().unwrap();
    let mut chunk = [0; CHUNK_SIZE];
    loop {
        // Read header
        stream.read_exact(&mut chunk).expect("error reading header");
        let mut header: Header = bincode::deserialize(&chunk[0..size_of::<Header>()]).expect("error deserializing header");
        println!("{:?}", header);
        // Read jpeg
        let mut jpeg = Vec::with_capacity(header.len);
        while header.len > 0 {
            match stream.read_exact(&mut chunk) {
                Ok(_) => {
                    let end;
                    if CHUNK_SIZE > header.len {
                        end = header.len;
                        header.len = 0;
                    } else {
                        end = CHUNK_SIZE;
                        header.len = header.len - CHUNK_SIZE;
                    }
                    for i in 0..end {
                        jpeg.push(chunk[i]);
                    }
                }
                Err(e) => {
                    println!("Error strem read jpeg - {}", e);
                    return;
                }
            }
        }
        let image = image::load_from_memory_with_format(&jpeg, ImageFormat::Jpeg).expect("Error: load_from_memory_wuth_format");
        let rgba = image.to_rgba8();
        let channel_jpeg = ChannelFRAME::new(header.image_width, header.image_height, rgba.into_raw());
        channel_s.send(channel_jpeg).expect("error sending channel_jpeg");
        _ctx.request_repaint();
    }
}