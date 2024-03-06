use actix_web::{http::header::ContentType, web, HttpResponse};
use anyhow::{Context, Ok};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{authentication::UserId, utils::e500};

pub async fn admin_dashboard(
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let username = get_username(*user_id.into_inner(), &pool)
        .await
        .map_err(e500)?;
    core::result::Result::Ok(
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(format!(
                r#"<!DOCTYPE html>
                <html lang="en">
                <head>
                <meta http-equiv="content-type" content="text/html; charset=utf-8">
                <title>Admin dashboard</title>
                </head>
                <body>
                <p>Welcome {username}!</p>
                <p>Available actions:</p>
                <ol>
                <li><a href="/admin/password">Change password</a></li>
                <li>
                <form name="logoutForm" action="/admin/logout" method="post">
                <input type="submit" value="Logout">
                </form>
                </li>
                </ol>
                </body>
                </html>"#,
            )),
    )
}

#[tracing::instrument(name = "Getting a username", skip(pool))]
pub async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
    let record = sqlx::query!(
        r#"
    SELECT username
    FROM users
    WHERE user_id = $1"#,
        user_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to retrieve username from user id")?;
    Ok(record.username)
}
