use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    authentication::{validate_credentials, Credentials, UserId},
    routes::admin::dashboard::get_username,
    utils::{e500, redirect},
};

#[derive(Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    form: web::Form<FormData>,
    // session: TypedSession,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error(
            "You entered two different new passwords - the field values must match.",
        )
        .send();
        return Ok(redirect("/admin/password"));
    }

    let username = get_username(*user_id, &pool).await.map_err(e500)?;

    if let Err(e) = validate_credentials(
        Credentials {
            username,
            password: form.0.current_password,
        },
        &pool,
    )
    .await
    {
        return match e {
            crate::authentication::AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(redirect("/admin/password"))
            }
            crate::authentication::AuthError::UnexpectedError(_) => Err(e500(e).into()),
        };
    }

    if form.0.new_password.expose_secret().len() < 12
        || form.0.new_password.expose_secret().len() > 128
    {
        FlashMessage::error("The new password is too weak").send();
        return Ok(redirect("/admin/password"));
    }

    crate::authentication::change_password(*user_id, form.0.new_password, &pool)
        .await
        .map_err(e500)?;
    FlashMessage::error("Your password has been changed.").send();
    Ok(redirect("/admin/password"))
}
