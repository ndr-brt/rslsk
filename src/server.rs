use crate::protocol::input_message::InputMessage;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::net::TcpStream;
use std::thread;
use crate::protocol::message::Message;
use crate::protocol::packet::InputPackets;
use std::io::Write;
use crate::protocol::{Looper, LoginResponded};
use std::collections::HashMap;

pub trait Listener {
    fn handle_input_messages(&self, receiver: Receiver<Box<dyn InputMessage>>);
    fn interpret_messages(&mut self, sender: Sender<Box<dyn InputMessage>>);
}

pub(crate) struct Server {
    pub(crate) out: Sender<Box<dyn Message>>,
    responses: HashMap<&'static str, Sender<Box<&'static str>>>,
    messages: Receiver<Box<String>>,
}

impl Server {
    pub(crate) fn new(socket: TcpStream) -> Self {
        let output_socket = socket.try_clone().unwrap();

        let (packets_sink, packets_stream) = channel::<Box<Vec<u8>>>();

        let mut input_packets = InputPackets::new(socket, packets_sink);
        thread::spawn(move || input_packets.loop_forever());

        let (messages_sink, messages_stream) = channel::<Box<String>>();
        thread::spawn(move || Server::handle_input_messages(packets_stream, messages_sink));

        let (server_out, server_out_listener) = channel::<Box<dyn Message>>();
        thread::spawn(move || { Server::write_to_server(output_socket, server_out_listener) });

        Server {
            out: server_out,
            responses: HashMap::new(),
            messages: messages_stream,
        }
    }

    pub fn send(&self, message: Box<dyn Message>) {
        self.out.send(message);
    }

    pub fn login(&mut self, username: &'static str, password: &'static str, response: Sender<Box<&'static str>>) {
        self.out.send(Message::login_request(username, password));
        self.responses.insert("login", response);
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

    fn handle_input_messages(receiver: Receiver<Box<Vec<u8>>>, messages_sink: Sender<Box<String>>) {
        loop {
            match receiver.recv() {
                Ok(bytes) => {
                    let message = InputMessage::from(bytes.to_vec());
                    match message.code() {
                        1 => {
                            println!("Login response!");
                            let login_responded = LoginResponded { success: true, message: "TO BE FIXED!" };
                            messages_sink.send(Box::new(serde_json::to_string(&login_responded).unwrap()));
                        },
                        _ => println!("Unknown message: {}", message.code())
                    }
                },
                Err(_e) => println!("something wrong")
            }
        }
    }
}