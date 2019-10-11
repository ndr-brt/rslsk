use crate::protocol::input_message::InputMessage;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::net::TcpStream;
use std::io::Read;
use std::thread;
use crate::protocol::message::Message;
use std::io::Write;

pub trait Listener {
    fn handle_input_messages(&self, receiver: Receiver<Box<dyn InputMessage>>);
    fn interpret_messages(&mut self, sender: Sender<Box<dyn InputMessage>>);
}

pub(crate) struct Server {
    pub(crate) out: Sender<Box<dyn Message>>,
}

impl Server {
    pub(crate) fn new(stream: TcpStream) -> Self {
        let (sender, receiver) = channel::<Box<dyn InputMessage>>();
        thread::spawn(move || handle_input_messages(receiver));
        let input_stream = stream.try_clone().unwrap();
        let mut output_stream = stream.try_clone().unwrap();
        thread::spawn(move || interpret_messages(input_stream, sender));

        let (server_out, server_out_listener) = channel::<Box<dyn Message>>();
        thread::spawn(move || {
            loop {
                match server_out_listener.recv() {
                    Ok(message) => {
                        match output_stream.write(message.as_buffer().buf()) {
                            Ok(count) => println!("Message sent: Writed {} bytes to server", count),
                            Err(e) => panic!(e)
                        }
                    },
                    Err(_) => println!("an error!")
                }
            }
        });

        Server {
            out: server_out,
        }
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
