use once_cell::sync::Lazy;
use reqwest::{StatusCode, Url};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use sqlx::types::Uuid;
use events::application::{Application, get_connection_pool};
use events::configuration::{DatabaseSettings, get_configuration};
use events::routes::events::{Event, EventsQuery, QueryEventsResponse};
use events::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub api_client: reqwest::Client,
}

impl TestApp {
    pub async fn get_event_count(&self, events_query: &EventsQuery) -> QueryEventsResponse {
        let response = self.api_client
            .get(&format!("{}/event?{}", &self.address, &serde_qs::to_string(events_query).unwrap()))
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(StatusCode::from_u16(200).unwrap(), response.status());
        response.json().await.unwrap()
    }

    pub async fn post_event(&self, event: &Event) -> StatusCode {
        let response = self.api_client
            .post(&format!("{}/event", &self.address))
            .json(event)
            .send()
            .await
            .expect("Failed to send request.");
        response.status()
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };
    configure_database(&configuration.database).await;

    let application = Application::build(&configuration)
        .await
        .expect("Failed to build application");
    let port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();


    TestApp {
        address: format!("http://localhost:{}", port),
        port,
        db_pool: get_connection_pool(&configuration.database),
        api_client: client
    }
}