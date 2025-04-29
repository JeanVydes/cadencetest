use std::sync::Arc;

use cadence_common::{api::service::service::EnviromentCommon, token::token::TokenService};
use jsonwebtoken::Algorithm;
use nervio_limiter::limiter::{BucketConfig, Limiter};
use serde::Deserialize;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Enviroment {
    pub h2: bool,
    pub h3: bool,
    pub address: Option<String>,
    pub port: Option<u16>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,

    pub service_name: String,
    pub service_description: String,
    pub service_version: String,

    pub postgres_uri: String,
    pub tokens_key: String,
}

impl EnviromentCommon for Enviroment {
    fn h2(&self) -> bool {
        self.h2
    }

    fn h3(&self) -> bool {
        self.h3
    }

    fn get_address(&self) -> Option<String> {
        self.address.clone()
    }

    fn get_port(&self) -> Option<u16> {
        self.port
    }

    fn get_cert_path(&self) -> Option<String> {
        self.cert_path.clone()
    }

    fn get_key_path(&self) -> Option<String> {
        self.key_path.clone()
    }

    fn get_service_name(&self) -> String {
        self.service_name.clone()
    }

    fn get_service_description(&self) -> String {
        self.service_description.clone()
    }

    fn get_service_version(&self) -> String {
        self.service_version.clone()
    }
}

pub struct ServiceState {
    pub env: Enviroment,
    pub limiter: Arc<Mutex<Limiter>>,
    pub limiter_buckets: LimiterBuckets,
    pub token_algorithm: Algorithm,
}

impl ServiceState {
    pub fn get_token_service(&self) -> TokenService {
        TokenService {
            algorithm: self.token_algorithm.clone(),
            key: self.env.tokens_key.clone(),
        }
    }
}

pub struct LimiterBuckets {
    pub global: BucketConfig,
}
