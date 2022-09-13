pub mod tls {
    use std::{sync::Arc, io::{self, BufReader}, fs::File};
    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio_rustls::{TlsConnector, TlsAcceptor, rustls::{self, OwnedTrustAnchor, Certificate, PrivateKey}, client, server};
    use rustls_pemfile::{certs, rsa_private_keys};
    use crate::TlsServerConfig;

    pub use rustls::ServerName;

    pub fn build_connector() -> TlsConnector {
        let mut root_cert_store = rustls::RootCertStore::empty();

        root_cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(
            |ta| {
                OwnedTrustAnchor::from_subject_spki_name_constraints(
                    ta.subject,
                    ta.spki,
                    ta.name_constraints,
                )
            },
        ));

        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        TlsConnector::from(Arc::new(config))
    }

    #[inline]
    pub async fn connect<S: AsyncRead + AsyncWrite + Unpin>(connector: &TlsConnector, domain: &str, stream: S) -> io::Result<client::TlsStream<S>> {
        let domain = rustls::ServerName::try_from(domain)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "invalid dns name"))?;
        connector.connect(domain, stream).await
    }

    pub fn build_acceptor(TlsServerConfig { key_path, certs_path }: TlsServerConfig) -> io::Result<TlsAcceptor> {
        let mut certs = certs(&mut BufReader::new(File::open(certs_path)?))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid certs"))?;

        let certs: Vec<_> = certs.drain(..).map(Certificate).collect();

        let mut keys = rsa_private_keys(&mut BufReader::new(File::open(key_path)?))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))?;

        let key = keys.drain(..).map(PrivateKey).next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))?;

        let config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

        Ok(TlsAcceptor::from(Arc::new(config)))
    }

    #[inline]
    pub async fn accept<S: AsyncRead + AsyncWrite + Unpin>(accpetor: &TlsAcceptor, stream: S) -> io::Result<server::TlsStream<S>> {
        let sess_acceptor = accpetor.clone();
        sess_acceptor.accept(stream).await
    }
}

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

use std::path::PathBuf;
use futures::{StreamExt, SinkExt};
use tokio::{spawn, net::{TcpStream, TcpListener}};
use tokio_tungstenite::{WebSocketStream, tungstenite::protocol::{Role, Message}};

pub async fn ws_client_example(Connection { role, tls, domain, port, .. }: Connection) {
    assert_eq!(tls, false);
    assert_eq!(role, Role::Client);
    let stream = TcpStream::connect((domain, port)).await.unwrap();
    let websocket = WebSocketStream::from_raw_socket(stream, role, None).await;
    // start example
    let (mut tx, mut rx) = websocket.split();
    tx.send(Message::Text("hi".to_owned())).await.unwrap();
    println!("{:?}", rx.next().await.unwrap());
}

pub async fn ws_server_example(Connection { role, tls, domain, port, .. }: Connection) {
    assert_eq!(tls, false);
    assert_eq!(role, Role::Server);
    let server = TcpListener::bind((domain, port)).await.unwrap();
    while let Ok((stream, _)) = server.accept().await {
        spawn(async move {
            let websocket = WebSocketStream::from_raw_socket(stream, role, None).await;
            // start example
            let (mut tx, mut rx) = websocket.split();
            println!("{:?}", rx.next().await.unwrap());
            tx.send(Message::Text("hello".to_owned())).await.unwrap();
        });
    }
}

pub async fn wss_client_example(Connection { role, tls, domain, port, .. }: Connection) {
    assert_eq!(tls, true);
    assert_eq!(role, Role::Client);
    let connector = tls::build_connector();
    let stream = TcpStream::connect((domain.as_str(), port)).await.unwrap();
    let stream = tls::connect(&connector, &domain, stream).await.unwrap();
    let websocket = WebSocketStream::from_raw_socket(stream, role, None).await;
    // start example
    let (mut tx, mut rx) = websocket.split();
    tx.send(Message::Text("hi".to_owned())).await.unwrap();
    println!("{:?}", rx.next().await.unwrap());
}

pub async fn wss_server_example(Connection { role, tls, domain, port, tls_server_config }: Connection) {
    assert_eq!(tls, true);
    assert_eq!(role, Role::Server);
    let accpetor = tls::build_acceptor(tls_server_config.unwrap()).unwrap();
    let server = TcpListener::bind((domain, port)).await.unwrap();
    while let Ok((stream, _)) = server.accept().await {
        let stream = tls::accept(&accpetor, stream).await.unwrap();
        spawn(async move {
            let websocket = WebSocketStream::from_raw_socket(stream, role, None).await;
            // start example
            let (mut tx, mut rx) = websocket.split();
            println!("{:?}", rx.next().await.unwrap());
            tx.send(Message::Text("hello".to_owned())).await.unwrap();
        });
    }
}
