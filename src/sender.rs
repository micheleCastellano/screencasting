use std::vec::Vec;
use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;
use std::io::Write;
use std::net::TcpStream;
use image::codecs::jpeg::JpegEncoder;
use image::{ColorType};
use crate::CHUNK_SIZE;
use crate::util::Header;


pub fn capture_screen(delay: Duration) -> Result<(Vec<u8>, usize, usize), Box<dyn std::error::Error>> {
    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

    loop {
        match capturer.frame() {
            Ok(frame) => {
                return Ok((frame.to_vec(), capturer.width(), capturer.height()));
            }
            Err(error) => {
                if error.kind() == WouldBlock {
                    thread::sleep(delay);
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
    let frame_number = 0;
    let fps60 = Duration::new(1, 0) / 60;
    let mut i = 0;
    while i < 4 {
        i = i + 1;
        thread::sleep(fps60);
        let (mut frame, w, h) = capture_screen(fps60).unwrap();

        // Scambia i canali di colore (rosso e blu): cicla su un vettore di 4MB
        for chunk in frame.chunks_exact_mut(4) {
            chunk.swap(0, 2); // Scambia il canale rosso (0) con quello blu (2)
        }

        let mut jpeg = vec![];
        let mut encoder = JpegEncoder::new_with_quality(&mut jpeg, 60);
        encoder.encode(&frame, w as u32, h as u32, ColorType::Rgba8).unwrap();

        // Send header
        let header = Header::new(frame_number, jpeg.len(), w, h);
        let mut encoded: Vec<u8> = bincode::serialize(&header).unwrap();
        encoded.extend_from_slice(&[0; CHUNK_SIZE - size_of::<Header>()]);
        stream.write(&encoded).expect("Error stream write");

        // Send jpeg
        let jpeg_pad = CHUNK_SIZE - (jpeg.len() % CHUNK_SIZE);
        if jpeg_pad < CHUNK_SIZE {
            for _ in 0..jpeg_pad {
                jpeg.push(0);
            }
        }
        stream.write_all(&jpeg).expect("Error stream write all");
    }
}