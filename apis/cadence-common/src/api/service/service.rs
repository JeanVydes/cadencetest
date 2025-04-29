use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tracing::{error, info, trace, warn};

use crate::env::{load_enviroment_from_path, parse_environment_into_config};

use super::certs;

#[derive(Debug, Clone)]
pub enum ServiceError {
    NotFound,
    AlreadyExists,
    InvalidCreation,
    InvalidUpdate,
    InvalidDeletion,
    Internal,
    CertificateError(String),
    KeyError(String),
    EnviromentError(String),
    EnviromentParseError(String),
    TLSConfigError(String),
    SocketAddrNotDefined,
    ListenerError(String),
    ServerError(String),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::NotFound => write!(f, "Resource not found"),
            ServiceError::AlreadyExists => write!(f, "Resource already exists"),
            ServiceError::InvalidCreation => write!(f, "Invalid creation data"),
            ServiceError::InvalidUpdate => write!(f, "Invalid update data"),
            ServiceError::InvalidDeletion => write!(f, "Invalid deletion operation"),
            ServiceError::Internal => write!(f, "Internal server error"),
            ServiceError::CertificateError(s) => write!(f, "Certificate error: {}", s),
            ServiceError::KeyError(s) => write!(f, "Key error: {}", s),
            ServiceError::EnviromentError(s) => write!(f, "Environment configuration error: {}", s),
            ServiceError::EnviromentParseError(s) => write!(f, "Environment parsing error: {}", s),
            ServiceError::TLSConfigError(s) => write!(f, "TLS configuration error: {}", s),
            ServiceError::SocketAddrNotDefined => write!(f, "Socket address not defined"),
            ServiceError::ListenerError(s) => write!(f, "Listener error: {}", s),
            ServiceError::ServerError(s) => write!(f, "Server runtime error: {}", s),
        }
    }
}

