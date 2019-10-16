use std::net::{TcpStream};
use std::io::{Error};
use std::sync::mpsc::{channel, RecvError};
use server::Server;
use crate::protocol::LoginResponded;

mod protocol;
mod server;
mod utils;

pub struct Slsk {
    username: &'static str,
    password: &'static str,
    server: Server,
}

impl Slsk {
    pub fn connect(server: &'static str, port: u16, username: &'static str, password: &'static str) -> Result<Self, Error> {
        let address = format!("{}:{}", server, port);

        println!("{}", address);
        match TcpStream::connect(address) {
            Ok(socket) => {
                let server = Server::new(socket);

                Result::Ok(
                    Slsk {
                        username,
                        password,
                        server,
                    }
                )
            }
            Err(error) => {
                println!("{}", error.to_string());
                Result::Err(error)
            }
        }
    }

    pub fn login(&mut self) -> Result<LoginResponded, RecvError> {
        let (sink, stream) = channel::<Box<&'static str>>();
        self.server.login(self.username, self.password, sink);
        match stream.recv() {
            Ok(message) => Result::Ok(serde_json::from_str(message.as_ref()).unwrap()),
            Err(e) => Result::Err(e)
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn none() {

    }
}
