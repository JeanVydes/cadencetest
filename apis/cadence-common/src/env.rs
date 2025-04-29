use std::path::Path;

use tracing::{error, info};

use crate::error::ServerError;

pub fn load_enviroment_from_path<T>(path: &str) -> Result<(), dotenvy::Error> {
    match dotenvy::from_path(Path::new(path)) {
        Ok(_) => {
            info!("Loaded .env file from specific path: {}", path);
            Ok(())
        }
        Err(e) => {
            info!("Failed to load .env file from path: {}", path);
            Err(e)
        }
    }
}

pub fn parse_environment_into_config<T>() -> Result<T, ServerError>
where
    T: serde::de::DeserializeOwned,
    {
    match envy::from_env::<T>() {
        Ok(loaded_config) => {
            info!("Successfully parsed environment variables into config struct.");
            Ok(loaded_config)
        }
        Err(e) => {
            let err_msg = format!("Failed to parse environment variables: {}", e);
            error!("{}", err_msg);
            Err(ServerError::EnviromentParseError(err_msg))
        }
    }
}
