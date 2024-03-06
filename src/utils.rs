use core::fmt;

use actix_web::HttpResponse;
use reqwest::header::LOCATION;

pub fn e500<T>(e: T) -> actix_web::error::Error
where
    T: fmt::Display + fmt::Debug + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

pub fn e400<T: std::fmt::Debug + std::fmt::Display>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorBadRequest(e)
}

pub fn redirect(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}
