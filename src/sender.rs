use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;
use std::io::Write;
use std::net::TcpStream;
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
    let mut stream = TcpStream::connect("192.168.10.114:8080").unwrap();
    println!("Connesso al server!");
    let frame_number = 0;
    let fps60 = Duration::new(1, 0) / 60;

    loop {
        thread::sleep(fps60);
        let (mut frame, w, h) = capture_screen(fps60).unwrap();

        // Scambia i canali di colore
        for chunk in frame.chunks_exact_mut(4) {
            chunk.swap(0, 2); // Scambia il canale rosso (0) con quello blu (2)
        }

        let header = Header::new(frame_number, frame.len() as u32, w as u32, h as u32);

        let encoded: Vec<u8> = bincode::serialize(&header).unwrap();

        stream.write(&encoded).unwrap();
        stream.write_all(&frame).unwrap();
    }
}