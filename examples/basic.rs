use env_logger::Env;
use gossip::{GossipConfig, start, tailscale};

#[tokio::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let num_nodes = 1;
    let mut handles = Vec::new();

    for _ in 0..num_nodes {
        let tmp_dir = tempdir::TempDir::new("gossip-test").unwrap();

        let handle = tokio::spawn(async move {
            let gossip_config = GossipConfig::default();

            // connect to tailscale
            let ts = tailscale::Tailscale::new(
                gossip_config.clone(),
                Some(tmp_dir.path().to_path_buf()),
            )
            .unwrap();

            let seed_peers = ts.get_peers().await.unwrap();

            start(gossip_config, Box::new(ts), seed_peers)
                .await
                .unwrap();
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
