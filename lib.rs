pub mod tls;
pub mod stream;

use std::path::PathBuf;
use futures::{StreamExt, SinkExt};
use tokio::{spawn, net::{TcpStream, TcpListener}};
use tokio_tungstenite::{WebSocketStream, tungstenite::protocol::{Role, Message}};
use crate::stream::MaybeTlsStream;

pub struct Connection {
    pub role: Role,
    pub tls: bool,
    pub domain: String,
    pub port: u16,
    pub tls_server_config: Option<TlsServerConfig>,
}

pub struct TlsServerConfig {
    pub key_path: PathBuf,
    pub certs_path: PathBuf,
}

pub async fn ws_or_wss_client_example(Connection { role, tls, domain, port, .. }: Connection) {
    assert_eq!(role, Role::Client);

    let connector = tls.then(|| tls::build_connector());
    let stream = TcpStream::connect((domain.as_str(), port)).await.unwrap();
    let stream = if tls {
        MaybeTlsStream::ClientTls(tls::connect(connector.as_ref().unwrap(), &domain, stream).await.unwrap())
    } else {
        MaybeTlsStream::Plain(stream)
    };
    let websocket = WebSocketStream::from_raw_socket(stream, role, None).await;

    // start example
    let (mut tx, mut rx) = websocket.split();
    tx.send(Message::Text("hi".to_owned())).await.unwrap();
    println!("{:?}", rx.next().await.unwrap());
}

pub async fn ws_or_wss_server_example(Connection { role, tls, domain, port, tls_server_config }: Connection) {
    assert_eq!(role, Role::Server);

    let acceptor = tls.then(|| tls::build_acceptor(tls_server_config.unwrap()).unwrap());
    let server = TcpListener::bind((domain, port)).await.unwrap();

    while let Ok((stream, _)) = server.accept().await {
        let stream = if tls {
            MaybeTlsStream::ServerTls(tls::accept(acceptor.as_ref().unwrap(), stream).await.unwrap())
        } else {
            MaybeTlsStream::Plain(stream)
        };
        let websocket = WebSocketStream::from_raw_socket(stream, role, None).await;

        spawn(async move {
            // start example
            let (mut tx, mut rx) = websocket.split();
            println!("{:?}", rx.next().await.unwrap());
            tx.send(Message::Text("hello".to_owned())).await.unwrap();
        });
    }
}
