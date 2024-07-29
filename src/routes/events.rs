use std::time::{SystemTime, UNIX_EPOCH};
use actix_web::{web, Result, HttpResponse, ResponseError, Responder};
use actix_web::http::StatusCode;
use anyhow::Context;
use log::debug;
use sqlx::{Execute, Executor, PgPool, Postgres, QueryBuilder, Row, Transaction};
use crate::routes::events::QueryEventsError::ValidationError;
use crate::util::SnowflakeCreator::SnowflakeCreator;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Event {
    id: String,
    key_value: Vec<EventKeyValue>,
}

impl Event {
    pub fn new(id: String, key_value: Vec<EventKeyValue>) -> Event {
        Event {
            id,
            key_value
        }
    }
}
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct EventKeyValue {
    key: String,
    value: Option<String>
}

impl EventKeyValue {
    pub fn new(key: String, value: Option<String>) -> EventKeyValue {
        EventKeyValue {
            key,
            value
        }
    }
}

#[derive(thiserror::Error)]
pub enum EventError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}


impl std::fmt::Debug for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for EventError {
    fn status_code(&self) -> StatusCode {
        match self {
            EventError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}



#[tracing::instrument(
    name = "Create an event"
)]
pub async fn create_event(
    event: web::Json<Event>,
    pool: web::Data<PgPool>,
    snowflake_creator: web::Data<SnowflakeCreator>,
) -> Result<HttpResponse, EventError> {
    debug!("received event: {:?}", event);

    let id = snowflake_creator
        .create_id(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Failed to get epoch ms")
                .as_secs() * 1000
        );
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a connection from postgres")?;

    insert_event(&mut transaction, &id, &event).await.context("Failed to create an event")?;
    insert_key_value(&mut transaction, &id, &event).await.context("Failed to create a key value for the event")?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new event.")?;

    Ok(
        HttpResponse::Created().body("")
    )
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct EventsQuery {
    // TODO: Might want to support other fields, limit, etc.
    // But don't do it in this project~
    from: u64,
    to: u64,
    // limit: Option<u32>,
    id: String,
    key: String,
    // last_id: Option<u64>,
    // order_by: Option<SortOrder>,
    query_type: QueryType,
}

impl EventsQuery {
    pub fn new(from: u64, to: u64, id: String, key: String, query_type: QueryType) -> EventsQuery {
        EventsQuery {
            from: from,
            to: to,
            id: id,
            key: key,
            query_type: query_type
        }
    }
}

// #[derive(serde::Deserialize, Debug)]
// #[serde(rename_all = "lowercase")]
// enum SortOrder{
//     ASC,
//     DESC
// }
//
// impl SortOrder {
//     pub fn get_order_string(&self) -> String {
//         match self {
//             SortOrder::ASC => String::from("ASC"),
//             SortOrder::DESC => String::from("DESC")
//         }
//     }
// }


#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum QueryType{
    COUNT
}

#[derive(thiserror::Error, Debug)]
pub enum QueryEventsError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for QueryEventsError {
    fn status_code(&self) -> StatusCode {
        match self {
            QueryEventsError::ValidationError(_) => StatusCode::BAD_REQUEST,
            QueryEventsError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct QueryEventsResponse {
    count: u64
}

impl QueryEventsResponse {
    pub fn get_count(&self)-> u64 {
        self.count
    }
}

#[tracing::instrument(
    name = "query events"
)]
pub async fn get_query_events(
    query: web::Query<EventsQuery>,
    pool: web::Data<PgPool>,
    snowflake_creator: web::Data<SnowflakeCreator>,
) -> Result<impl Responder, QueryEventsError> {
    debug!("received query: {:?}", query);
    if query.from > query.to {
        return Err(ValidationError(String::from("From must be less than to.")))
    }

    let count = query_events(&pool, &query, &snowflake_creator).await?;

    Ok(web::Json(QueryEventsResponse { count }))
}


pub async fn insert_event(
    transaction: &mut Transaction<'_, Postgres>,
    snowflake: &u64,
    event: &Event
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"INSERT INTO events (id, external_id) VALUES ($1, $2)"#,
        *snowflake as i64,
        event.id,
    );
    transaction.execute(query).await?;
    Ok(())
}

pub async fn insert_key_value(
    transaction: &mut Transaction<'_, Postgres>,
    snowflake: &u64,
    event: &Event
) -> Result<(), sqlx::Error> {
    for key_value in &event.key_value {
        let query = sqlx::query!(
            r#"INSERT INTO events_key_value (events_id, key, value) VALUES ($1, $2, $3)"#,
            *snowflake as i64,
            key_value.key,
            key_value.value
        );
        transaction.execute(query).await?;
    }
    Ok(())
}


pub async fn query_events(
    pool: &PgPool,
    events_query: &EventsQuery,
    snowflake_creator: &SnowflakeCreator
)-> Result<u64, anyhow::Error> {
    // TODO: You might want to handle queries like give me the top 5 most clicked events.
    // TODO: Handle other query types.
    // This is not a real project and to save time, we'll only perform a count.
    // We'll just handle count for now.
    // refer to https://docs.rs/sqlx/latest/sqlx/struct.QueryBuilder.html
    let mut query = QueryBuilder::new(
        "SELECT COUNT(events.external_id), events.external_id AS events_count FROM events JOIN events_key_value as ekv ON ekv.events_id = events.id"
    );
    query.push(" WHERE ");
    query.push(" events.id >= ");
    query.push_bind(snowflake_creator.convert_time_to_snowflake(&events_query.from)  as i64);

    query.push(" AND ");
    query.push(" events.id < ");
    // add a second.
    query.push_bind(snowflake_creator.convert_time_to_snowflake(&(events_query.to)) as i64);

    // let order = match &events_query.order_by {
    //     None => { &SortOrder::ASC}
    //     Some(order) => order
    // };
    //
    // if let Some(last_id) = events_query.last_id {
    //     if matches!(order, SortOrder::DESC) {
    //         query.push(" AND events.id > ");
    //         query.push_bind(last_id as i64);
    //     } else {
    //         query.push(" AND events.id < ");
    //         query.push_bind(last_id as i64);
    //     }
    // }
    query.push( " AND events.external_id = ");
    query.push_bind(events_query.id.clone());
    query.push( " AND ekv.key = ");
    query.push_bind(events_query.key.clone());
    query.push(" GROUP BY events.external_id ");
    let sql = query.build();
    tracing::debug!("sql we will perform: {}", sql.sql());
    println!("{}", sql.sql());
    match sql.fetch_optional(pool).await.context("Failed to perform events query")? {
        None => {
            Ok(0)
        }
        Some(result) => {
            let count: i64 = result.try_get(0)?;
            Ok(count as u64)
        }
    }
}