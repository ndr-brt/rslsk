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
    pub(crate) fn new(mut socket: TcpStream) -> Self {
        let mut output_socket = socket.try_clone().unwrap();

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
    loop {
        let mut buffer: [u8; 2048] = [0; 2048];
        match stream.read(&mut buffer) {
            Ok(size) => {
                sender.send(Box::new(buffer.to_vec()));
            }
            Err(_) => panic!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::server::{handle_input_message};
    use std::net::{TcpStream, SocketAddr, SocketAddrV4, Ipv4Addr, TcpListener};
    use std::sync::mpsc::channel;
    use crate::protocol::slsk_buffer::SlskBuffer;
    use std::io::Write;
    use std::sync::atomic::Ordering;
    use std::sync::atomic::AtomicUsize;
    use std::error::Error;
    use std::thread;

    static PORT: AtomicUsize = AtomicUsize::new(0);

    fn next_test_ip4() -> SocketAddr {
        let port = PORT.fetch_add(1, Ordering::SeqCst) as u16 + 19000;
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port))
    }

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

        let mut stream = t!(TcpStream::connect(address));
        let (sender, receiver) = channel::<Box<Vec<u8>>>();
        let output = t!(stream.try_clone());

        thread::spawn(move || handle_input_message(output, sender));

        let input = SlskBuffer::new()
            .append_u32(34)
            .append_string("12345678")
            .to_buffer();

        let mut server = t!(listener.accept()).0;
        t!(server.write(input.buf()));

        match receiver.recv() {
            Ok(message) => {
                assert_eq!(&message[0..input.len()], input.buf());
            },
            Err(e) => panic!("Error: {}", e.description())
        }
    }
}