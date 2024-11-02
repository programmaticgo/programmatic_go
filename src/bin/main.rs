use programmatic_go::listener;
use std::{error::Error, net::SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    listener::start(addr).await?;
    Ok(())
}
