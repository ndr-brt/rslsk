use serde::{Serialize, Deserialize};

pub(crate) mod message;
pub(crate) mod input_message;
pub(crate) mod slsk_buffer;
pub(crate) mod packet;

pub(crate) trait Looper {
    fn loop_forever(&mut self);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponded {
    pub success: bool,
    pub message: &'static str,
}