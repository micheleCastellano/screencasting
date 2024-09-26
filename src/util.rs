use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Header {
    pub frame_number: usize,
    pub len: usize,
    pub frame_width: usize,
    pub frame_height: usize,
}

impl Header {
    pub fn new(frame_number: usize, len: usize, image_width: usize, image_height: usize) -> Self {
        Self { frame_number, len, frame_width: image_width, frame_height: image_height }
    }
}
#[derive(Debug)]
pub struct ChannelFrame {
    pub w: usize,
    pub h: usize,
    pub data: Vec<u8>,
}
impl ChannelFrame {
    pub fn new(w: usize, h: usize, data: Vec<u8>) -> Self {
        Self { w, h, data }
    }
}

