use actix_web::dev::Server;
use std::net::TcpListener;
use actix_web::{App, HttpServer, web};
use actix_web::web::Data;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tracing_actix_web::TracingLogger;
use crate::configuration::{DatabaseSettings, Settings, SnowflakeConfig};
use crate::routes::events::{create_event, get_query_events};
use crate::routes::health_check::health_check;
use crate::util::SnowflakeCreator::SnowflakeCreator;

pub struct Application {
    port: u16,
    server: Server,
}
pub struct ApplicationBaseUrl(pub String);

impl Application {
    pub async fn build(configuration: &Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            &configuration.snowflake
        ).await?;

        Ok(Self { port, server})
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    snowflake_config: &SnowflakeConfig,
) -> Result<Server, anyhow::Error> {
    let db_pool = Data::new(db_pool);
    let snowflake = Data::new(
        SnowflakeCreator::new(
            snowflake_config.worker_id as u64,
            snowflake_config.process_id as u64,
            snowflake_config.start_millis,
            0
        )
    );
    let server: Server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/event", web::post().to(create_event))
            .route("/event", web::get().to(get_query_events))
            .app_data(db_pool.clone())
            .app_data(snowflake.clone())
    }).listen(listener)?.run();
    Ok(server)
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configuration.with_db())
}