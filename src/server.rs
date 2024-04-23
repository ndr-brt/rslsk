use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{channel, Sender};

use crate::command_handlers::LoginHandler;
use crate::commands::Command;
use crate::message::pack::Pack;
use crate::message::server_requests::ServerRequests;
use crate::message::server_responses::{LoginResponse, RoomList, ServerResponses};
use crate::message::unpack::Unpack;

pub(crate) struct Server {
    pub(crate) command_sender: Sender<Command>
}

impl Server {
    pub(crate) async fn new(socket: TcpStream) -> Self {
        let (read_socket, mut write_socket) = socket.into_split();

        let (command_sender, mut command_receiver) = channel(8);
        let (message_sender, mut message_receiver) = channel::<ServerRequests>(8);
        let (msg_tx, msg_rx) = broadcast::channel::<ServerResponses>(8);

        tokio::spawn(handle_server_input(read_socket, msg_tx));

        tokio::spawn(async move {
            while let Some(message) = message_receiver.recv().await {
                let message_vec = message.pack();
                match write_socket.write(message_vec.pack().as_slice()).await {
                    Ok(count) => println!("Message sent: Wrote {} bytes to server", count),
                    Err(e) => std::panic::panic_any(e)
                }
            }
        });

        tokio::spawn(async move {
            while let Some(command) = command_receiver.recv().await {
                match command {
                    Command::Login { username, password, tx } => {
                        LoginHandler::new(message_sender.clone(), msg_rx.resubscribe())
                            .handle(username, password, tx)
                            .await;
                    }
                }
            }
        });

        Server { command_sender }
    }

}

async fn handle_server_input(mut read_socket: tokio::net::tcp::OwnedReadHalf, msg_tx: broadcast::Sender<ServerResponses>) {
    loop {
        let mut length: [u8; 4] = [0, 0, 0, 0];
        match read_socket.read_exact(&mut length).await {
            Ok(_len) => (),
            Err(_err) => return,
        }

        let length = u32::from_le_bytes(length);
        let mut bytes: Vec<u8> = vec![0; length as usize];
        let _ = read_socket.read_exact(&mut bytes).await;
        let code = <u32>::unpack(&mut bytes);

        println!("Received message type {}, length {}", code, length);
        match code {
            1 => {
                let response = <LoginResponse>::unpack(&mut bytes);
                println!("Login response. Success? {}. Message: {}", response.success, response.message);
                msg_tx.send(ServerResponses::LoginResponse(response)).unwrap();
            },
            64 => {
                let response = <RoomList>::unpack(&mut bytes);
                println!("RoomList count: {}.", response.number_of_rooms)
            }
            code => println!("Unknown message code: {}", code)
        }
    }
}
