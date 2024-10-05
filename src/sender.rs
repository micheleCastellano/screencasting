use std::vec::Vec;
use std::time::{SystemTime};
use std::net::TcpStream;
use std::io::Write;
use std::sync::mpsc::Receiver;
use std::thread;
use crate::capturer;
use crate::capturer::{Area, capture, Frame};
use crate::util::{Header, CHUNK_SIZE, Message, MessageType};

pub fn start(ip_addr: String, mut area: Area, msg_r: Receiver<Message>) {
    //initialization
    let mut stream;
    match TcpStream::connect(format!("{}:8080", ip_addr)) {
        Ok(s) => { stream = s; }
        Err(e) => {
            println!("Impossible connecting to {ip_addr}: {e}");
            return;
        }
    }
    println!("Connection successed");
    let mut frame_number = 0;

    let mut cpt = capturer::create();

    // streaming
    'streaming: loop {
        thread::sleep(capturer::FPS_SLEEP);

        // manage messages from gui
        if let Ok(msg) = msg_r.try_recv() {
            match msg.message_type {
                MessageType::Stop => {
                    println!("received stop request from gui");
                    break 'streaming;
                }
                MessageType::Area => {
                    area = msg.area;
                }
                _ => {}
            }
        }

        frame_number = frame_number + 1;

        let data = capture(&mut cpt);
        assert_ne!(data.len(), 0);
        let frame = Frame::new(cpt.width() as u32, cpt.height() as u32, data);
        let mut frame = capturer::u8x4_crop(frame,&area);
        frame.data = capturer::from_bgra_to_rgb(frame.data);

        // Send header
        let header = Header::new(frame_number, frame.data.len() as u32, frame.w, frame.h);
        let encoded_header: Vec<u8> = bincode::serialize(&header).unwrap();

        if let Err(e) = stream.write(&encoded_header) {
            println!("Connection closed: {}", e);
            break 'streaming;
        }
        println!("Header sent {} {}", header.frame_number, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());

        // Send frame
        let frame_pad = CHUNK_SIZE - (frame.data.len() as u32 % CHUNK_SIZE);
        println!("frame pad {}", frame_pad);
        if frame_pad < CHUNK_SIZE {
            for _ in 0..frame_pad {
                frame.data.push(0);
            }
        }
        if let Err(e) = stream.write_all(&frame.data) {
            println!("Server closed: {}", e);
            break 'streaming;
        }
        println!("Frame sent {} {}", header.frame_number, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());
    }
    println!("Sender terminated");
}