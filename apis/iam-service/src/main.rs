use axum::Router;
use cadence_common::api::service::builder::APIServiceBuilder;
use cadence_common::api::service::service::EnviromentCommon;
use iam_service_lib::service::Enviroment;
use iam_service_lib::{build_router, build_service_state, setup_essentials, setup_limiter};
use jsonwebtoken::Algorithm;
use tracing::info;

#[tokio::main]
async fn main() {
    let (env, db_connection) = setup_essentials()
        .await
        .expect("Failed to setup essentials");

    tracing::info!("Database connection established and tables created.");
    tracing::info!("Starting Cadence IAM Service...");

    let (limiter, bucket_config) = setup_limiter();

    info!("Application state initialized.");

    let state = build_service_state(
        &env,
        &db_connection,
        limiter.clone(),
        &bucket_config,
        Algorithm::HS256,
    );

    let app: Router = build_router(limiter, bucket_config, state);

    info!("Router initialized.");

    let mut service = APIServiceBuilder::<Enviroment>::new()
        .name(env.get_service_name())
        .version(env.get_service_version())
        .description(env.get_service_description())
        .env_path("dev.env")
        .router(app)
        .build()
        .await
        .expect("Failed to build service");

    info!("Service initialized.");
    info!("Starting service...");

    service
        .spawn_h1_server()
        .await
        .expect("Failed to spawn H1/H2 server");
}
