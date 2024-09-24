use std::io::{Read};
use std::net::{TcpListener};
//use std::sync::mpsc::Sender;
use crossbeam::channel::Sender;
use eframe::egui::Context;
use crate::CHUNCK_SIZE;
use crate::util::{Header, ChannelFrame};

pub fn start(channel_s: Sender<ChannelFrame>, ctx: Context) {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Server listening to 127.0.0.1:8080");
    let (mut stream, _) = listener.accept().unwrap();

    let mut header_buffer = [0; size_of::<Header>()];
    let mut frame_buffer = [0; CHUNCK_SIZE];

    loop {
        //read header
        stream.read(&mut header_buffer).expect("error reading header");
        let mut header: Header = bincode::deserialize(&header_buffer[..]).expect("error deserializing header");
        let mut frame = Vec::with_capacity(header.len as usize);

        //read frame
        while header.len > 0 {
            match stream.read(&mut frame_buffer) {
                Ok(0) => {
                    println!("Connection closed");
                    return;
                }
                Ok(bytes_read) => {
                    header.len = header.len - bytes_read as u32;

                    for i in 0..bytes_read {
                        frame.push(frame_buffer[i]);
                    }
                }
                Err(e) => panic!("Error reading from socket: {:?}", e),
            }
        }
        let channel_frame = ChannelFrame::new(header.image_width, header.image_height,frame);
        //channel_s.send(channel_frame).expect("error sending channel_frame");
        match channel_s.try_send(channel_frame) {
            Ok(_) => (),
            Err(e) => println!("frame dropped! {}", e),
        }
        ctx.request_repaint();
    }
}