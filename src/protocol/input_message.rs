use std::convert::TryInto;

pub trait InputMessage: Send {
    fn code(&self) -> u32;
}

impl dyn InputMessage {
    pub fn from(buffer: Vec<u8>) -> Box<dyn InputMessage> {
        Box::new(InputMessageImpl { buffer })
    }
}

pub(crate) struct InputMessageImpl {
    buffer: Vec<u8>
}

impl InputMessage for InputMessageImpl {
    fn code(&self) -> u32 {
        as_u32_le(self.buffer[4..8].try_into().unwrap())
    }
}

fn as_u32_le(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) <<  0) |
        ((array[1] as u32) <<  8) |
        ((array[2] as u32) << 16) |
        ((array[3] as u32) << 24)
}