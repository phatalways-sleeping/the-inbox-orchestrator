use actix_web::HttpResponse;
use actix_web_flash_messages::FlashMessage;

use crate::{session_state::TypedSession, utils::redirect};

pub async fn logout(session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    if let Ok(wrapped) = session.get_user_id() {
        if wrapped.is_some() {
            session.log_out();
            FlashMessage::info("You have successfully logged out.").send();
        }
    }
    Ok(redirect("/login"))
}
