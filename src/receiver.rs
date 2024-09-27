use std::io::{Read};
use std::net::{TcpListener};
use std::sync::mpsc::Sender;
use eframe::egui::Context;
use crate::CHUNK_SIZE;
use crate::util::{Header, ChannelFrame};

pub fn start(channel_s: Sender<ChannelFrame>, _ctx: Context) {
    let ip_addr = local_ip_address::local_ip().unwrap().to_string();
    let listener = TcpListener::bind(format!("{ip_addr}:8080")).unwrap();
    println!("Server listening to {ip_addr}:8080");
    let (mut stream, _) = listener.accept().unwrap();
    let mut header_buffer = [0; size_of::<Header>()];
    let mut frame_buffer = [0; CHUNK_SIZE as usize];
    loop {
        // Read header
        if let Err(e) = stream.read_exact(&mut header_buffer) {
            println!("Sender closed: {e}");
            break;
        }
        let mut header: Header = bincode::deserialize(&header_buffer).expect("error deserializing header");

        // Read frame
        let mut frame = Vec::with_capacity(header.len as usize);
        while header.len > 0 {
            if let Err(e) = stream.read_exact(&mut frame_buffer) {
                println!("Sender closed: {e}");
                break;
            }

            let end;
            if CHUNK_SIZE > header.len {
                end = header.len;
                header.len = 0;
            } else {
                end = CHUNK_SIZE ;
                header.len = header.len - CHUNK_SIZE;
            }
            for i in 0..end {
                frame.push(frame_buffer[i as usize]);
            }
        }

        let channel_frame = ChannelFrame::new(header.frame_width, header.frame_height, frame);
        if let Err(e) = channel_s.send(channel_frame) {
            println!("Impossible sending frame via channel: {:?}", e);
            break;
        }
        _ctx.request_repaint();
    }
    println!("Receiver terminated.")
}