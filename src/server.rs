use crate::protocol::input_message::InputMessage;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::net::TcpStream;
use std::io::Read;
use std::thread;
use crate::protocol::message::Message;
use crate::protocol::packet::InputPackets;
use std::io::Write;
use crate::protocol::packet::Looper;

pub trait Listener {
    fn handle_input_messages(&self, receiver: Receiver<Box<dyn InputMessage>>);
    fn interpret_messages(&mut self, sender: Sender<Box<dyn InputMessage>>);
}

pub(crate) struct Server {
    pub(crate) out: Sender<Box<dyn Message>>,
}

impl Server {
    pub(crate) fn new(socket: TcpStream) -> Self {
        let output_socket = socket.try_clone().unwrap();

        let (packets_in, packets_out) = channel::<Box<Vec<u8>>>();

        let mut input_packets = InputPackets::new(socket, packets_in);
        thread::spawn(move || input_packets.loop_forever());

        let (sender, receiver) = channel::<Box<dyn InputMessage>>();
        thread::spawn(move || interpret_messages(packets_out, sender));

        thread::spawn(move || handle_input_messages(receiver));

        let (server_out, server_out_listener) = channel::<Box<dyn Message>>();
        thread::spawn(move || { Server::write_to_server(output_socket, server_out_listener) });

        Server {
            out: server_out,
        }
    }

    fn write_to_server(mut output_stream: TcpStream, server_out_listener: Receiver<Box<dyn Message>>) {
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
            Err(_e) => println!("something wrong")
        }
    }
}

fn interpret_messages(mut receiver: Receiver<Box<Vec<u8>>>, sender: Sender<Box<dyn InputMessage>>) {
    loop {
        match receiver.recv() {
            Ok(message) => {
                sender.send(InputMessage::from(message.to_vec()));
            }
            Err(_) => panic!()
        }
    }
}
