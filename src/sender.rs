use crate::sender::ScapError::{ScapNotSupported, ScapPermissionDenied};
use crate::util::{Header, Message, MessageType, CHUNK_SIZE};
use scap::capturer::{Area, Capturer, Options};
use scap::frame::Frame;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, SystemTime};
use std::vec::Vec;

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
    //let targets = scap::get_all_targets();
    //println!("{:?}", targets);
    //let targets = scap::get_targets();
    let options = Options {
        fps: 30,
        show_cursor: true,
        show_highlight: false,
        //target: Some(targets.get(0).unwrap().clone()),
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

fn from_bgrx_to_rgba(mut frame: Vec<u8>) -> Vec<u8> {
    assert!(
        frame.len() % 4 == 0,
        "Frame length must be a multiple of 4!"
    );

    for chunk in frame.chunks_exact_mut(4) {
        let blue = chunk[0];
        let green = chunk[1];
        let red = chunk[2];
        chunk.copy_from_slice(&[red, green, blue, 255]);
    }

    frame
}

pub fn start(ip_addr: String, mut area: Area, msg_r: Receiver<Message>) {
    //initialization
    let mut stream;
    match TcpStream::connect(format!("{}:8080", ip_addr)) {
        Ok(s) => {
            stream = s;
        }
        Err(e) => {
            println!("Impossible connecting to {ip_addr}: {e}");
            return;
        }
    }
    println!("Connection successed");
    let mut frame_number = 0;
    scap_init().unwrap();
    let mut capturer: Capturer = create_capturer(area);

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
                    capturer = create_capturer(area);
                }
                _ => {}
            }
        }

        frame_number = frame_number + 1;

        capturer.start_capture();
        //let next_frame = capturer.get_next_frame().unwrap();
        let next_frame = capturer.get_next_frame().unwrap();
        /*
                let next_frame = match capturer.get_next_frame() {
                    Ok(next_frame) => next_frame,
                    Err(e) => {
                        thread::sleep(Duration::from_millis(1000));
                        println!("{e}");
                        continue 'streaming;
                    }
                };
        */
        //capturer.stop_capture();

        /*
                match next_frame {
                    Frame::YUVFrame(_) => println!("yuv"),
                    Frame::RGB(_) => println!("rgb"),
                    Frame::RGBx(_) => println!("rgbx"),
                    Frame::BGRA(_) => println!("bgra"),
                    Frame::XBGR(_) => println!("xbgr"),
                    Frame::BGRx(_) => println!("bgrx"),
                    Frame::BGR0(_) => println!("bgr0"),
                }
        */

        match next_frame {
            Frame::BGRA(mut frame) => {
                // Send header
                let header = Header::new(
                    frame_number,
                    frame.data.len() as u32,
                    frame.width as u32,
                    frame.height as u32,
                );
                let encoded_header: Vec<u8> = bincode::serialize(&header).unwrap();

                if let Err(e) = stream.write(&encoded_header) {
                    println!("Connection closed: {}", e);
                    break 'streaming;
                }
                println!(
                    "Header sent {} {}",
                    header.frame_number,
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );

                // Send frame
                frame.data = from_bgra_to_rgba(frame.data);
                let frame_pad = CHUNK_SIZE - (frame.data.len() as u32 % CHUNK_SIZE);
                if frame_pad < CHUNK_SIZE {
                    for _ in 0..frame_pad {
                        frame.data.push(0);
                    }
                }
                if let Err(e) = stream.write_all(&frame.data) {
                    println!("Server closed: {}", e);
                    break 'streaming;
                }
                println!(
                    "Frame sent {} {}",
                    header.frame_number,
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
            }
            Frame::BGRx(mut frame) => {
                // Send header
                let header = Header::new(
                    frame_number,
                    frame.data.len() as u32,
                    frame.width as u32,
                    frame.height as u32,
                );
                let encoded_header: Vec<u8> = bincode::serialize(&header).unwrap();

                if let Err(e) = stream.write(&encoded_header) {
                    println!("Connection closed: {}", e);
                    break 'streaming;
                }
                println!(
                    "Header sent {} {}",
                    header.frame_number,
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );

                // Send frame
                frame.data = from_bgrx_to_rgba(frame.data);
                //frame.data = from_bgra_to_rgba(frame.data);
                let frame_pad = CHUNK_SIZE - (frame.data.len() as u32 % CHUNK_SIZE);
                if frame_pad < CHUNK_SIZE {
                    for _ in 0..frame_pad {
                        frame.data.push(0);
                    }
                }
                if let Err(e) = stream.write_all(&frame.data) {
                    println!("Server closed: {}", e);
                    break 'streaming;
                }
                println!(
                    "Frame sent {} {}",
                    header.frame_number,
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
            }
            _ => {
                println!("Sender side: the captured frame format is not supported.");
                break 'streaming;
            }
        }

        /*
                if let Frame::BGRA(mut frame) = next_frame {
                    // Send header
                    let header = Header::new(
                        frame_number,
                        frame.data.len() as u32,
                        frame.width as u32,
                        frame.height as u32,
                    );
                    let encoded_header: Vec<u8> = bincode::serialize(&header).unwrap();

                    if let Err(e) = stream.write(&encoded_header) {
                        println!("Connection closed: {}", e);
                        break 'streaming;
                    }
                    println!(
                        "Header sent {} {}",
                        header.frame_number,
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis()
                    );

                    // Send frame
                    frame.data = from_bgra_to_rgba(frame.data);
                    let frame_pad = CHUNK_SIZE - (frame.data.len() as u32 % CHUNK_SIZE);
                    if frame_pad < CHUNK_SIZE {
                        for _ in 0..frame_pad {
                            frame.data.push(0);
                        }
                    }
                    if let Err(e) = stream.write_all(&frame.data) {
                        println!("Server closed: {}", e);
                        break 'streaming;
                    }
                    println!(
                        "Frame sent {} {}",
                        header.frame_number,
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis()
                    );
                } else {
                    println!("Sender side: the captured frame format is not supported.");
                    break 'streaming;
                }
        */
    }

    println!("Sender terminated");
}
