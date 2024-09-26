use crate::util::Header;
use local_ip_address;
use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::io::Write;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

pub fn capture_screen(
    delay: Duration,
) -> Result<(Vec<u8>, usize, usize), Box<dyn std::error::Error>> {
    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let mut last_frame: Option<Vec<u8>> = None; // Salva l'ultimo frame

    loop {
        match capturer.frame() {
            Ok(frame) => {
                last_frame = Some(frame.to_vec()); // Salva il frame attuale
                return Ok((
                    last_frame.clone().unwrap(),
                    capturer.width(),
                    capturer.height(),
                ));
            }
            Err(error) => {
                if error.kind() == WouldBlock {
                    if let Some(ref frame) = last_frame {
                        return Ok((frame.clone(), capturer.width(), capturer.height()));
                        // Restituisci l'ultimo frame valido
                    }
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
    let my_ip = local_ip_address::local_ip().unwrap();
    let mut stream = TcpStream::connect(format!("{:?}:8080", my_ip)).unwrap();
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

        let header = Header::new(frame_number, frame.len(), w, h);

        let encoded: Vec<u8> = bincode::serialize(&header).unwrap();

        stream.write(&encoded).unwrap();
        stream.write_all(&frame).unwrap();
    }
}

