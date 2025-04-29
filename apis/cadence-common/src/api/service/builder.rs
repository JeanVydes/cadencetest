use std::marker::PhantomData;
use std::net::SocketAddr;

use axum::Router;
use serde::Deserialize;
use tracing::{error, info};

use crate::api::service::service::{APIService, APIServiceMetadata, ServiceError};
use super::service::EnviromentCommon;


#[derive(Default)]
pub struct APIServiceBuilder<T>
where
    // Added Send + Sync as they are often needed and present on APIService
    T: for<'de> Deserialize<'de> + EnviromentCommon + std::fmt::Debug + Clone + Send + Sync + 'static + Default,
{
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    // If state is always intended, consider using Router<ApplicationState> here
    app_root: Option<Router>,
    env_path: Option<String>,
    socket_addr_override: Option<SocketAddr>,
    _phantom: PhantomData<T>,
}

impl<T> APIServiceBuilder<T>
where
    T: for<'de> Deserialize<'de> + EnviromentCommon + std::fmt::Debug + Clone + Send + Sync + 'static + Default,
{
    /// Creates a new builder instance.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the service name (required).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the service version (required).
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Sets the service description (required).
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the Axum router (required).
    // Consider changing the type to Router<ApplicationState> if appropriate
    pub fn router(mut self, router: Router) -> Self {
        self.app_root = Some(router);
        self
    }

    /// Specifies a custom path for the .env file (optional).
    pub fn env_path(mut self, path: impl Into<String>) -> Self {
        self.env_path = Some(path.into());
        self
    }

    /// Manually sets the socket address, overriding environment variables (optional).
    pub fn socket_addr(mut self, addr: SocketAddr) -> Self {
        self.socket_addr_override = Some(addr);
        self
    }

    /// Builds the `APIService`, performing all necessary setup steps.
    pub async fn build(self) -> Result<APIService<T>, ServiceError> {
        // --- 1. Validate required fields ---
        // Using specific errors or a BuilderError enum would be clearer than InvalidCreation
        let name = self.name.ok_or(ServiceError::InvalidCreation)?;
        let version = self.version.ok_or(ServiceError::InvalidCreation)?;
        let description = self.description.ok_or(ServiceError::InvalidCreation)?;
        let app_root = self.app_root.ok_or(ServiceError::InvalidCreation)?;

        // --- 2. Load and Parse Environment ---
        let mut temp_service = APIService::<T>::new_minimal();

        if let Some(path) = self.env_path {
            // Propagate the error properly instead of mapping to a generic one
            temp_service.load_enviroment_from_path(&path)
                .map_err(|e| ServiceError::EnviromentError(format!("Failed loading env from path '{}': {}", path, e)))?;
        } else {
            // Propagate the error properly
            temp_service.load_enviroment_default()
                 .map_err(|e| ServiceError::EnviromentError(format!("Failed loading default env: {}", e)))?;
            // Note: load_enviroment_default already handles 'not found' gracefully internally
        }

        // Propagate the error properly
        temp_service.parse_environment_into_config()?;

        // Ensure environment was actually parsed if needed by subsequent steps
        let config = temp_service.config;
        if config.enviroment.is_none() {
             // If subsequent steps *require* the environment config, error out here.
             // Otherwise, this check might be unnecessary.
             return Err(ServiceError::EnviromentParseError("Environment config failed to load or parse.".to_string()));
        }

        // --- 3. Create APIService instance ---
        let mut service = APIService {
            metadata: APIServiceMetadata {
                name,
                version,
                description,
            },
            app_root,
            config,
            socket_addr: None,
            listener: None,
            tls_config: None,
            tls_acceptor: None,
        };

        // --- 4. Set Socket Address ---
        if let Some(addr) = self.socket_addr_override {
            service.set_socket_addr(addr);
            info!("Using overridden socket address: {}", addr);
        } else {
            service.try_set_socket_addr_from_env()?;
            info!(
                "Socket address set from environment: {:?}",
                service.socket_addr
            );
        }

        // Ensure socket address is now set
        if service.socket_addr.is_none() {
            // Use error level for failures
            error!("Socket address could not be determined from override or environment.");
            return Err(ServiceError::SocketAddrNotDefined);
        }

        if let Some(ref env) = service.config.enviroment {
            if env.h2() || env.h3() {
                info!("Setting up TLS configuration...");
                service.setup_tls_config()?;
            } else {
                info!("TLS is disabled in the environment configuration.");
            }
        }

        info!(
            "APIService build successful for '{}' v{}",
            service.metadata.name, service.metadata.version
        );
        Ok(service)
    }
}
