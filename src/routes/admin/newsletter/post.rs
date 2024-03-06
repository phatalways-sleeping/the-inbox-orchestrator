use crate::authentication::UserId;
use crate::idempotency::{save_response, try_processing, IdempotencyKey};
use crate::utils::{e400, e500, redirect};
use actix_web::web::ReqData;
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
    idempotency_key: String,
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(form, pool, user_id),
    fields(user_id=%*user_id)
)]
pub async fn publish_newsletter(
    form: web::Form<FormData>,
    user_id: ReqData<UserId>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let FormData {
        title,
        text_content,
        html_content,
        idempotency_key,
    } = form.0;
    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;
    // try_processing will first insert into the idemptency table the value of user_id,
    // idempotency_key without the response data to handle concurrent requests
    // it then returns the transaction
    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e400)?
    {
        crate::idempotency::NextAction::StartProcessing(t) => t,
        // In the case the second request comes in, it will have to wait for the first request to commit
        // the transaction. After that, it will get the old response from the database saved by
        // the first request below to return back to the client
        // ensure idempotent operations for concurrent requests
        crate::idempotency::NextAction::ReturnedHttpResponse(response) => {
            success_message().send();
            return Ok(response);
        }
    };
    // To ensure fault tolerance, we have to use the forward recovery - active recovery in which
    // we limit the scope of our POST /admin/newsletter to asynchronously send issues to all emails in the background
    // instead of performing all the sending before responsing back to the users.
    let issue_id =
        insert_into_newsletter_issue(&mut transaction, &title, &html_content, &text_content)
            .await
            .map_err(e500)?;
    enqueue_delivery_task(&mut transaction, issue_id)
        .await
        .map_err(e500)?;
    success_message().send();
    let response = redirect("/admin/newsletters");
    // Here, we continue to use the transaction to update the response data in the table
    // and commit the transaction
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;
    Ok(response)
}

fn success_message() -> FlashMessage {
    FlashMessage::info("The newsletter issue has been accepted - emails will go out shortly.")
}

#[tracing::instrument(skip_all)]
async fn insert_into_newsletter_issue(
    transaction: &mut Transaction<'static, Postgres>,
    title: &str,
    html_content: &str,
    text_content: &str,
) -> Result<Uuid, anyhow::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter_issues (newsletter_issue_id, title, html_content, text_content, published_at)
        VALUES ($1, $2, $3, $4, now())
    "#,
        newsletter_issue_id,
        title,
        html_content,
        text_content,
    ).execute(&mut **transaction)
    .await.context("Failed to insert into newsletter_issue")?;
    Ok(newsletter_issue_id)
}

#[tracing::instrument(skip_all)]
async fn enqueue_delivery_task(
    transaction: &mut Transaction<'static, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        INSERT INTO issue_delivery_table (newsletter_issue_id, subscriber_email)
        SELECT $1, email
        FROM subscriptions
        WHERE status = 'confirmed'
    "#,
        newsletter_issue_id
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}
