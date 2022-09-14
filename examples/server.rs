use std::path::PathBuf;
use futures::{StreamExt, SinkExt};
use tokio::{spawn, net::TcpListener};
use tokio_tungstenite::tungstenite::protocol::Message;
use headless_wss::{accept, tls};

/// headless wss client example
#[derive(argh::FromArgs)]
struct Args {
    /// domain
    #[argh(option, short = 'd')]
    domain: String,
    /// post
    #[argh(option, short = 'p')]
    port: u16,
    /// rsa key path
    #[argh(option, short = 'k')]
    rsa_key_path: PathBuf,
    /// certs path
    #[argh(option, short = 'c')]
    certs_path: PathBuf,
}

#[tokio::main]
async fn main() {
    let Args { domain, port, rsa_key_path, certs_path } = argh::from_env();

    let key = tls::load_rsa_key(rsa_key_path).unwrap();
    let certs = tls::load_certs(certs_path).unwrap();

    let server = TcpListener::bind((domain.as_str(), port)).await.unwrap();
    let acceptor = tls::build_acceptor(key, certs).unwrap();

    while let Ok((stream, _)) = server.accept().await {
        let websocket = accept(stream, Some(&acceptor), None).await.unwrap();
        spawn(async move {
            let (mut tx, mut rx) = websocket.split();
            println!("{:?}", rx.next().await.unwrap());
            tx.send(Message::Text("pong".to_owned())).await.unwrap();
        });
    }
}
