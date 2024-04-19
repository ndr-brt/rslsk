use std::io::Error;
use std::net::TcpStream;
use std::sync::mpsc::Sender;

use server::Server;

use crate::message::pack::Pack;
use crate::message::server_requests::LoginRequest;

mod protocol;
mod server;
mod utils;
mod message;

pub struct Slsk {
    username: String,
    password: String,
    server_out: Sender<Box<Vec<u8>>>,
}

impl Slsk {
    pub fn connect(server: &'static str, port: u16, username: String, password: String) -> Result<Self, Error> {
        let address = format!("{}:{}", server, port);

        println!("{}", address);
        match TcpStream::connect(address) {
            Ok(socket) => {
                let server = Server::new(socket);

                Ok(
                    Slsk {
                        username,
                        password,
                        server_out: server.out,
                    }
                )
            }
            Err(error) => {
                println!("{}", error.to_string());
                Err(error)
            }
        }
    }

    pub fn login(&self) -> Result<(), Error> {
        let username = self.username.clone();
        let password = self.password.clone();
        let request = LoginRequest { username, password };
        self.server_out.send(Box::new(request.pack())).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn none() {

    }
}
