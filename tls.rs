use std::{sync::Arc, path::Path, io::{self, BufReader}, fs::File};
use tokio_rustls::{TlsConnector, TlsAcceptor, rustls::{self, OwnedTrustAnchor, Certificate, PrivateKey}};

pub fn load_certs<P: AsRef<Path>>(path: P) -> io::Result<Vec<Certificate>> {
    let mut certs = rustls_pemfile::certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid certs"))?;
    Ok(certs.drain(..).map(Certificate).collect())
}

pub fn load_rsa_key<P: AsRef<Path>>(path: P) -> io::Result<PrivateKey> {
    let mut keys = rustls_pemfile::rsa_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))?;

    let key = keys.drain(..).map(PrivateKey).next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))?;

    Ok(key)
}

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

pub fn build_acceptor(key: PrivateKey, certs: Vec<Certificate>) -> io::Result<TlsAcceptor> {
    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}
