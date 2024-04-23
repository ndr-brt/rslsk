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

#[cfg(test)]
mod tests {
    use tokio::sync::{broadcast, mpsc, oneshot};

    use crate::command_handlers::LoginHandler;
    use crate::events::Event;
    use crate::message::server_requests::ServerRequests;
    use crate::message::server_responses::{LoginResponse, ServerResponses};

    #[tokio::test]
    async fn should_return_success_when_login_succeeded() {
        let (server_requests, mut server_requests_rx) = mpsc::channel(1);
        let (server_responses_tx, server_responses) = broadcast::channel(1);
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            let request = server_requests_rx.recv().await.unwrap();
            match request {
                ServerRequests::LoginRequest(request) => {
                    let response = LoginResponse {
                        success: true,
                        message: String::from("ok"),
                        ip: Some(12u32),
                        hash: None,
                        is_supporter: None
                    };

                    server_responses_tx.send(ServerResponses::LoginResponse(response))
                }
            }
        });

        LoginHandler::new(server_requests, server_responses)
            .handle(String::from("user"), String::from("pwd"), tx)
            .await;

        let result = rx.await.unwrap();

        assert_eq!(result, Event::LoginSucceeded { message: String::from("ok") })
    }

}