#[cfg(any(feature = "client-tls-helper", feature = "server-tls-helper"))]
pub mod tls;
mod stream;

pub use tokio_tungstenite::{WebSocketStream, tungstenite};
pub use crate::stream::MaybeTlsStream;

use std::io;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::{TlsConnector, TlsAcceptor};
use tungstenite::protocol::{Role, WebSocketConfig};

pub async fn connect<S>(
    stream: S,
    tls_connector: Option<&TlsConnector>,
    tls_domain: Option<&str>,
    config: Option<WebSocketConfig>
) -> io::Result<WebSocketStream<MaybeTlsStream<S>>>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let stream = if let Some(connector) = tls_connector {
        let domain = tokio_rustls::rustls::ServerName::try_from(tls_domain.unwrap())
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "invalid dns name"))?;
        MaybeTlsStream::ClientTls(connector.connect(domain, stream).await?)
    } else {
        MaybeTlsStream::Plain(stream)
    };
    Ok(WebSocketStream::from_raw_socket(stream, Role::Client, config).await)
}

pub async fn accept<S>(
    stream: S,
    tls_acceptor: Option<&TlsAcceptor>,
    config: Option<WebSocketConfig>
) -> io::Result<WebSocketStream<MaybeTlsStream<S>>>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let stream = if let Some(acceptor) = tls_acceptor {
        let sess_acceptor = acceptor.clone();
        MaybeTlsStream::ServerTls(sess_acceptor.accept(stream).await?)
    } else {
        MaybeTlsStream::Plain(stream)
    };
    Ok(WebSocketStream::from_raw_socket(stream, Role::Server, config).await)
}
