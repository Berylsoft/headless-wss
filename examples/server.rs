use std::{path::PathBuf, io, fs, time::Duration};
use futures::{StreamExt, SinkExt};
use tokio::{spawn, net::TcpListener, time::sleep};
use headless_wss::{accept, tls, tungstenite::protocol::Message};

/// headless wss server example
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
    /// rsa key path
    #[argh(option, short = 'k')]
    rsa_key_path: PathBuf,
    /// certs path
    #[argh(option, short = 'c')]
    certs_path: PathBuf,
    /// text file path
    #[argh(option, short = 't')]
    text_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let Args { domain, ip, port, rsa_key_path, certs_path, text_path } = argh::from_env();

    let key = tls::load_rsa_key(rsa_key_path).unwrap();
    let certs = tls::load_certs(certs_path).unwrap();

    let server = TcpListener::bind((ip.unwrap_or(domain).as_str(), port)).await.unwrap();
    let acceptor = tls::build_acceptor(key, certs).unwrap();

    let text = text_path.and_then(|path| {
        use io::BufRead;
        let file = io::BufReader::new(fs::OpenOptions::new().read(true).open(path).unwrap());
        Some(file.lines().collect::<Result<Vec<_>, _>>().unwrap())
    });

    while let Ok((stream, _)) = server.accept().await {
        let websocket = accept(stream, Some(&acceptor), None).await.unwrap();
        let text = text.clone();
        spawn(async move {
            let (mut tx, mut rx) = websocket.split();
            println!("{:?}", rx.next().await.unwrap());
            tx.send(Message::Text("pong".to_owned())).await.unwrap();

            if let Some(text) = text {
                for line in text {
                    sleep(Duration::from_secs(1)).await;
                    tx.send(Message::Text(line)).await.unwrap();
                }
            }
        });
    }
}
