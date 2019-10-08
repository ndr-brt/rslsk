use std::net::{TcpStream};
use std::io::{Error, Write, Read};
use buf_redux::Buffer;
use std::thread;
use std::convert::TryInto;
use std::borrow::Borrow;
use std::ops::Deref;
use message::Message;
use crate::message::LoginRequest;

mod message;

pub struct Slsk {
    server: &'static str,
    port: u16,
    username: &'static str,
    password: &'static str,
}

impl Slsk {
    pub fn new(server: &'static str, port: u16, username: &'static str, password: &'static str) -> Self {
        Slsk {
            server,
            port,
            username,
            password,
        }
    }

    pub fn login(&self) -> Result<(), Error> {
        let address = format!("{}:{}", self.server, self.port);
        println!("{}", address);
        match TcpStream::connect(address) {
            Ok(mut server) => {
                let mut output_server = server.try_clone().unwrap();
                thread::spawn(move || {
                    loop {
                        let mut buffer: [u8; 1388] = [0; 1388];
                        match output_server.read(&mut buffer) {
                            Ok(size) => {
                                if size > 0 {
                                    let size = as_u32_le(&buffer[0..4].try_into().unwrap());
                                    let code = as_u32_le(&buffer[4..8].try_into().unwrap());
                                    println!("Received data: {}. Size: {}. Code: {}", size, size, code);
                                    match code {
                                        1 => {
                                            println!("Login response message")
                                        }
                                        _ => println!("Message {} not known", code)
                                    }
                                }
                            }
                            Err(_) => panic!()
                        }
                    }
                });

                let loginRequest = Message::login_request(self.username, self.password);

                match server.write(loginRequest.as_buffer().buf()) {
                    Ok(count) => println!("Writed {} bytes to server", count),
                    Err(e) => panic!(e)
                }

                loop {}

                Result::Ok(())

            },
            Err(error) => {
                println!("{}", error.to_string());
                Result::Err(error)
            }
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

    #[test]
    fn none() {

    }
}
