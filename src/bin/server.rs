use anyhow::Result;
use tokio::{select, signal::ctrl_c, net::TcpListener, stream::StreamExt};
use tokio_util::compat::*;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let listener = TcpListener::bind("0.0.0.0:12342").await?;
    let listener = listener.map(|s| s.map(|s| s.compat()));

    select! {
        r = ctrl_c() => r?,
        r = uit::server_main(listener) => r?,
    };
    Ok(())
}
