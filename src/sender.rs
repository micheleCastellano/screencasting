use std::vec::Vec;
use std::thread;
use std::time::Duration;
use std::io::Write;
use std::net::{TcpStream};
use scap::capturer::{Capturer, Options};
use scap::frame::Frame;
use crate::CHUNK_SIZE;
use crate::sender::ScapError::{ScapNotSupported, ScapPermissionDenied};
use crate::util::Header;
#[derive(Debug)]
enum ScapError {
    ScapNotSupported,
    ScapPermissionDenied,
}
fn scap_init() -> Result<Capturer, ScapError> {
    // Check if the platform is supported
    let supported = scap::is_supported();
    if !supported {
        println!("❌ Platform not supported");
        return Err(ScapNotSupported);
    } else {
        println!("✅ Platform supported");
    }

    // Check if we have permission to capture screen
    // If we don't, request it.
    if !scap::has_permission() {
        println!("❌ Permission not granted. Requesting permission...");
        if !scap::request_permission() {
            println!("❌ Permission denied");
            return Err(ScapPermissionDenied);
        }
    }
    println!("✅ Permission granted");
    let targets = scap::get_targets();
    let options = Options {
        // fps: 60,
        show_cursor: true,
        show_highlight: true,
        targets,
        // excluded_targets: None,
        output_type: scap::frame::FrameType::BGRAFrame,
        // output_resolution: scap::capturer::Resolution::_720p,
        // source_rect: Some(Area {
        //     origin: Point { x: 0.0, y: 0.0 },
        //     size: Size {
        //         width: 2000.0,
        //         height: 1000.0,
        //     },
        // }),
        ..Default::default()
    };
    Ok(Capturer::new(options))
}

fn from_bgra_to_rgba(mut frame: Vec<u8>) -> Vec<u8>{
    for chunk in frame.chunks_exact_mut(4) {
        chunk.swap(0, 2); // Scambia il canale rosso (0) con quello blu (2)
    }
    frame
}
pub fn send(ip_addr: String) {
    let mut stream = TcpStream::connect(format!("{}:8080", ip_addr)).unwrap();
    println!("Connection successed");
    let frame_number = 0;
    let fps60 = Duration::new(1, 0) / 60;

    let mut capturer = scap_init().unwrap();
    capturer.start_capture();

    loop {
        thread::sleep(fps60);
        let next_frame = capturer.get_next_frame().unwrap();
        if let Frame::BGRA(mut frame) =  next_frame{

            // Send header
            let header = Header::new(frame_number, frame.data.len(), frame.width as usize, frame.height as usize);
            let encoded_header: Vec<u8> = bincode::serialize(&header).unwrap();
            if let Err(e) = stream.write(&encoded_header){
                println!("Server closed: {}", e);
                break;
            }
            frame.data = from_bgra_to_rgba(frame.data);

            // Send jpeg
            let frame_pad = CHUNK_SIZE - (frame.data.len() % CHUNK_SIZE);
            if frame_pad < CHUNK_SIZE {
                for _ in 0..frame_pad {
                    frame.data.push(0);
                }
            }
            if let Err(e) = stream.write_all(&frame.data){
                println!("Server closed: {}", e);
                break;
            }
        } else {
            println!("Sender side: the captured frame format is not supported.");
            break;
        }
    }
    capturer.stop_capture();
    println!("Sender terminated");
}