use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::message::pack::Pack;
use crate::message::server_responses::LoginResponse;
use crate::message::unpack::Unpack;
use crate::protocol::Looper;
use crate::protocol::packet::InputPackets;

pub(crate) struct Server {
    pub(crate) out: Sender<Box<Vec<u8>>>,
}

impl Server {
    pub(crate) fn new(socket: TcpStream) -> Self {
        let output_socket = socket.try_clone().unwrap();

        let (packets_sink, packets_stream) = channel::<Box<Vec<u8>>>();

        let mut input_packets = InputPackets::new(socket, packets_sink);
        thread::spawn(move || input_packets.loop_forever());
        thread::spawn(move || handle_input_messages(packets_stream));

        let (server_out, server_out_listener) = channel::<Box<Vec<u8>>>();
        thread::spawn(move || { Server::write_to_server(output_socket, server_out_listener) });

        Server {
            out: server_out,
        }
    }

    fn write_to_server(mut output_stream: TcpStream, server_out_listener: Receiver<Box<Vec<u8>>>) {
        loop {
            match server_out_listener.recv() {
                Ok(message) => {
                    match output_stream.write(message.as_ref().pack().as_slice()) {
                        Ok(count) => println!("Message sent: Wrote {} bytes to server", count),
                        Err(e) => std::panic::panic_any(e)
                    }
                },
                Err(_) => println!("an error!")
            }
        }
    }
}

fn handle_input_messages(receiver: Receiver<Box<Vec<u8>>>) {
    loop {
        match receiver.recv() {
            Ok(bytes) => {
                let mut vec = bytes.to_vec();
                let message_length = <u32>::unpack(&mut vec);
                let message_type = <u32>::unpack(&mut vec);
                println!("Received message type {}, lenght {}", message_type, message_length);
                match message_type {
                    1 => {
                        let response = <LoginResponse>::unpack(&mut vec);
                        println!("Login response. Success? {}. Message: {}", response.success, response.message)
                    },
                    code => println!("Unknown message code: {}", code)
                }
            },
            Err(_e) => println!("something wrong")
        }
    }
}