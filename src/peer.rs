use std::sync::Arc;

use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::events::SearchResultItem;
use crate::message::next_packet::NextPacket;
use crate::message::peer_responses::FileSearchResponse;
use crate::message::unpack::Unpack;
use crate::server::Searches;

pub struct Peer {
    pub username: String,
    write_stream: OwnedWriteHalf
}

impl Peer {

    pub(crate) fn new(username: String, read_stream: OwnedReadHalf, write_stream: OwnedWriteHalf, searches: Searches) -> Peer {
        let peer = Peer { username, write_stream };
        tokio::spawn(Self::peer_message_receiver(read_stream, Arc::clone(&searches)));
        peer
    }

    async fn peer_message_receiver(mut read_stream: OwnedReadHalf, searches: Searches) {
        let mut packet = read_stream.next_packet().await.expect("cannot read peer packet");

        match <u32>::unpack(&mut packet) {
            9 => {
                let message = <FileSearchResponse>::unpack(&mut packet);
                println!("Received from peer: FileSearchResponse. username {}, token {}, count {}", message.username, message.token, message.results.len());
                let mut guard = searches.lock().await;
                for item in message.results {
                    match guard.get_mut(&message.token) {
                        Some(sender) => {
                            let username = message.username.clone();
                            let search_item = SearchResultItem { username, filename: item.filename };
                            sender.send(search_item).await.unwrap()
                        },
                        None => {
                            println!("no search result sender available. what do?")
                        }
                    }
                }
            }
            code => println!("Received from peer: Unknown message code: {}, length: {}", code, packet.len())
        }
    }

    pub fn transfer_request(&self) {

    }

}