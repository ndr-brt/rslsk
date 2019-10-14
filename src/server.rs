use crate::protocol::input_message::InputMessage;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::net::TcpStream;
use std::io::Read;
use std::thread;
use crate::protocol::message::Message;
use std::io::Write;
use buf_redux::Buffer;
use std::convert::TryInto;

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

        let (sender, receiver) = channel::<Box<dyn InputMessage>>();
        thread::spawn(move || handle_input_messages(receiver));
        thread::spawn(move || interpret_messages(socket, sender));

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

fn handle_input_message(mut stream: TcpStream, sender: Sender<Box<Vec<u8>>>) {
    let mut memory = Buffer::new();
    loop {
        let mut buffer: [u8; 2048] = [0; 2048];
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                memory.push_bytes(&buffer[0..bytes_read]);

                let message_size = as_u32_le(&memory.buf()[0..4].try_into().unwrap());
                println!("MEMORY_SIZE {}", memory.len());
                println!("MESSAGE_SIZE {}", message_size);
                println!("MEMORY {:?}", memory.buf());
                if message_size + 4 <= memory.len() as u32 {
                    println!("È QUI!");
                    sender.send(Box::new(Vec::from(memory.buf())));
                }

            }
            Err(_) => panic!()
        }
    }
}

fn as_u32_le(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) <<  0) |
        ((array[1] as u32) <<  8) |
        ((array[2] as u32) << 16) |
        ((array[3] as u32) << 24)
}

#[cfg(test)]
mod tests {
    use crate::server::{handle_input_message};
    use std::net::{TcpStream, TcpListener};
    use std::sync::mpsc::channel;
    use crate::protocol::slsk_buffer::SlskBuffer;
    use std::io::Write;
    use std::thread;

    macro_rules! t {
        ($e:expr) => {
            match $e {
                Ok(t) => t,
                Err(e) => panic!("received error for `{}`: {}", stringify!($e), e),
            }
        }
    }

    #[test]
    fn handle_small_message() {
        let address = "127.0.0.1:13123";
        let listener = t!(TcpListener::bind(address));

        let (sender, receiver) = channel::<Box<Vec<u8>>>();

        thread::spawn(move || handle_input_message(t!(TcpStream::connect(address)), sender));

        let input = SlskBuffer::new()
            .append_u32(16)
            .append_u32(1)
            .append_string("12345678")
            .raw_buffer();

        let mut writer = t!(listener.accept()).0;
        t!(writer.write(input.buf()));

        let actual = t!(receiver.recv());
        assert_eq!(&actual[0..input.len()], input.buf());
    }

    #[test]
    fn handle_message_split_in_two_parts() {
        let address = "127.0.0.1:13124";
        let listener = t!(TcpListener::bind(address));

        let (sender, receiver) = channel::<Box<Vec<u8>>>();

        thread::spawn(move || handle_input_message(t!(TcpStream::connect(address)), sender));

        let first_part = SlskBuffer::new()
            .append_u32(16)
            .append_u32(1)
            .raw_buffer();

        let second_part = SlskBuffer::new()
            .append_string("12345678")
            .raw_buffer();

        let mut writer = t!(listener.accept()).0;
        t!(writer.write(first_part.buf()));
        t!(writer.write(second_part.buf()));

        let actual = t!(receiver.recv());
        let expected = [first_part.buf(), second_part.buf()].concat();
        assert_eq!(&actual[0..first_part.len() + second_part.len()], &expected[0..expected.len()]);
    }
}