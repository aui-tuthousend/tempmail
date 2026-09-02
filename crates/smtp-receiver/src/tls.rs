use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

pub fn tls_acceptor(cert_path: &str, key_path: &str) -> Result<TlsAcceptor> {
    let certs = load_certs(cert_path)?;
    let key = load_private_key(key_path)?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("failed to build TLS server config")?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}

fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path).with_context(|| format!("failed to open TLS cert: {path}"))?;
    let mut reader = BufReader::new(file);
    rustls_pemfile::certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("failed to read TLS certs")
}

fn load_private_key(path: &str) -> Result<PrivateKeyDer<'static>> {
    let file = File::open(path).with_context(|| format!("failed to open TLS key: {path}"))?;
    let mut reader = BufReader::new(file);
    rustls_pemfile::private_key(&mut reader)
        .context("failed to read TLS private key")?
        .ok_or_else(|| anyhow!("TLS private key is empty"))
}
