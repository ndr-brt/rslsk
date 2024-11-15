use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::Arc;

use tokio::io::Result;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, oneshot};
use tokio::sync::mpsc::{Receiver, Sender};

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

#[derive(Clone)]
pub struct Slsk {
    username: String,
    password: String,
    command_bus: Sender<Command>,
    search_results: Arc<Mutex<HashMap<u32, SearchResults>>>
}

pub struct SearchResults {
    pub items: Vec<SearchResultItem>
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
                        search_results: Arc::new(Mutex::new(HashMap::new()))
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

    pub async fn search(&self, query: String) -> Result<u32> {
        let (tx, rx) = oneshot::channel::<Event>();

        let command = Command::Search { query, tx };
        self.command_bus.send(command).await.unwrap();

        let response = rx.await;

        match response {
            Ok(event) => {
                match event {
                    Event::SearchResultReceived { token, mut recv } => {
                        self.search_results.lock().await.insert(token, SearchResults { items: vec![] });
                        let results = Arc::clone(&self.search_results);
                        let other_token = token.clone();
                        tokio::spawn(save_search_result(other_token, results, recv));
                        Ok(token)
                    },
                    _ => Err(Error::new(ErrorKind::Other, "event not expected"))
                }
            },
            Err(_err) => Err(Error::new(ErrorKind::Other, "cannot search"))
        }
    }

    pub async fn get_search_results(self, token: u32) -> Vec<SearchResultItem> {
        match &self.search_results.lock().await.get(&token) {
            None => vec![],
            Some(results) => results.items.clone()
        }
    }

    pub async fn download(&self, item: SearchResultItem, destination: String) -> Result<String> {
        let (tx, rx) = oneshot::channel::<Event>();

        let command = Command::Download { item, destination, tx };
        self.command_bus.send(command).await.unwrap();

        let response = rx.await;

        match response {
            Ok(event) => {
                match event {
                    Event::DownloadQueued { message } => Ok(message),
                    Event::DownloadFailed { message } => Err(Error::new(ErrorKind::Other, message)),
                    _ => Err(Error::new(ErrorKind::Other, "event not expected"))
                }
            },
            Err(_err) => Err(Error::new(ErrorKind::Other, format!("cannot download: {}", _err)))
        }
    }


}


async fn save_search_result(token: u32, search_results: Arc<Mutex<HashMap<u32, SearchResults>>>, mut recv: Receiver<SearchResultItem>) {
    // TODO: drop the thread after some time!
    while let Some(item) = recv.recv().await {
        search_results.lock().await.get_mut(&token).expect("cannot find results")
            .items.push(item);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn none() {

    }
}
