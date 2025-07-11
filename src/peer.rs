use std::fmt::Display;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use message::peer_requests::QueueUpload;

use crate::events::SearchResultItem;
use crate::message;
use crate::message::next_packet::NextPacket;
use crate::message::pack::Pack;
use crate::message::peer_requests::{PeerRequests, UserInfoResponse};
use crate::message::peer_responses::{FileSearchResponse, UploadFailed};
use crate::message::unpack::Unpack;
use crate::server::Searches;

pub struct Peer {
    pub username: String,
    pub sender: Sender<PeerRequests>
}

impl Peer {

    pub(crate) fn new(username: String, read_stream: OwnedReadHalf, write_stream: OwnedWriteHalf, searches: Searches) -> Peer {

        let (outgoing_peer_message_bus_sender, outgoing_peer_message_bus_receiver) = mpsc::channel::<PeerRequests>(2);

        tokio::spawn(peer_message_receiver(username.clone(), read_stream, Arc::clone(&searches), outgoing_peer_message_bus_sender.clone()));
        tokio::spawn(peer_message_sender(username.clone(), outgoing_peer_message_bus_receiver, write_stream));

        Peer { username, sender: outgoing_peer_message_bus_sender }
    }

    pub async fn queue_upload(&mut self, filename: String) {
        self.sender.send(PeerRequests::QueueUpload(QueueUpload { filename })).await.unwrap();
    }

}

async fn peer_message_receiver(username: String, mut read_stream: OwnedReadHalf, searches: Searches, peer_requests: Sender<PeerRequests>) {
    let mut packet = read_stream.next_packet().await.expect("cannot read peer packet");

    match <u32>::unpack(&mut packet) {
        9 => {
            let message = <FileSearchResponse>::unpack(&mut packet);
            println!("Received from peer {}: FileSearchResponse. username {}, token {}, count {}", username, message.username, message.token, message.results.len());
            let mut guard = searches.lock().await;
            for item in message.results {
                match guard.get_mut(&message.token) {
                    Some(sender) => {
                        let username = message.username.clone();
                        let search_item = SearchResultItem { username, filename: item.filename };
                        sender.send(search_item).await.unwrap()
                    },
                    None => {
                        println!("no search result sender available. ignore message.")
                    }
                }
            }
        }
        15 => {
            println!("Received from peer {}: UserInfoRequest.", username);
            let request = PeerRequests::UserInfoResponse(UserInfoResponse {
                description: String::from("rslsk"),
                has_picture: false,
                total_upload: 1u32,
                queue_size: 1u32,
                slots_free: false
            });
            peer_requests.send(request).await.unwrap()
        }
        46 => {
            let message = <UploadFailed>::unpack(&mut packet);
            println!("Received from peer {}: {:?}", username, message)
        }

        code => println!("Received from peer {}: Unknown message code: {}, length: {}", username, code, packet.len())
    }
}

async fn peer_message_sender(username: String, mut message_receiver: mpsc::Receiver<PeerRequests>, mut write_socket: OwnedWriteHalf) {
    while let Some(message) = message_receiver.recv().await {
        let message_vec = message.pack();
        match write_socket.write(message_vec.pack().as_slice()).await {
            Ok(_count) => println!("Sent to peer {}: {:?}", username, message),
            Err(e) => std::panic::panic_any(e)
        }
    }
}