use std::{sync::Arc, time::Duration};

use axum::{
    Router, middleware,
    routing::{get, patch, post},
};
use cadence_common::repository_traits::BasicApplicationService;
use cadence_common::{
    api::state::{ApplicationState, Services},
    entities::util::create_tables_if_not_exists,
    env::{load_enviroment_from_path, parse_environment_into_config},
    logging::start_logging_subscriber,
};
use jsonwebtoken::Algorithm;
use middlewares::auth::require_authentication;
use nervio_limiter::{
    limiter::{BucketConfig, LimitEntityType, Limiter},
    middleware::axum::axum_limiter_middleware,
};
use sea_orm::DatabaseConnection;
use service::{Enviroment, LimiterBuckets, ServiceState};
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
};
use tracing::Level;

pub mod controllers;
pub mod middlewares;
pub mod responses;
pub mod service;

pub async fn setup_essentials()
-> Result<(Enviroment, DatabaseConnection), Box<dyn std::error::Error>> {
    start_logging_subscriber(Level::TRACE);
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    load_enviroment_from_path::<Enviroment>("dev.env")
        .expect("Failed to load environment variables");
    let env = parse_environment_into_config::<Enviroment>()
        .expect("Failed to parse environment variables");

    let db_connection = match sea_orm::Database::connect(env.postgres_uri.clone()).await {
        Ok(connection) => connection,
        Err(err) => {
            tracing::error!("Failed to connect to the database: {}", err);
            std::process::exit(1);
        }
    };

    create_tables_if_not_exists(&db_connection)
        .await
        .expect("Failed to create tables");

    return Ok((env, db_connection));
}

pub fn setup_limiter() -> (Arc<tokio::sync::Mutex<Limiter>>, BucketConfig) {
    let limiter = Arc::new(tokio::sync::Mutex::new(Limiter::builder().build()));
    let bucket_config = BucketConfig {
        name: "service_global".to_string(),
        limit_by: LimitEntityType::ProxiedIP,
        max_requests_per_cycle: 20,
        cycle_duration: Duration::from_secs(60),
    };

    return (limiter, bucket_config);
}

pub fn build_service_state(
    env: &Enviroment,
    db_connection: &DatabaseConnection,
    limiter: Arc<tokio::sync::Mutex<Limiter>>,
    bucket_config: &BucketConfig,
    token_algorithm: Algorithm,
) -> Arc<ApplicationState<ServiceState>> {
    let state = Arc::new(ApplicationState {
        services: Services {
            account_service: cadence_common::entities::services::account::AccountService::new(
                db_connection.clone(),
            ),
        },
        databases: cadence_common::api::state::Databases {
            postgres_connection: Arc::new(tokio::sync::Mutex::new(db_connection.clone())),
        },
        internal: ServiceState {
            env: env.clone(),
            limiter: limiter.clone(),
            limiter_buckets: LimiterBuckets {
                global: bucket_config.clone(),
            },
            token_algorithm,
        },
    });

    return state;
}

pub fn build_router(
    limiter: Arc<tokio::sync::Mutex<Limiter>>,
    bucket_config: BucketConfig,
    state: Arc<ApplicationState<ServiceState>>
) -> Router {
    Router::new()
        .route(
            "/auth/token",
            post(controllers::auth::request_token::request_token_controller),
        )
        .route(
            "/auth/token",
            get(controllers::auth::validate_token::validate_token_controller).route_layer(
                middleware::from_fn_with_state(state.clone(), require_authentication),
            ),
        )
        .route(
            "/account",
            get(controllers::get_account::get_account_controller)
                .post(controllers::create_account::create_account_controller)
                .delete(controllers::delete_account::delete_account_controller),
        )
        .route(
            "/account",
            patch(controllers::update_account::update_account_controller).route_layer(
                middleware::from_fn_with_state(state.clone(), require_authentication),
            ),
        )
        .route(
            "/accounts",
            get(controllers::get_accounts::get_accounts_controller),
        )
        .with_state(state)
        .layer(middleware::from_fn_with_state(
            (limiter.clone(), bucket_config),
            axum_limiter_middleware,
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_credentials(false),
        )
        .layer(RequestBodyLimitLayer::new(4096))
}
