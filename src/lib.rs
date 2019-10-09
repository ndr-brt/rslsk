use std::net::{TcpStream};
use std::io::{Error, Write, Read};
use std::thread;
use protocol::message::{Message};
use protocol::input_message::{InputMessage};
use std::sync::mpsc::{channel, Sender};

mod protocol;

pub struct Slsk {
    server: &'static str,
    port: u16,
    username: &'static str,
    password: &'static str,
    server_out: Sender<Box<dyn Message>>,
}

impl Slsk {
    pub fn connect(server: &'static str, port: u16, username: &'static str, password: &'static str) -> Result<Self, Error> {
        let (sender, receiver) = channel::<Box<dyn InputMessage>>();
        let address = format!("{}:{}", server, port);
        thread::spawn(move || {
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
        });

        println!("{}", address);
        match TcpStream::connect(address) {
            Ok(mut serverStream) => {
                let mut output_server = serverStream.try_clone().unwrap();
                thread::spawn(move || {
                    loop {
                        let mut buffer: [u8; 1388] = [0; 1388];
                        match output_server.read(&mut buffer) {
                            Ok(size) => {
                                if size > 0 {
                                    sender.send(InputMessage::from(buffer.to_vec()));
                                }
                            }
                            Err(_) => panic!()
                        }
                    }
                });



                let (server_out, server_out_listener) = channel::<Box<dyn Message>>();
                thread::spawn(move || {
                    loop {
                        match server_out_listener.recv() {
                            Ok(message) => {
                                match serverStream.write(message.as_buffer().buf()) {
                                    Ok(count) => println!("Message sent: Writed {} bytes to server", count),
                                    Err(e) => panic!(e)
                                }
                            },
                            Err(_) => println!("an error!")
                        }
                    }
                });


                Result::Ok(
                    Slsk {
                        server,
                        port,
                        username,
                        password,
                        server_out,
                    }
                )

            },
            Err(error) => {
                println!("{}", error.to_string());
                Result::Err(error)
            }
        }
    }

    pub fn login(&self) -> Result<(), Error> {
        self.server_out.send(Message::login_request(self.username, self.password));
        Result::Ok(())
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

    #[test]
    fn none() {

    }
}
