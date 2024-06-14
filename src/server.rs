use std::collections::HashMap;
use std::sync::Arc;

use rand::random;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::{broadcast, mpsc};
use tokio::sync::Mutex;

use crate::command_handlers::login_command_handler::LoginHandler;
use crate::commands::Command;
use crate::events::{Event, SearchResultItem};
use crate::message::pack::Pack;
use crate::message::peer::{FileSearchResponse, PeerInit};
use crate::message::server_requests::{FileSearch, ServerRequests};
use crate::message::server_responses::{ConnectToPeer, ExcludedSearchPhrases, LoginResponse, ParentMinSpeed, ParentSpeedRatio, PrivilegedUsers, RoomList, ServerResponses, WishlistInterval};
use crate::message::unpack::Unpack;

pub(crate) struct Server {
    pub(crate) command_sender: mpsc::Sender<Command>
}

type Searches = Arc<Mutex<HashMap<u32, mpsc::Sender<SearchResultItem>>>>;

impl Server {
    pub(crate) async fn new(socket: TcpStream) -> Self {
        let (read_socket, write_socket) = socket.into_split();

        let (command_bus_sender, command_bus_receiver) = mpsc::channel(8);
        let (outgoing_server_message_bus_sender, outgoing_server_message_bus_receiver) = mpsc::channel::<ServerRequests>(8);
        let (msg_tx, server_responses) = broadcast::channel::<ServerResponses>(8);

        let searches: Searches = Arc::new(Mutex::new(HashMap::new()));

        tokio::spawn(server_message_receiver(read_socket, msg_tx));
        tokio::spawn(server_message_sender(outgoing_server_message_bus_receiver, write_socket));
        tokio::spawn(peer_connections_listener(Arc::clone(&searches)));
        tokio::spawn(command_handler(command_bus_receiver, outgoing_server_message_bus_sender, server_responses, Arc::clone(&searches)));

        Server { command_sender: command_bus_sender }
    }

}

async fn command_handler(
    mut command_receiver: mpsc::Receiver<Command>,
    outgoing_server_message_bus_sender: mpsc::Sender<ServerRequests>,
    server_responses: broadcast::Receiver<ServerResponses>,
    searches: Searches
) {
    while let Some(command) = command_receiver.recv().await {
        match command {
            Command::Login { username, password, tx } => {
                LoginHandler::new(outgoing_server_message_bus_sender.clone(), server_responses.resubscribe())
                    .handle(username, password, tx)
                    .await;
            },
            Command::Search { query, tx} => {
                let token = random::<u32>();
                outgoing_server_message_bus_sender.clone().send(ServerRequests::FileSearch(FileSearch { token, query })).await.unwrap();

                let (search_results_sender, search_results_receiver) = mpsc::channel::<SearchResultItem>(1024);
                searches.lock().await.insert(token, search_results_sender);

                let event = Event::SearchResultReceived { recv: search_results_receiver };
                tx.send(event).unwrap();
            }
        }
    }
}

async fn server_message_receiver(mut read_socket: tokio::net::tcp::OwnedReadHalf, msg_tx: broadcast::Sender<ServerResponses>) {
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

async fn server_message_sender(mut message_receiver: mpsc::Receiver<ServerRequests>, mut write_socket: OwnedWriteHalf) {
    while let Some(message) = message_receiver.recv().await {
        let message_vec = message.pack();
        match write_socket.write(message_vec.pack().as_slice()).await {
            Ok(count) => println!("Message sent: Wrote {} bytes to server", count),
            Err(e) => std::panic::panic_any(e)
        }
    }
}

async fn peer_connections_listener(searches: Searches) {
    let listener = TcpListener::bind("0.0.0.0:2234").await.unwrap();
    println!("Listening for connections on: {}", listener.local_addr().unwrap());
    loop {
        let (listener_stream, socket_address) = listener.accept().await.unwrap();

        println!("Incoming connection from {}", socket_address);

        tokio::spawn(peer_init_message_receiver(listener_stream, Arc::clone(&searches)));
    }
}

async fn peer_init_message_receiver(mut listener_stream: TcpStream, searches: Searches) {
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

            tokio::spawn(peer_message_receiver(listener_stream, Arc::clone(&searches)));
        }
        code => println!("Received from peer: Unknown message code: {}, length: {}", code, length)
    }
}

async fn peer_message_receiver(mut listener_stream: TcpStream, searches: Searches) {
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
        9 => {
            let message = <FileSearchResponse>::unpack(&mut bytes);
            println!("Received from peer: FileSearchResponse. username {}, token {}, count {}", message.username, message.token, message.results.len());
            let mut guard = searches.lock().await;
            for item in message.results {
                let sender = guard.get_mut(&message.token).unwrap();
                let username = message.username.clone();
                let search_item = SearchResultItem { username, filename: item.filename };
                sender.send(search_item).await.unwrap()
            }
        }
        code => println!("Received from peer: Unknown message code: {}, length: {}", code, length)
    }
}
