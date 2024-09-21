// use image::GenericImageView;
use scrap::{Capturer, Display};
use std::io::BufWriter;
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;
use std::io::Write;
use std::fs::File;
use std::net::TcpStream;

fn capture_screen() -> Result<(Vec<u8>, usize, usize), Box<dyn std::error::Error>> {
    let one_second = Duration::new(1, 0);
    let one_frame = one_second / 60;
    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (w, h) = (capturer.width(), capturer.height());

    loop {
        match capturer.frame() {
            Ok(buffer) => {
                let frame = buffer.to_vec();

                // Converte il frame in un'immagine RGBA
                let buffer: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
                    image::ImageBuffer::from_raw(w as u32, h as u32, frame)
                        .expect("Failed to convert frame to ImageBuffer");


                // Salva l'immagine come PNG in memoria
                let mut png_data = Vec::new();
                {
                    let writer = BufWriter::new(&mut png_data);
                    let mut encoder = png::Encoder::new(writer, buffer.width(), buffer.height());
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);
                    let mut writer = encoder.write_header().expect("Failed to write PNG header");
                    writer
                        .write_image_data(&buffer)
                        .expect("Failed to write PNG data");
                }

                return Ok((png_data, w, h));
            }
            Err(error) => {
                if error.kind() == WouldBlock {
                    // Keep spinning.
                    thread::sleep(one_frame);
                    continue;
                } else {
                    return Err(Box::new(error));
                }
            }
        };
    }
}
pub fn send() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    println!("Connesso al server!");

    let (v, _w, _h) = capture_screen().unwrap();
    let chunk_size = 64 * 1024;
    let mut offset = 0;

    let mut file = File::create("frame_sender.txt").unwrap();
    for item in &v {
        writeln!(file, "{}", item).unwrap();
    }

    while offset < v.len() {
        let end = std::cmp::min(offset + chunk_size, v.len());
        let chunk = &v[offset..end];
        stream.write_all(chunk).unwrap();
        offset = end;
    }
    println!("sender finished");
}