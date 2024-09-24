use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Header {
    pub frame_number: usize,
    pub len: usize,
    pub image_width: usize,
    pub image_height: usize,
}

impl Header {
    pub fn new(frame_number: usize, len: usize, image_width: usize, image_height: usize) -> Self {
        Self { frame_number, len, image_width, image_height }
    }
}

pub struct ChannelFRAME {
    pub w: usize,
    pub h: usize,
    pub data: Vec<u8>,
}
impl ChannelFRAME {
    pub fn new(w: usize, h: usize, data: Vec<u8>) -> Self {
        Self { w, h, data }
    }
}

