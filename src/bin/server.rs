use anyhow::Result;
use tokio::{select, signal::ctrl_c};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    select! {
        r = ctrl_c() => r?,
        r = uit::server_main() => r?,
    };
    Ok(())
}
