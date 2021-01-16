use actix_web::{web};
use crate::controller::{index, account};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("api/v1")
        .route("", web::get().to(index::index))
        .route("register", web::post().to(account::register))
        .route("login", web::post().to(account::login))
        .route("otp/verify", web::post().to(account::verify_otp))
    );
}