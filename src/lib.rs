use std::path::PathBuf;

mod discovery;
mod util;

pub async fn start(state_dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let discovery = discovery::Network::new(state_dir)?;
    discovery.join().await?;

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         // let result = add(2, 2);
//         // assert_eq!(result, 4);
//     }
// }
