use std::vec::Vec;
use std::time::{SystemTime};
use std::net::TcpStream;
use std::io::Write;
use std::sync::mpsc::Receiver;
use scap::capturer::{Area, Capturer, Options};
use scap::frame::Frame;
use crate::sender::ScapError::{ScapNotSupported, ScapPermissionDenied};
use crate::util::{Header, CHUNK_SIZE, Message, MessageType};

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
fn create_capturer(area: Area) -> Capturer {
    // let targets = scap::get_all_targets();
    let options = Options {
        fps: 30,
        show_cursor: true,
        show_highlight: false,
        // target: Some(targets.get(0).unwrap().clone()),
        // excluded_targets: None,
        output_type: scap::frame::FrameType::BGRAFrame,
        output_resolution: scap::capturer::Resolution::_720p,
        crop_area: Some(area),
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
    scap_init().unwrap();
    let mut capturer: Capturer = create_capturer(area);

    #[cfg(not(target_os = "windows"))]
    capturer.start_capture();

    // streaming
    'streaming: loop {
        //manage messages from gui
        if let Ok(msg) = msg_r.try_recv() {
            match msg.message_type {
                MessageType::Stop => {
                    println!("received stop request from gui");
                    break 'streaming;
                }
                MessageType::Area => {
                    area = msg.area;

                    #[cfg(not(target_os = "windows"))]
                    capturer.stop_capture();

                    capturer = create_capturer(area);

                    #[cfg(not(target_os = "windows"))]
                    capturer.start_capture();
                }
                _ => {}
            }
        }

        frame_number = frame_number + 1;

        #[cfg(target_os = "windows")]
        capturer.start_capture();

        let next_frame = capturer.get_next_frame().unwrap();

        #[cfg(target_os = "windows")]
        capturer.stop_capture();

        let size = capturer.get_output_frame_size();
        let width = size[0];
        let height = size[1];
        let mut data;

        match next_frame {
            Frame::BGRA(f) => {
                data = from_bgra_to_rgba(f.data);
            }
            Frame::BGRx(f) => {
                data = from_bgra_to_rgba(f.data);
            }
            _ => {
                println!("Sender side: the captured frame format is not supported.");
                break 'streaming;
            }
        }

        // Send header
        let header = Header::new(frame_number, data.len() as u32, width, height);
        let encoded_header: Vec<u8> = bincode::serialize(&header).unwrap();

        if let Err(e) = stream.write(&encoded_header) {
            println!("Connection closed: {}", e);
            break 'streaming;
        }
        println!("Header sent {} {}", header.frame_number, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());

        // Send frame
        let frame_pad = CHUNK_SIZE - (data.len() as u32 % CHUNK_SIZE);
        if frame_pad < CHUNK_SIZE {
            for _ in 0..frame_pad {
                data.push(0);
            }
        }
        if let Err(e) = stream.write_all(&data) {
            println!("Server closed: {}", e);
            break 'streaming;
        }
        println!("Frame sent {} {}", header.frame_number, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());
    }

    #[cfg(not(target_os = "windows"))]
    capturer.stop_capture();

    println!("Sender terminated");
}