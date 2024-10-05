use std::time::Duration;
use scrap::{Capturer, Display};

pub const FPS_SLEEP: Duration = Duration::from_millis(1000 / 9);
#[derive(Debug, Default)]
pub struct Frame {
    pub w: u32,
    pub h: u32,
    pub data: Vec<u8>,
}
#[derive(Debug, Default, Clone)]
pub struct Area {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
impl Frame {
    pub fn new(w: u32, h: u32, data: Vec<u8>) -> Self {
        Self { w, h, data }
    }
}
impl Area {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
}
pub fn create() -> Capturer {
    let display = Display::primary().expect("Couldn't find primary display.");
    Capturer::new(display).expect("Couldn't begin capture.")
}
pub fn capture(cpt: &mut Capturer) -> Vec<u8> {
    loop {
        match cpt.frame() {
            Ok(buffer) => {
                return buffer.to_vec();
            }
            Err(error) => {
                if error.kind() != std::io::ErrorKind::WouldBlock {
                    println!("Error: {}", error);
                }
                std::thread::sleep(FPS_SLEEP);
                continue;
            }
        };
    }
}
pub fn u8x4_crop(frame: Frame, area: &Area) -> Frame {
    if frame.data.len() == (area.width * area.height) as usize * 4 {
        return frame;
    }
    let mut rgba = image::RgbaImage::from_raw(frame.w, frame.h, frame.data).unwrap();
    let sub_rgba = image::imageops::crop(&mut rgba, area.x, area.y, area.width, area.height);
    Frame::new(area.width, area.height, sub_rgba.to_image().to_vec())
}
pub fn from_bgra_to_rgb(frame_data: Vec<u8>) -> Vec<u8> {
    let width = frame_data.len();
    let width_without_alpha = (width / 4) * 3;

    let mut data: Vec<u8> = vec![0; width_without_alpha];

    for (src, dst) in frame_data.chunks_exact(4).zip(data.chunks_exact_mut(3)) {
        dst[0] = src[2];
        dst[1] = src[1];
        dst[2] = src[0];
    }

    return data;
}