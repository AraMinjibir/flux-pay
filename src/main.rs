use actix_web::{App, HttpServer, web};
use flux_pay::config::{bootstrap::build_app_state, routes};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let state = web::Data::new(
        build_app_state()
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?,
    );

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .configure(routes::configure)
    })
    .bind(("127.0.0.1", 8080))?;

    info!("Server started at http://127.0.0.1:8080");

    server.run().await?;

    Ok(())
}
