use std::sync::atomic::{AtomicBool, Ordering};
use std::vec::Vec;
use std::time::{Duration, SystemTime};
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use scap::capturer::{Area, Capturer, Options};
use scap::frame::Frame;
use crate::sender::ScapError::{ScapNotSupported, ScapPermissionDenied};
use crate::util::{Header, CHECK_STOP, CHUNK_SIZE};

#[derive(Debug)]
enum ScapError {
    ScapNotSupported,
    ScapPermissionDenied,
}
fn scap_init() -> Result<(), ScapError> {
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
    Ok(())
}

fn create_capturer(area: Area) -> Capturer{
    let targets = scap::get_targets();
    let options = Options {
        fps: 30,
        show_cursor: true,
        show_highlight: false,
        targets,
        // excluded_targets: None,
        output_type: scap::frame::FrameType::BGRAFrame,
        output_resolution: scap::capturer::Resolution::_720p,
        // source_rect: Some(Area {
        //     origin: Point { x: 0.0, y: 0.0 },
        //     size: Size {
        //         width: 2000.0,
        //         height: 1000.0,
        //     },
        // }),
        source_rect: Some(area),
        ..Default::default()
    };
    Capturer::new(options)
}

fn from_bgra_to_rgba(mut frame: Vec<u8>) -> Vec<u8> {
    for chunk in frame.chunks_exact_mut(4) {
        chunk.swap(0, 2); // Scambia il canale rosso (0) con quello blu (2)
    }
    frame
}

pub fn start(ip_addr: String, stop_request: Arc<AtomicBool>, area: Arc<Mutex<Area>>) {
    let mut stream = TcpStream::connect(format!("{}:8080", ip_addr)).unwrap();
    println!("Connection successed");
    let mut frame_number = 0;
    let _fps60 = Duration::new(1, 0) / 60;

    scap_init().unwrap();
    let mut capturer:Capturer = create_capturer(area.lock().unwrap().clone());

    loop {
        frame_number = frame_number + 1;
        
        capturer.start_capture();
        let next_frame = capturer.get_next_frame().unwrap();
        capturer.stop_capture();

        if let Frame::BGRA(mut frame) = next_frame {
            // Send header
            let header = Header::new(frame_number, frame.data.len() as u32, frame.width as u32, frame.height as u32);
            let encoded_header: Vec<u8> = bincode::serialize(&header).unwrap();

            if frame_number % CHECK_STOP == 0 {
                if stop_request.load(Ordering::Relaxed) == true {
                    println!("Received stop request from gui");
                    break;
                }
                capturer = create_capturer(area.lock().unwrap().clone());
            }

            if let Err(e) = stream.write(&encoded_header) {
                println!("Server closed: {}", e);
                break;
            }


            println!("Header sent {} {}", header.frame_number, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());
            frame.data = from_bgra_to_rgba(frame.data);

            // Send frame
            let frame_pad = CHUNK_SIZE - (frame.data.len() as u32 % CHUNK_SIZE);
            if frame_pad < CHUNK_SIZE {
                for _ in 0..frame_pad {
                    frame.data.push(0);
                }
            }
            if let Err(e) = stream.write_all(&frame.data) {
                println!("Server closed: {}", e);
                break;
            }
            println!("Frame sent {} {}", header.frame_number, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());
        } else {
            println!("Sender side: the captured frame format is not supported.");
            break;
        }
    }
    println!("Sender terminated");
}