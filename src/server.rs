use std::collections::HashMap;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{broadcast, mpsc};
use tokio::sync::Mutex;

use crate::command_handlers::login_command_handler::LoginHandler;
use crate::command_handlers::search_command_handler::SearchHandler;
use crate::commands::Command;
use crate::events::Event::DownloadFailed;
use crate::events::SearchResultItem;
use crate::message::next_packet::NextPacket;
use crate::message::pack::Pack;
use crate::message::peer::{FileSearchResponse, PeerInit};
use crate::message::server_requests::ServerRequests;
use crate::message::server_responses::{ConnectToPeer, ExcludedSearchPhrases, LoginResponse, ParentMinSpeed, ParentSpeedRatio, PrivilegedUsers, RoomList, ServerResponses, WishlistInterval};
use crate::message::unpack::Unpack;
use crate::peer::Peer;

pub(crate) struct Server {
    pub(crate) command_sender: mpsc::Sender<Command>
}

pub type Searches = Arc<Mutex<HashMap<u32, mpsc::Sender<SearchResultItem>>>>;
pub type Peers = Arc<Mutex<HashMap<String, Peer>>>;

impl Server {
    pub(crate) async fn new(socket: TcpStream) -> Self {
        let (read_socket, write_socket) = socket.into_split();

        let (command_bus_sender, command_bus_receiver) = mpsc::channel(8);
        let (outgoing_server_message_bus_sender, outgoing_server_message_bus_receiver) = mpsc::channel::<ServerRequests>(8);
        let (msg_tx, server_responses) = broadcast::channel::<ServerResponses>(8);

        let searches: Searches = Arc::new(Mutex::new(HashMap::new()));
        let peers: Peers = Arc::new(Mutex::new(HashMap::new()));

        tokio::spawn(server_message_receiver(read_socket, msg_tx));
        tokio::spawn(server_message_sender(outgoing_server_message_bus_receiver, write_socket));
        tokio::spawn(peer_connections_listener(Arc::clone(&searches), Arc::clone(&peers)));
        tokio::spawn(command_handler(command_bus_receiver, outgoing_server_message_bus_sender, server_responses, Arc::clone(&searches), Arc::clone(&peers)));

        Server { command_sender: command_bus_sender }
    }

}

async fn command_handler(
    mut command_receiver: mpsc::Receiver<Command>,
    outgoing_server_message_bus_sender: mpsc::Sender<ServerRequests>,
    server_responses: broadcast::Receiver<ServerResponses>,
    searches: Searches,
    peers: Peers
) {
    while let Some(command) = command_receiver.recv().await {
        match command {
            Command::Login { username, password, tx } => {
                LoginHandler::new(outgoing_server_message_bus_sender.clone(), server_responses.resubscribe())
                    .handle(username, password, tx)
                    .await;
            },
            Command::Search { query, tx} => {
                SearchHandler::new(outgoing_server_message_bus_sender.clone(), Arc::clone(&searches))
                    .handle(query, tx)
                    .await;
            },
            Command::Download { item, destination, tx } => {
                // start download, save to file, return good!
                match peers.lock().await.get(&item.username) {
                    Some(peer) => {

                    }
                    None => {
                        tx.send(DownloadFailed { message: format!("Cannot find peer {}", item.username) }).unwrap()
                    }
                }
            }
        }
    }
}

async fn server_message_receiver(mut read_socket: OwnedReadHalf, msg_tx: broadcast::Sender<ServerResponses>) {
    loop {
        let mut packet = read_socket.next_packet().await.expect("cannot read server packet");

        match <u32>::unpack(&mut packet) {
            1 => {
                let response = <LoginResponse>::unpack(&mut packet);
                println!("Received from server: LoginResponse. success: {}. message: {}. address: {:?}", response.success, response.message, response.ip);
                msg_tx.send(ServerResponses::LoginResponse(response)).unwrap();
            }
            18 => {
                let response = <ConnectToPeer>::unpack(&mut packet);
                println!("Received from server: ConnectToPeer. token: {}. username: {}. type: {}", response.token, response.username, response.connection_type)
            }
            64 => {
                let response = <RoomList>::unpack(&mut packet);
                println!("Received from server: RoomList. count: {}", response.number_of_rooms)
            }
            69 => {
                let response = <PrivilegedUsers>::unpack(&mut packet);
                println!("Received from server: PrivilegedUsers. number {}", response.number_of_users)
            }
            83 => {
                let response = <ParentMinSpeed>::unpack(&mut packet);
                println!("Received from server: ParentMinSpeed. speed: {}", response.speed);
            }
            84 => {
                let response = <ParentSpeedRatio>::unpack(&mut packet);
                println!("Received from server: ParentSpeedRatio. ratio: {}", response.ratio);
            }
            104 => {
                let response = <WishlistInterval>::unpack(&mut packet);
                println!("Received from server: WishlistInterval. ratio: {}", response.interval);
            }
            160 => {
                let response = <ExcludedSearchPhrases>::unpack(&mut packet);
                println!("Received from server: ExcludedSearchPhrases. count {}. phrases: {:?}", response.count, response.phrases)
            }
            code => println!("Received from server: Unknown message code: {}, length: {}", code, packet.len())
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

async fn peer_connections_listener(searches: Searches, peers: Peers) {
    let listener = TcpListener::bind("0.0.0.0:2234").await.unwrap();
    println!("Listening for connections on: {}", listener.local_addr().unwrap());
    loop {
        let (listener_stream, socket_address) = listener.accept().await.unwrap();
        let (read_stream, _write_stream) = listener_stream.into_split();

        println!("Incoming connection from {}", socket_address);

        tokio::spawn(peer_init_message_receiver(read_stream, Arc::clone(&searches), Arc::clone(&peers)));
    }
}

async fn peer_init_message_receiver(mut read_stream: OwnedReadHalf, searches: Searches, peers: Peers) {
    let mut packet = read_stream.next_packet().await.expect("cannot read peer packet");

    match <u8>::unpack(&mut packet) {
        1 => {
            let message = <PeerInit>::unpack(&mut packet);
            println!("Received from peer: PeerInit: username: {}. type: {}. token: {}", message.username, message.connection_type, message.token);
            let username = message.username.clone();
            peers.lock().await.insert(message.username, Peer::new(username, read_stream, Arc::clone(&searches)));
        }
        code => println!("Received from peer: Unknown message code: {}, length: {}", code, packet.len())
    }
}