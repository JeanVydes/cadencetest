// certs.rs
use rcgen::generate_simple_self_signed;
use tracing::{info, warn};
use std::fs;
use std::io::Result;
use std::path::Path;

pub fn generate_self_signed_cert(cert_path: &str, key_path: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    if Path::new(cert_path).exists() && Path::new(key_path).exists() {
        let cert_pem = fs::read(cert_path)?;
        let key_pem = fs::read(key_path)?;
        return Ok((cert_pem, key_pem));
    }

    warn!("Certificate or key not found, generating new self-signed certificate");
    info!("Generating self-signed certificate");

    let subject_alt_names = vec!["cadence.local".to_string()];

    let cert = generate_simple_self_signed(subject_alt_names).unwrap();
    info!("Generated self-signed certificate: {:?}", cert.cert.pem());
    Ok((cert.cert.pem().into_bytes(), cert.key_pair.serialize_pem().into_bytes()))
}

pub fn load_certs(
    cert_pem: &[u8],
) -> std::io::Result<Vec<rustls::pki_types::CertificateDer<'static>>> {
    let mut reader = std::io::Cursor::new(cert_pem);
    rustls_pemfile::certs(&mut reader).collect()
}

pub fn load_key(key_pem: &[u8]) -> std::io::Result<rustls::pki_types::PrivateKeyDer<'static>> {
    let mut reader = std::io::Cursor::new(key_pem);
    rustls_pemfile::private_key(&mut reader).map(|key| key.unwrap()) // panic if key is invalid for this example
}
