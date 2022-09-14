use futures::{StreamExt, SinkExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use headless_wss::{connect, tls};

/// headless wss client example
#[derive(argh::FromArgs)]
struct Args {
    /// domain
    #[argh(option, short = 'd')]
    domain: String,
    /// post
    #[argh(option, short = 'p')]
    port: u16,
}

#[tokio::main]
async fn main() {
    let Args { domain, port } = argh::from_env();

    let stream = TcpStream::connect((domain.as_str(), port)).await.unwrap();
    let connector = tls::build_connector();
    let websocket = connect(stream, Some(&connector), Some(&domain), None).await.unwrap();

    let (mut tx, mut rx) = websocket.split();
    tx.send(Message::Text("ping".to_owned())).await.unwrap();
    println!("{:?}", rx.next().await.unwrap());
}
