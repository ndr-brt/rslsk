use std::convert::TryInto;
use crate::utils::as_u32_le;

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