use crate::protocol::input_message::InputMessage;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::net::TcpStream;
use std::io::Read;
use std::thread;

pub trait Listener {
    fn handle_input_messages(&self, receiver: Receiver<Box<dyn InputMessage>>);
    fn interpret_messages(&mut self, sender: Sender<Box<dyn InputMessage>>);
}

pub(crate) struct Server {
}

impl Server {
    pub(crate) fn new(stream: TcpStream) -> Self {
        let (sender, receiver) = channel::<Box<dyn InputMessage>>();
        thread::spawn(move || handle_input_messages(receiver));
        thread::spawn(move || interpret_messages(stream, sender));
        Server {}
    }
}

fn handle_input_messages(receiver: Receiver<Box<dyn InputMessage>>) {
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

fn interpret_messages(mut stream: TcpStream, sender: Sender<Box<dyn InputMessage>>) {
    loop {
        let mut buffer: [u8; 1388] = [0; 1388];
        match stream.read(&mut buffer) {
            Ok(size) => {
                if size > 0 {
                    sender.send(InputMessage::from(buffer.to_vec()));
                }
            }
            Err(_) => panic!()
        }
    }
}
