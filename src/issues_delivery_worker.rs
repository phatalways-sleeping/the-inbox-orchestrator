use std::time::Duration;

use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use tracing::{field::display, Span};
use uuid::Uuid;

use crate::{
    configurations::Settings, domain::SubscriberEmail, email_client::EmailClient,
    startup::get_connection_pool,
};

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration);

    let email_client = configuration.email_client.client();

    worker_loop(connection_pool, email_client).await
}

async fn worker_loop(pool: PgPool, email_client: EmailClient) -> Result<(), anyhow::Error> {
    loop {
        match try_execute_task(&pool, &email_client).await {
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Ok(ExecutionOutcome::TaskCompleted) => {}
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

#[tracing::instrument(
    skip_all,
    fields(
        newsletter_issue_id=tracing::field::Empty,
        subscriber_email=tracing::field::Empty
    ),
    err
)]
pub async fn try_execute_task(
    pool: &PgPool,
    email_client: &EmailClient,
) -> Result<ExecutionOutcome, anyhow::Error> {
    if let Some((mut transaction, newsletter_issue_id, subscriber_email)) =
        dequeue_task(pool).await?
    {
        Span::current()
            .record("newsletter_issue_id", &display(newsletter_issue_id))
            .record("subscriber_email", &display(&subscriber_email));

        let newsletter_issue = get_issue(pool, &newsletter_issue_id).await?;

        match SubscriberEmail::parse(subscriber_email) {
            Ok(subscriber) => {
                if let Err(_) = email_client
                    .send_email(
                        &subscriber,
                        &newsletter_issue.title,
                        &newsletter_issue.html_content,
                        &newsletter_issue.text_content,
                    )
                    .await
                    .context(format!(
                        "Failed to send newsletter issue to {}",
                        subscriber.as_ref()
                    ))
                {
                    update_task_retry(&mut transaction, &newsletter_issue_id, subscriber.as_ref())
                        .await
                        .context("Failed to update task retries")?;
                }
                delete_task(&mut transaction, &newsletter_issue_id, subscriber.as_ref()).await?;
                remove_non_complete_tasks(transaction)
                    .await
                    .context("Failed to remove tasks where n_retries > 10")?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    error.message = %error,
                    "Skipping a confirmed subscriber. Their stored contact details are invalid",
                );
            }
        }
        Ok(ExecutionOutcome::TaskCompleted)
    } else {
        Ok(ExecutionOutcome::EmptyQueue)
    }
}

type PgTransaction = Transaction<'static, Postgres>;

#[tracing::instrument(skip_all)]
async fn dequeue_task(
    pool: &PgPool,
) -> Result<Option<(PgTransaction, Uuid, String)>, anyhow::Error> {
    let mut transaction = pool.begin().await?;

    match sqlx::query!(
        r#"
    SELECT newsletter_issue_id, subscriber_email
    FROM issue_delivery_table
    WHERE execute_after <= now()
    FOR UPDATE
    SKIP LOCKED
    LIMIT 1
"#
    )
    .fetch_optional(&mut *transaction)
    .await?
    {
        Some(r) => Ok(Some((
            transaction,
            r.newsletter_issue_id,
            r.subscriber_email,
        ))),
        None => Ok(None),
    }
}

#[tracing::instrument(skip_all)]
async fn delete_task(
    transaction: &mut PgTransaction,
    newsletter_issue_id: &Uuid,
    subscriber_email: &str,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        DELETE FROM issue_delivery_table
        WHERE newsletter_issue_id = $1 AND subscriber_email = $2
    "#,
        newsletter_issue_id,
        subscriber_email
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

struct NewsletterIssue {
    title: String,
    html_content: String,
    text_content: String,
}

#[tracing::instrument(skip_all)]
async fn get_issue(
    pool: &PgPool,
    newsletter_issue_id: &Uuid,
) -> Result<NewsletterIssue, anyhow::Error> {
    let newsletter_issue = sqlx::query_as!(
        NewsletterIssue,
        r#"
            SELECT title, html_content, text_content
            FROM newsletter_issues
            WHERE newsletter_issue_id = $1
        "#,
        newsletter_issue_id
    )
    .fetch_one(pool)
    .await?;
    Ok(newsletter_issue)
}

#[tracing::instrument(skip_all)]
async fn update_task_retry(
    transaction: &mut PgTransaction,
    newsletter_issue_id: &Uuid,
    subscriber_email: &str,
) -> Result<(), anyhow::Error> {
    // When the attempt to send email fails, we update
    // n_retries and execute_after to 1 minutes later
    sqlx::query!(
        r#"
        UPDATE issue_delivery_table
        SET n_retries = n_retries + 1,
            execute_after = execute_after + (1 * interval '1 minute')
        WHERE newsletter_issue_id = $1 AND subscriber_email = $2
    "#,
        newsletter_issue_id,
        subscriber_email
    )
    .execute(&mut **transaction)
    .await
    .context("Failed to execute update query")?;
    Ok(())
}

async fn remove_non_complete_tasks(mut transaction: PgTransaction) -> Result<(), anyhow::Error> {
    // Remove all the tasks whose n_retries > 10
    sqlx::query!(
        r#"
        DELETE FROM issue_delivery_table
        WHERE n_retries > 10
    "#
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(())
}
