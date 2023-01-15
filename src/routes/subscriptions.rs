use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>) -> HttpResponse {
    format!("Welcome {}, with email {}!", form.name, form.email);
    HttpResponse::Ok().finish()
}
