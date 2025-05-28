use gossip::start;

#[tokio::main]
async fn main() {
    let num_nodes = 3;
    let mut handles = Vec::new();

    for _ in 0..num_nodes {
        let tmp_dir = tempdir::TempDir::new("gossip-test").unwrap();
        let handle = tokio::spawn(async move {
            let state_dir = tmp_dir.path().to_path_buf();
            start(Some(state_dir)).await.unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
