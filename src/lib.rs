use std::io::{Error, ErrorKind};

use tokio::io::Result;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;

use server::Server;

use crate::commands::Command;
use crate::commands::Command::Login;
use crate::events::Event;

mod server;
mod message;
pub mod events;
mod commands;

pub struct Slsk {
    username: String,
    password: String,
    command_bus: Sender<Command>
}

impl Slsk {
    pub async fn connect(server: &'static str, port: u16, username: String, password: String) -> Result<Self> {
        let address = format!("{}:{}", server, port);

        println!("{}", address);
        match TcpStream::connect(address).await {
            Ok(socket) => {

                let server = Server::new(socket).await;

                Ok(
                    Slsk {
                        username,
                        password,
                        command_bus: server.command_sender.clone(),
                    }
                )
            }
            Err(error) => {
                println!("{}", error.to_string());
                Err(error)
            }
        }
    }

    pub async fn login(&self) -> Result<String> {
        let username = self.username.clone();
        let password = self.password.clone();

        let (tx, rx) = tokio::sync::oneshot::channel::<Event>();

        let command = Login { username, password, tx };
        self.command_bus.send(command).await.unwrap();

        let response = rx.await;

        match response {
            Ok(event) => {
                match event {
                    Event::LoginSucceeded { message } => Ok(message)
                }
            },
            Err(_err) => Err(Error::new(ErrorKind::Other, "cannot login"))
        }
    }

}

#[cfg(test)]
mod tests {

    #[test]
    fn none() {

    }
}
