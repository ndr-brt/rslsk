use std::collections::HashMap;
use std::sync::Mutex;

use rand::random;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};

use crate::command_handlers::login_command_handler::LoginHandler;
use crate::commands::Command;
use crate::events::{Event, SearchResultItem};
use crate::message::pack::Pack;
use crate::message::peer::PeerInit;
use crate::message::server_requests::{FileSearch, ServerRequests};
use crate::message::server_responses::{ConnectToPeer, ExcludedSearchPhrases, LoginResponse, ParentMinSpeed, ParentSpeedRatio, PrivilegedUsers, RoomList, ServerResponses, WishlistInterval};
use crate::message::unpack::Unpack;

pub(crate) struct Server {
    pub(crate) command_sender: mpsc::Sender<Command>
}

impl Server {
    pub(crate) async fn new(socket: TcpStream) -> Self {
        let (read_socket, mut write_socket) = socket.into_split();

        let (command_sender, mut command_receiver) = mpsc::channel(8);
        let (server_requests, mut message_receiver) = mpsc::channel::<ServerRequests>(8);
        let (msg_tx, server_responses) = broadcast::channel::<ServerResponses>(8);

        let mut searches: Mutex<HashMap<u32, mpsc::Sender<SearchResultItem>>> = Mutex::new(HashMap::new());


        tokio::spawn(async move {
            let listener = TcpListener::bind("0.0.0.0:2234").await.unwrap();
            println!("Listening for connections on: {}", listener.local_addr().unwrap());
            loop {
                let (mut listener_stream, socket_address) = listener.accept().await.unwrap();

                println!("Incoming connection from {}", socket_address);

                tokio::spawn(async move {
                    let mut length_buffer: [u8; 4] = [0, 0, 0, 0];
                    match listener_stream.read_exact(&mut length_buffer).await {
                        Ok(_len) => (),
                        Err(_err) => { return },
                    }

                    let length = u32::from_le_bytes(length_buffer);
                    let mut bytes: Vec<u8> = vec![0; length as usize];
                    let _ = listener_stream.read_exact(&mut bytes).await;

                    match <u8>::unpack(&mut bytes) {
                        1 => {
                            let message = <PeerInit>::unpack(&mut bytes);
                            println!("Received from peer: PeerInit: username: {}. type: {}. token: {}", message.username, message.connection_type, message.token);

                            // received peerinit, now we can receive messages from peer
                            tokio::spawn(async move {
                                // TODO: refactor this duplication!
                                let mut length_buffer: [u8; 4] = [0, 0, 0, 0];
                                match listener_stream.read_exact(&mut length_buffer).await {
                                    Ok(_len) => (),
                                    Err(_err) => { return },
                                }

                                let length = u32::from_le_bytes(length_buffer);
                                let mut bytes: Vec<u8> = vec![0; length as usize];
                                let _ = listener_stream.read_exact(&mut bytes).await;

                                match <u32>::unpack(&mut bytes) {
                                    code => println!("Received from peer: Unknown message code: {}, length: {}", code, length)
                                }
                            });
                        }
                        code => println!("Received from peer: Unknown message code: {}, length: {}", code, length)
                    }
                });

            }
        });

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
                        LoginHandler::new(server_requests.clone(), server_responses.resubscribe())
                            .handle(username, password, tx)
                            .await;
                    },
                    Command::Search { query, tx} => {
                        let token = random::<u32>();
                        server_requests.clone().send(ServerRequests::FileSearch(FileSearch { token, query })).await.unwrap();

                        let (search_results_sender, search_results_receiver) = mpsc::channel::<SearchResultItem>(1024);
                        searches.get_mut().unwrap().insert(token, search_results_sender);

                        let event = Event::SearchResultReceived { recv: search_results_receiver };
                        tx.send(event).unwrap();
                    }
                }
            }
        });

        Server { command_sender }
    }

}

async fn handle_server_input(mut read_socket: tokio::net::tcp::OwnedReadHalf, msg_tx: broadcast::Sender<ServerResponses>) {
    loop {
        let mut length_buffer: [u8; 4] = [0, 0, 0, 0];
        match read_socket.read_exact(&mut length_buffer).await {
            Ok(_len) => (),
            Err(_err) => return,
        }

        let length = u32::from_le_bytes(length_buffer);
        let mut bytes: Vec<u8> = vec![0; length as usize];
        let _ = read_socket.read_exact(&mut bytes).await;

        match <u32>::unpack(&mut bytes) {
            1 => {
                let response = <LoginResponse>::unpack(&mut bytes);
                println!("Received from server: LoginResponse. success: {}. message: {}. address: {:?}", response.success, response.message, response.ip);
                msg_tx.send(ServerResponses::LoginResponse(response)).unwrap();
            }
            18 => {
                let response = <ConnectToPeer>::unpack(&mut bytes);
                println!("Received from server: ConnectToPeer. token: {}. username: {}. type: {}", response.token, response.username, response.connection_type)
            }
            64 => {
                let response = <RoomList>::unpack(&mut bytes);
                println!("Received from server: RoomList. count: {}", response.number_of_rooms)
            }
            69 => {
                let response = <PrivilegedUsers>::unpack(&mut bytes);
                println!("Received from server: PrivilegedUsers. number {}", response.number_of_users)
            }
            83 => {
                let response = <ParentMinSpeed>::unpack(&mut bytes);
                println!("Received from server: ParentMinSpeed. speed: {}", response.speed);
            }
            84 => {
                let response = <ParentSpeedRatio>::unpack(&mut bytes);
                println!("Received from server: ParentSpeedRatio. ratio: {}", response.ratio);
            }
            104 => {
                let response = <WishlistInterval>::unpack(&mut bytes);
                println!("Received from server: WishlistInterval. ratio: {}", response.interval);
            }
            160 => {
                let response = <ExcludedSearchPhrases>::unpack(&mut bytes);
                println!("Received from server: ExcludedSearchPhrases. count {}. phrases: {:?}", response.count, response.phrases)
            }
            code => println!("Received from server: Unknown message code: {}, length: {}", code, length)
        }
    }
}
