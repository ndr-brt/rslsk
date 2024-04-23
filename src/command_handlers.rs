use tokio::sync::{broadcast, mpsc};
use tokio::sync::oneshot::Sender;

use crate::events::Event::LoginSucceeded;
use crate::message::server_requests::{LoginRequest, ServerRequests};
use crate::message::server_responses::ServerResponses;

pub struct LoginHandler {
    server_requests: mpsc::Sender<ServerRequests>,
    server_responses: broadcast::Receiver<ServerResponses>
}

impl LoginHandler {

    pub fn new(server_requests: mpsc::Sender<ServerRequests>, server_responses: broadcast::Receiver<ServerResponses>) -> LoginHandler {
        return LoginHandler { server_requests, server_responses }
    }

    pub async fn handle(mut self, username: String, password: String, tx: Sender<crate::Event>) {
        println!("Execute commands: login");

        let login_request = ServerRequests::LoginRequest(LoginRequest { username, password });
        self.server_requests.send(login_request).await.unwrap();

        match self.server_responses.recv().await {
            Ok(response) => {
                match response {
                    ServerResponses::LoginResponse(login_response) => {
                        tx.send(LoginSucceeded { message: login_response.message }).unwrap()
                    }
                }
            },
            Err(err) => {
                println!("{}", err)
            }
        }
    }
}

