use std::net::{TcpStream};
use std::io::{Error};
use protocol::message::{Message};
use std::sync::mpsc::{Sender};
use server::Server;

mod protocol;
mod server;
mod utils;

pub struct Slsk {
    username: &'static str,
    password: &'static str,
    server_out: Sender<Box<dyn Message>>,
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
                        server_out: server.out,
                    }
                )
            }
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

#[cfg(test)]
mod tests {

    #[test]
    fn none() {

    }
}
