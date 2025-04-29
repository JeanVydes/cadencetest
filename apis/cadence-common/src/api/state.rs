use std::sync::Arc;

use tokio::sync::Mutex;

use crate::entities::services::account::AccountService;

#[derive(Clone, Debug)]
pub struct ApplicationState<I> {
    pub services: Services,
    pub databases: Databases,
    pub internal: I,
}

#[derive(Clone, Debug)]
pub struct Services {
    pub account_service: AccountService,
}

#[derive(Clone, Debug)]
pub struct Databases {
    pub postgres_connection: Arc<Mutex<sea_orm::DatabaseConnection>>,
    //pub redis_connection: Option<sea_orm::DatabaseConnection>,
}
