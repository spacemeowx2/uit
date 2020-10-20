use anyhow::Result;
use tokio::net::{TcpStream, TcpListener};

async fn process(s: TcpStream) {

}

#[tokio::main]
async fn main() -> Result<()> {
    let t = TcpListener::bind("0.0.0.0:12342").await?;
    loop {
        let (s, _) = t.accept().await?;
        tokio::spawn(async { process(s).await });
    }
    // Ok(())
}
