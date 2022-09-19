use futures::{StreamExt, SinkExt};
use tokio::net::TcpStream;
use headless_wss::{connect, tls, tungstenite::protocol::Message};

/// headless wss client example
#[derive(argh::FromArgs)]
struct Args {
    /// domain
    #[argh(option, short = 'd')]
    domain: String,
    /// custom ip address
    #[argh(option)]
    ip: Option<String>,
    /// port
    #[argh(option, short = 'p')]
    port: u16,
    /// keep listening
    #[argh(switch)]
    keep: bool,
}

#[tokio::main]
async fn main() {
    let Args { domain, ip, port, keep } = argh::from_env();

    let stream = TcpStream::connect((ip.as_ref().map(|x| &**x).unwrap_or(domain.as_str()), port)).await.unwrap();
    let connector = tls::build_connector();
    let websocket = connect(stream, Some(&connector), Some(&domain), None).await.unwrap();

    let (mut tx, mut rx) = websocket.split();
    tx.send(Message::Text("ping".to_owned())).await.unwrap();
    println!("{:?}", rx.next().await.unwrap());

    if keep {
        while let Some(msg) = rx.next().await {
            println!("{:?}", msg)
        }
    }
}
