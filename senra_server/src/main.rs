use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use senra_server::{
    config::Config, db::Database, errors::Result, routes::create_router, state::AppState,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::default();

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("Server listening on {}", listener.local_addr().unwrap());

    let db = Database::new(&config).await?;
    let state = AppState::new(config, db);
    state.db.run_migrations().await?;

    axum::serve(listener, create_router(state)).await.unwrap();

    Ok(())
}
