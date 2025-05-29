use std::sync::Arc;

use env_logger::Env;
use gossip::message::GossipMessage;
use gossip::{GossipHandler, start};
use tokio::sync::Mutex;

struct Handler;

impl GossipHandler for Handler {
    fn handle_message(
        &mut self,
        msg: GossipMessage,
        src: std::net::SocketAddr,
    ) {
        println!("Received message: {:?} from {}", msg, src);
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(
        Env::default().default_filter_or("info"),
    );

    let num_nodes = 3;
    let mut handles = Vec::new();

    let handler = Arc::new(Mutex::new(Handler));

    for _ in 0..num_nodes {
        let handler = Arc::clone(&handler);
        let tmp_dir = tempdir::TempDir::new("gossip-test").unwrap();
        let handle = tokio::spawn(async move {
            let state_dir = tmp_dir.path().to_path_buf();
            start(handler.clone(), Some(state_dir)).await.unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
