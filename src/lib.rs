use std::net::{TcpStream};
use std::io::{Error, Write, Read};
use buf_redux::Buffer;
use std::thread;
use std::convert::TryInto;

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

                let message_type: u32 = 1;
                let username = self.username.as_bytes();
                let password = self.password.as_bytes();
                let unknown_field: u32 = 160;
                let another_unknown_field: u32 = 0x11;
                let cred = format!("{}{}", self.username, self.password);
                println!("credentials: {}", cred);
                let computed = md5::compute(format!("{}{}", self.username, self.password));
                let computed_string = format!("{:x}", computed);
                println!("hex credentials: {}", computed_string.as_str());

                let mut message = Buffer::new();
                message.push_bytes(&message_type.to_le_bytes());
                let username_len: u32 = username.len() as u32;
                message.push_bytes(&username_len.to_le_bytes());
                message.push_bytes(username);
                let password_len: u32 = password.len() as u32;
                message.push_bytes(&password_len.to_le_bytes());
                message.push_bytes(password);
                message.push_bytes(&unknown_field.to_le_bytes());
                let hex_credentials_len: u32 = computed_string.as_str().as_bytes().len() as u32;
                message.push_bytes(&hex_credentials_len.to_le_bytes());
                message.push_bytes(&computed_string.as_str().as_bytes());
                message.push_bytes(&another_unknown_field.to_le_bytes());

                let mut complete = Buffer::new();
                let message_len: u32 = message.len() as u32;
                complete.push_bytes(&message_len.to_le_bytes());
                complete.push_bytes(message.buf());

                println!("{:?}", complete);
                println!("{:?}", complete.buf());

                match server.write(complete.buf()) {
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
