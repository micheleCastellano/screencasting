use scap::capturer::Area;
use serde::{Deserialize, Serialize};

pub const CHUNK_SIZE: u32 = 64 * 1024; //65536

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Header {
    pub frame_number: u32,
    pub len: u32,
    pub frame_width: u32,
    pub frame_height: u32,
}

impl Header {
    pub fn new(frame_number: u32, len: u32, image_width: u32, image_height: u32) -> Self {
        Self { frame_number, len, frame_width: image_width, frame_height: image_height }
    }
}
#[derive(Debug)]
pub struct ChannelFrame {
    pub w: u32,
    pub h: u32,
    pub data: Vec<u8>,
}
impl ChannelFrame {
    pub fn new(w: u32, h: u32, data: Vec<u8>) -> Self {
        Self { w, h, data }
    }
}

#[derive(Default)]
pub enum MessageType {
    #[default]
    Stop,
    Area,
    Save,
}

#[derive(Default)]
pub struct Message {
    pub message_type: MessageType,
    pub area: Area,
    pub save_option: bool,
}

impl Message {
    pub fn stop_request() -> Self {
        Self::default()
    }
    pub fn save_request(save_option: bool) -> Self {
        Self {
            message_type: MessageType::Save,
            save_option,
            ..Default::default()
        }
    }
    pub fn area_request(area: Area) -> Self {
        Self {
            message_type: MessageType::Area,
            area,
            ..Default::default()
        }
    }
}

