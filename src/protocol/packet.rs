use std::convert::TryInto;
use std::io::Read;
use std::net::TcpStream;
use std::sync::mpsc::Sender;

use buffer_redux::Buffer;

use crate::protocol::Looper;
use crate::utils::as_u32_le;

pub(crate) struct InputPackets {
    stream: TcpStream,
    sender: Sender<Box<Vec<u8>>>,
}

impl InputPackets {
    pub(crate) fn new(stream: TcpStream, sender: Sender<Box<Vec<u8>>>) -> Self {
        InputPackets { stream, sender }
    }
}

impl Looper for InputPackets {
    fn loop_forever(&mut self) {
        let mut memory = Buffer::new();
        loop {
            let mut buffer: [u8; 2048] = [0; 2048];
            match self.stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    memory.push_bytes(&buffer[0..bytes_read]);

                    let message_size = as_u32_le(memory.buf()[0..4].try_into().unwrap());
                    if message_size + 4 <= memory.len() as u32 {
                        let length = message_size + 4;
                        let message = &memory.buf()[0..length as usize];
                        self.sender.send(Box::new(Vec::from(message))).unwrap();
                        memory.consume(length as usize);
                    }
                }
                Err(_) => panic!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::net::{TcpListener, TcpStream};
    use std::sync::mpsc::channel;
    use std::thread;

    use crate::message::pack::Pack;
    use crate::protocol::Looper;
    use crate::protocol::packet::InputPackets;

    #[test]
    fn handle_small_message() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();

        let (sender, receiver) = channel::<Box<Vec<u8>>>();
        let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let mut input_message_handler = InputPackets::new(stream, sender);
        thread::spawn(move || input_message_handler.loop_forever());

        let mut input = vec![];
        input.extend(16u32.pack());
        input.extend(1u32.pack());
        input.extend(String::from("12345678").pack());
        let packed = input.pack();

        let mut writer = listener.accept().unwrap().0;
        writer.write(packed.as_slice()).unwrap();

        let actual = receiver.recv().unwrap();
        assert_eq!(&actual[0..packed.len()], packed);
    }

    #[test]
    fn handle_message_split_in_two_parts() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();

        let (sender, receiver) = channel::<Box<Vec<u8>>>();
        let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let mut input_message_handler = InputPackets::new(stream, sender);
        thread::spawn(move || input_message_handler.loop_forever());

        let mut first_part = vec![];
        first_part.extend(16u32.pack());
        first_part.extend(1u32.pack());

        let mut second_part = vec![];
        second_part.extend(String::from("12345678").pack());

        let mut writer = listener.accept().unwrap().0;
        writer.write(first_part.as_slice()).unwrap();
        writer.write(second_part.as_slice()).unwrap();

        let actual = receiver.recv().unwrap();
        let expected = [first_part.as_slice(), second_part.as_slice()].concat();
        assert_eq!(&actual[0..first_part.len() + second_part.len()], &expected[0..expected.len()]);
    }

    #[test]
    fn handle_two_messages_in_one_packet() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();

        let (sender, receiver) = channel::<Box<Vec<u8>>>();
        let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let mut input_message_handler = InputPackets::new(stream, sender);
        thread::spawn(move || input_message_handler.loop_forever());

        let mut input = vec![];
        input.extend(16u32.pack());
        input.extend(1u32.pack());
        input.extend(String::from("12345678").pack());

        let mut writer = listener.accept().unwrap().0;
        let message = [input.as_slice(), input.as_slice()].concat();
        writer.write(&message).unwrap();

        let first = receiver.recv().unwrap();
        assert_eq!(&first[0..input.len()], input.as_slice());
    }
}