use std::{fs};
use std::io::Read;
use std::net::{TcpListener};
use std::sync::mpsc::{Receiver, Sender};
use std::time::SystemTime;
use eframe::egui::Context;
use std::process::Command;
use tokio::runtime::Runtime;
use crate::capturer::Frame;
use crate::util::{Header, CHUNK_SIZE, Message, MessageType};

const PATH: &str = "./tmp";

fn make_video() {
    // ffmpeg -framerate 20 -i %d_img.jpeg -c:v libx264 -r 20 01_output.mp4
    // ffmpeg -framerate 1 -i happy%d.jpg -c:v libx264 -r 30 -pix_fmt yuv420p output.mp4
    // ffmpeg -framerate 1 -pattern_type glob -i '*.jpg' -c:v libx264 -r 30 -pix_fmt yuv420p output.mp4

    let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

    let ffmpeg_command = Command::new("ffmpeg")
        .arg("-framerate")
        .arg("15")
        .arg("-i")
        .arg("./tmp/%d_img.jpeg")
        .arg("-c:v")
        .arg("libx264")
        // .arg("-pix_fmt")
        // .arg("yuv420p")
        .arg("-r")
        .arg("15")
        .arg(format!("./0_video.mp4_{ts}.mp4"))
        .output()
        .expect("failed to execute ffmpeg");


    println!("FFmpeg status: {:?}", ffmpeg_command);
}

pub fn start(frame_s: Sender<Frame>, msg_r: Receiver<Message>, ctx: Context, mut save_option: bool) {

    //initialization
    let tokio_rt = Runtime::new().unwrap();
    let ip_addr = local_ip_address::local_ip().unwrap().to_string();
    let listener = TcpListener::bind(format!("{ip_addr}:8080")).unwrap();
    println!("Server listening to {ip_addr}:8080");
    fs::create_dir_all(PATH).unwrap(); // useful to record the streaming
    let (mut stream, _) = listener.accept().unwrap();
    let mut header_buffer = [0; std::mem::size_of::<Header>()];
    let mut frame_buffer = [0; CHUNK_SIZE as usize];

    'streaming: loop {
        //manage messages from gui
        if let Ok(msg) = msg_r.try_recv() {
            match msg.message_type {
                MessageType::Stop => {
                    println!("received stop request from gui");
                    break 'streaming;
                }
                MessageType::Save => {
                    save_option = msg.save_option;
                }
                _ => {}
            }
        }

        // Read header
        if let Err(e) = stream.read_exact(&mut header_buffer) {
            println!("Connection closed: {e}");
            break 'streaming;
        }
        let mut header: Header = bincode::deserialize(&header_buffer).expect("error deserializing header");
        println!("Header received {} {}", header.frame_number, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());

        // Read data
        let mut data = Vec::with_capacity(header.len as usize);
        while header.len > 0 {
            if let Err(e) = stream.read_exact(&mut frame_buffer) {
                println!("connection closed: {e}");
                break 'streaming;
            }
            let end;
            if CHUNK_SIZE > header.len {
                end = header.len;
                header.len = 0;
            } else {
                end = CHUNK_SIZE;
                header.len = header.len - CHUNK_SIZE;
            }
            for i in 0..end {
                data.push(frame_buffer[i as usize]);
            }
        }
        println!("Frame received {} {}", header.frame_number, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());

        // Save frame
        let frame_number = header.frame_number;
        match image::RgbImage::from_raw(header.frame_width, header.frame_height, data.clone()) {
            None => { println!("error occurs converting frame {frame_number} in RgbImage"); }
            Some(rgb) => {
                tokio_rt.spawn(async move {
                    if let Err(e) = rgb.save(format!("{PATH}/{frame_number}_img.jpeg")) {
                        println!("Error occurs saving image {frame_number}: {e}");
                    }
                });
            }
        }

        // Send frame to gui
        let frame = Frame::new(header.frame_width, header.frame_height, data);
        if let Err(e) = frame_s.send(frame) {
            println!("Impossible sending frame via channel: {:?}", e);
            break 'streaming;
        }

        ctx.request_repaint();
    }

    if save_option {
        make_video();
    }

    if let Err(e) = fs::remove_dir_all(PATH) {
        println!("impossible remove dir tmp: {e}");
    }
    println!("Receiver terminated.");
}