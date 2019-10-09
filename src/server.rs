use crate::protocol::input_message::InputMessage;
use std::sync::mpsc::Receiver;

pub trait Listener {
    fn handle_input_messages(&self, receiver: Receiver<Box<dyn InputMessage>>);
}

pub(crate) struct Server {

}

impl Server {
    pub(crate) fn new() -> Self {
        Server { }
    }
}

impl Listener for Server {
    fn handle_input_messages(&self, receiver: Receiver<Box<InputMessage>>) {
        loop {
            match receiver.recv() {
                Ok(message) => {
                    match message.code() {
                        1 => println!("Login response!"),
                        _ => println!("Unknown message: {}", message.code())
                    }
                },
                Err(e) => println!("something wrong")
            }
        }
    }
}