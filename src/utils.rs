use core::fmt;

use actix_web::HttpResponse;
use reqwest::header::LOCATION;

pub fn e500<T>(e: T) -> actix_web::error::Error
where
    T: fmt::Display + fmt::Debug + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

pub fn redirect(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}