#[derive(Clone)]
pub struct APIService<T>
where
    T: for<'de> Deserialize<'de>
        + EnviromentCommon
        + std::fmt::Debug
        + Clone
        + Send
        + Sync
        + 'static,
{
    pub metadata: APIServiceMetadata,
    pub app_root: Router,
    pub config: APIServiceConfig<T>,
    pub socket_addr: Option<SocketAddr>,
    pub listener: Option<Arc<tokio::net::TcpListener>>,
    pub tls_config: Option<Arc<rustls::ServerConfig>>,
    pub tls_acceptor: Option<Arc<tokio_rustls::TlsAcceptor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIServiceMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct APIServiceConfig<T> {
    pub enviroment: Option<T>,
}

pub trait EnviromentCommon {
    fn h2(&self) -> bool;
    fn h3(&self) -> bool;
    fn get_service_name(&self) -> String;
    fn get_service_description(&self) -> String;
    fn get_service_version(&self) -> String;
    fn get_address(&self) -> Option<String>;
    fn get_port(&self) -> Option<u16>;
    fn get_cert_path(&self) -> Option<String>;
    fn get_key_path(&self) -> Option<String>;
    fn get_service_metadata(&self) -> APIServiceMetadata {
        APIServiceMetadata {
            name: self.get_service_name(),
            version: self.get_service_version(),
            description: self.get_service_description(),
        }
    }
}

impl<
    T: for<'de> Deserialize<'de> + EnviromentCommon + std::fmt::Debug + Clone + Send + Sync + 'static,
> APIService<T>
{
    pub fn new(
        name: String,
        version: String,
        description: String,
        app_root: Router,
        config: APIServiceConfig<T>,
    ) -> Self {
        Self {
            metadata: APIServiceMetadata {
                name,
                version,
                description,
            },
            app_root,
            config,
            listener: None,
            tls_config: None,
            socket_addr: None,
            tls_acceptor: None,
        }
    }

    pub fn new_minimal() -> Self {
        Self {
            metadata: APIServiceMetadata {
                name: String::new(),
                version: String::new(),
                description: String::new(),
            },
            app_root: Router::new(),
            config: APIServiceConfig { enviroment: None },
            listener: None,
            tls_config: None,
            socket_addr: None,
            tls_acceptor: None,
        }
    }

    pub fn setup_certificates(
        &mut self,
    ) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), ServiceError> {
        let env = self.config.enviroment.as_ref().ok_or_else(|| {
            ServiceError::EnviromentError("Environment config not loaded".to_string())
        })?;

        let cert_path = env.get_cert_path().unwrap_or("".to_owned());
        let key_path = env.get_key_path().unwrap_or("".to_owned());
        // Generate or load certs/keys
        let (cert_bytes, key_bytes) = certs::generate_self_signed_cert(&cert_path, &key_path)
            .map_err(|e| {
                ServiceError::CertificateError(format!(
                    "Failed to generate/load cert/key files: {}",
                    e
                ))
            })?;

        trace!("Certificate bytes: {:?}", cert_bytes,);

        // Parse certs/keys
        let certs = certs::load_certs(&cert_bytes).map_err(|e| {
            ServiceError::CertificateError(format!("Failed to parse certificate bytes: {}", e))
        })?;
        let key = certs::load_key(&key_bytes)
            .map_err(|e| ServiceError::KeyError(format!("Failed to parse key bytes: {}", e)))?;

        trace!("Parsed certificate and key successfully: {:?}", certs,);

        Ok((certs, key))
    }

    pub fn setup_tls_config(&mut self) -> Result<Arc<rustls::ServerConfig>, ServiceError> {
        let (certs, key) = self.setup_certificates()?;

        let builder = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| {
                ServiceError::TLSConfigError(format!(
                    "Failed to create TLS config with cert/key: {}",
                    e
                ))
            })?;

        let mut tls_config = builder;
        let mut alpn_protocols = Vec::new();
        if self
            .config
            .enviroment
            .as_ref()
            .map_or(false, |env| env.h3())
        {
            info!("HTTP/3 is enabled in the environment configuration.");
            alpn_protocols.push(b"h3".to_vec());
        }

        if self
            .config
            .enviroment
            .as_ref()
            .map_or(false, |env| env.h2())
        {
            info!("HTTP/2 is enabled in the environment configuration.");
            alpn_protocols.push(b"h2".to_vec());
        }

        tls_config.alpn_protocols = alpn_protocols;

        let tls_arc = Arc::new(tls_config);
        self.tls_config = Some(tls_arc.clone());
        Ok(tls_arc)
    }

    /// Loads environment variables from the default `.env` file into the process environment.
    pub fn load_enviroment_default(&mut self) -> Result<(), dotenvy::Error> {
        load_enviroment_from_path::<T>(".env")
    }

    /// Loads environment variables from a specific `.env` file into the process environment.
    pub fn load_enviroment_from_path(&mut self, path: &str) -> Result<(), dotenvy::Error> {
        load_enviroment_from_path::<T>(path)
    }

    pub fn parse_environment_into_config(&mut self) -> Result<(), ServiceError> {
        parse_environment_into_config::<T>()
            .map_err(|e| {
                ServiceError::EnviromentParseError(format!(
                    "Failed to parse environment variables: {:?}",
                    e
                ))
            })
            .map(|loaded_config| {
                self.config.enviroment = Some(loaded_config);
            })
    }

    pub fn set_socket_addr(&mut self, socket_addr: SocketAddr) {
        self.socket_addr = Some(socket_addr);
    }

    pub fn try_set_socket_addr_from_env(&mut self) -> Result<(), ServiceError> {
        trace!("Attempting to set socket address from environment variables.");

        let env = self.config.enviroment.as_ref().ok_or_else(|| {
            ServiceError::EnviromentError("Environment config not loaded".to_string())
        })?;

        let address_str = env.get_address().ok_or_else(|| {
            ServiceError::EnviromentError("Address not found in environment".to_string())
        })?;
        let port = env.get_port().ok_or_else(|| {
            ServiceError::EnviromentError("Port not found in environment".to_string())
        })?;

        let ip_addr = address_str.parse::<std::net::IpAddr>().map_err(|e| {
            let err_msg = format!("Failed to parse IP address '{}': {}", address_str, e);
            error!("{}", err_msg); // Log as error
            ServiceError::EnviromentParseError(err_msg)
        })?;

        self.socket_addr = Some(SocketAddr::from((ip_addr, port)));

        info!(
            "Socket address set from environment: {:?}",
            self.socket_addr
        );

        Ok(())
    }

    pub async fn setup_listener_and_tls_acceptor(&mut self) -> Result<(), ServiceError> {
        info!(
            "Setting up listener and TLS acceptor for APIService on address: {:?}",
            self.socket_addr
        );

        let socket_addr = self.socket_addr.ok_or(ServiceError::SocketAddrNotDefined)?;

        let listener = tokio::net::TcpListener::bind(socket_addr)
            .await
            .map_err(|e| {
                let err_msg = format!("Failed to bind TCP listener to {}: {}", socket_addr, e);
                error!("{}", err_msg);
                ServiceError::ListenerError(err_msg)
            })?;
        info!("TCP Listener bound successfully to {}", socket_addr);
        self.listener = Some(Arc::new(listener));

        if let Some(tls_config) = self.tls_config.as_ref() {
            self.tls_acceptor = Some(Arc::new(tokio_rustls::TlsAcceptor::from(
                tls_config.clone(),
            )));
            info!("TLS Acceptor created successfully.");
        } else {
            warn!("No TLS config found, skipping TLS acceptor setup.");
        }

        Ok(())
    }

    pub async fn spawn_h1_server(&mut self) -> Result<(), ServiceError> {
        info!(
            "Spawning H1 server with axum-server on address: {:?}",
            self.socket_addr
        );

        let bind_addr = self.socket_addr.ok_or(ServiceError::SocketAddrNotDefined)?;

        let app_service = self
            .app_root
            .clone()
            .into_make_service_with_connect_info::<SocketAddr>();

        info!("Starting axum-server on address: {}", bind_addr);

        axum_server::bind(bind_addr)
            .serve(app_service)
            .await
            .map_err(|e| {
                let err_msg = format!("axum-server failed: {}", e);
                error!("{}", err_msg); // Log server failures as errors
                ServiceError::ServerError(err_msg)
            })?;

        Ok(())
    }

    /// Spawns the server using `axum-server`, handling HTTP/1.1 and HTTP/2 over TLS.
    pub async fn spawn_h1h2_server(&mut self) -> Result<(), ServiceError> {
        info!(
            "Spawning H1/H2 server with axum-server on address: {:?}",
            self.socket_addr
        );

        let bind_addr = self.socket_addr.ok_or(ServiceError::SocketAddrNotDefined)?;

        let tls_config = self
            .tls_config
            .as_ref()
            .ok_or_else(|| {
                ServiceError::TLSConfigError(
                    "TLS config not set up before spawning server".to_string(),
                )
            })?
            .clone();

        info!(
            "Setting up TLS configuration for axum-server with address: {}",
            bind_addr
        );

        let rustls_config = RustlsConfig::from_config(tls_config);
        let app_service = self
            .app_root
            .clone()
            .into_make_service_with_connect_info::<SocketAddr>();

        info!("Starting axum-server with Rustls on address: {}", bind_addr);

        axum_server::bind_rustls(bind_addr, rustls_config)
            .serve(app_service)
            .await
            .map_err(|e| {
                let err_msg = format!("axum-server failed: {}", e);
                error!("{}", err_msg); // Log server failures as errors
                ServiceError::ServerError(err_msg)
            })?;

        Ok(())
    }

    pub async fn spawn_h3_server(&mut self) {
        unimplemented!()
    }

    pub fn get_environment_config(&self) -> Option<&T> {
        self.config.enviroment.as_ref()
    }

    pub fn set_app_root(&mut self, app_root: Router) {
        self.app_root = app_root;
    }

    pub fn get_app_root(&self) -> &Router {
        &self.app_root
    }
}

pub struct APIServiceBuilder {}
