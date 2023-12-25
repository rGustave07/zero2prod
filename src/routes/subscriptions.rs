use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub fn parse_subscriber(form_data: FormData) -> Result<NewSubscriber, String> {
    let name = SubscriberName::parse(form_data.name)?;
    let email = SubscriberEmail::parse(form_data.email)?;

    Ok(NewSubscriber { email, name })
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;

        Ok(Self { email, name })
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool),
)]
// pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
pub async fn insert_subscriber(pool: &PgPool, new_subscriber: &NewSubscriber) -> Result<(), sqlx::Error> {
    let utc_now = Utc::now();
    let unique_id = Uuid::new_v4();

    sqlx::query!(
        r#"
                INSERT INTO subscriptions (id, email, name, subscribed_at)
                VALUES ($1, $2, $3, $4)
            "#,
        unique_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        utc_now
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
        // Using the `?` operator to return early
        // if the function failed
    })?;

    Ok(())
}

#[
    tracing::instrument(
        name = "Adding a new subscriber",
        skip(form, pool),
        fields(
            subscriber_email = %form.email,
            subscriber_name = %form.name
        )
    )
]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish()
    };

    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
