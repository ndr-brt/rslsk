use std::io::{Error, ErrorKind};

use tokio::io::Result;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::Sender;

use server::Server;

use crate::commands::Command;
use crate::commands::Command::Login;
use crate::events::{Event, SearchResultItem};

mod server;
mod message;
pub mod events;
mod commands;
pub mod command_handlers;
mod peer;

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

        let (tx, rx) = oneshot::channel::<Event>();

        let command = Login { username, password, tx };
        self.command_bus.send(command).await.unwrap();

        let response = rx.await;

        match response {
            Ok(event) => {
                match event {
                    Event::LoginSucceeded { message } => Ok(message),
                    _ => Err(Error::new(ErrorKind::Other, "event not expected"))
                }
            },
            Err(_err) => Err(Error::new(ErrorKind::Other, "cannot login"))
        }
    }

    pub async fn search(&self, query: String) -> Result<mpsc::Receiver<SearchResultItem>> {
        let (tx, rx) = oneshot::channel::<Event>();

        let command = Command::Search { query, tx };
        self.command_bus.send(command).await.unwrap();

        let response = rx.await;

        match response {
            Ok(event) => {
                match event {
                    Event::SearchResultReceived { recv } => Ok(recv),
                    _ => Err(Error::new(ErrorKind::Other, "event not expected"))
                }
            },
            Err(_err) => Err(Error::new(ErrorKind::Other, "cannot search"))
        }
    }

    pub async fn download(&self, item: SearchResultItem, destination: String) -> Result<bool> {
        let (tx, rx) = oneshot::channel::<Event>();

        let command = Command::Download { item, destination, tx };
        self.command_bus.send(command).await.unwrap();

        let response = rx.await;

        match response {
            Ok(event) => {
                match event {
                    Event::DownloadQueued { message } => Ok(true),
                    Event::DownloadFailed { message } => Ok(false),
                    _ => Err(Error::new(ErrorKind::Other, "event not expected"))
                }
            },
            Err(_err) => Err(Error::new(ErrorKind::Other, format!("cannot download: {}", _err)))
        }
    }

}

#[cfg(test)]
mod tests {

    #[test]
    fn none() {

    }
}
