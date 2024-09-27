use std::fs;
use std::io::{Read};
use std::net::{TcpListener};
use std::sync::mpsc::Sender;
use std::time::SystemTime;
use eframe::egui::Context;
use image::{RgbaImage};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use crate::util::{Header, ChannelFrame, CHECK_STOP, CHUNK_SIZE};

const PATH: &str = "./tmp";

pub fn start(channel_s: Sender<ChannelFrame>, _ctx: Context, stop_request: Arc<Mutex<bool>>) {
    let tokio_rt = Runtime::new().unwrap();
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
        println!("Header Received {} {}", header.frame_number, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());

        if header.frame_number % CHECK_STOP == 0 {
            let mutex = stop_request.lock().unwrap();
            if *mutex == true {
                println!("Received stop request from gui");
                break;
            }
        }

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
                end = CHUNK_SIZE;
                header.len = header.len - CHUNK_SIZE;
            }
            for i in 0..end {
                frame.push(frame_buffer[i as usize]);
            }
        }
        println!("Frame Received {} {}", header.frame_number, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());

        let frame_number = header.frame_number;
        match RgbaImage::from_raw(header.frame_width, header.frame_height, frame.clone()) {
            None => { println!("error occurs converting frame {frame_number} in RgbaImage"); }
            Some(rgba) => {
                tokio_rt.spawn(async move {
                    match fs::create_dir_all(PATH) {
                        Ok(_) => {
                            if let Err(e) = rgba.save(format!("{PATH}/{frame_number}_img.jpeg")) {
                                println!("Error occurs saving image {frame_number}: {e}");
                            }
                        }
                        Err(e) => { println!("error creating directory tmp : {e}"); }
                    }
                });
            }
        }


        let channel_frame = ChannelFrame::new(header.frame_width, header.frame_height, frame);
        if let Err(e) = channel_s.send(channel_frame) {
            println!("Impossible sending frame via channel: {:?}", e);
            break;
        }
        _ctx.request_repaint();
    }
    make_video();
    println!("Receiver terminated.");
}

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
    if let Err(e) = fs::remove_dir_all("./tmp") {
        println!("impossible remove dir tmp: {e}");
    }
}