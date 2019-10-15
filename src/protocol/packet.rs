use std::net::TcpStream;
use std::sync::mpsc::{Sender};
use buf_redux::Buffer;
use std::io::Read;
use std::convert::TryInto;
use crate::utils::as_u32_le;
use crate::protocol::Looper;

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

                    let message_size = as_u32_le(&memory.buf()[0..4].try_into().unwrap());
                    if message_size + 4 <= memory.len() as u32 {
                        let length = message_size + 4;
                        let message = &memory.buf()[0..length as usize];
                        self.sender.send(Box::new(Vec::from(message)));
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
    use std::net::{TcpStream, TcpListener};
    use std::sync::mpsc::channel;
    use crate::protocol::slsk_buffer::SlskBuffer;
    use std::io::Write;
    use std::thread;
    use crate::protocol::Looper;
    use crate::protocol::packet::InputPackets;

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
        let mut input_message_handler = InputPackets::new(t!(TcpStream::connect(address)), sender);
        thread::spawn(move || input_message_handler.loop_forever());

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
        let mut input_message_handler = InputPackets::new(t!(TcpStream::connect(address)), sender);
        thread::spawn(move || input_message_handler.loop_forever());

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

    #[test]
    fn handle_two_messages_in_one_packet() {
        let address = "127.0.0.1:13125";
        let listener = t!(TcpListener::bind(address));

        let (sender, receiver) = channel::<Box<Vec<u8>>>();
        let mut input_message_handler = InputPackets::new(t!(TcpStream::connect(address)), sender);
        thread::spawn(move || input_message_handler.loop_forever());

        let input = SlskBuffer::new()
            .append_u32(16)
            .append_u32(1)
            .append_string("12345678")
            .raw_buffer();

        let mut writer = t!(listener.accept()).0;
        let message = [input.buf(), input.buf()].concat();
        t!(writer.write(&message));

        let first = t!(receiver.recv());
        assert_eq!(&first[0..input.len()], input.buf());
    }
}