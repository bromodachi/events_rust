use actix_web::http::StatusCode;
use events::routes::events::{Event, EventKeyValue, EventsQuery, QueryType};
use crate::helpers::spawn_app;

#[tokio::test]
async fn check_count_for_non_existent() {
    let app = spawn_app().await;
    let query = EventsQuery::new(
        1722006000000,
        1722092399000,
        String::from("XiEez"),
        String::from("click"),
        QueryType::COUNT
    );
    let response = app.get_event_count(&query).await;
    assert_eq!(0, response.get_count());
}

#[tokio::test]
async fn check_count_after_creation() {
    let app = spawn_app().await;
    let event = Event::new(
        String::from("XiEez"),
        vec![
            EventKeyValue::new(String::from("click"), None)
        ]
    );
    let status_code = app.post_event(&event).await;
    assert_eq!(StatusCode::from_u16(201).unwrap(), status_code);

    let query = EventsQuery::new(
        0,
        i64::MAX as u64,
        String::from("XiEez"),
        String::from("click"),
        QueryType::COUNT
    );
    let response = app.get_event_count(&query).await;
    assert_eq!(1, response.get_count());
}