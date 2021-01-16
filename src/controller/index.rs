use actix_web::{Responder, HttpResponse};
use crate::controller::ResponseWrapper;

pub async fn index() -> impl Responder {
    let response = ResponseWrapper::<String>::message_response("Welcome to chat api.", 200);
    HttpResponse::Ok()
        .json(response)
}