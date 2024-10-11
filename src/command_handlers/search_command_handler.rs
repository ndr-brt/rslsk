use rand::random;
use tokio::sync::mpsc;
use tokio::sync::oneshot::Sender;

use crate::events::{Event, SearchResultItem};
use crate::message::server_requests::{FileSearch, ServerRequests};
use crate::server::Searches;

pub struct SearchHandler {
    server_requests: mpsc::Sender<ServerRequests>,
    searches: Searches
}

impl SearchHandler {
    pub fn new(server_requests: mpsc::Sender<ServerRequests>, searches: Searches) -> SearchHandler {
        return SearchHandler { server_requests, searches }
    }

    pub async fn handle(&self, query: String, tx: Sender<Event>) {
        let token = random::<u32>();

        let (search_results_sender, search_results_receiver) = mpsc::channel::<SearchResultItem>(1024);
        self.searches.lock().await.insert(token, search_results_sender);

        self.server_requests.clone().send(ServerRequests::FileSearch(FileSearch { token, query })).await.unwrap();
        let event = Event::SearchResultReceived { token, recv: search_results_receiver };
        tx.send(event).unwrap();
    }
}