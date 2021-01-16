use serde::{Deserialize};
use actix_web::{
    web::{Data, Json},
    Responder,
    HttpResponse,
};
use crate::{
    AppState, controller::ResponseWrapper,
    service::{
        user_service::{User},
        email_service,
    },
};
use regex::Regex;
use std::collections::HashMap;
use rand::Rng;
use log::{error, info, debug};

use chrono::{prelude::*};

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$").unwrap();
    static ref OTP_REGEX: Regex = Regex::new(r"^[0-9]{6}$").unwrap();
}


#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    email: Option<String>,
    full_name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    email: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct VerifyOtpRequest {
    user_id: Option<String>,
    otp: Option<String>,
}

/// Registers a new user
///
/// POST /api/v1/register
/// Content-Type: application/json
/// Body:
/// `
/// {
/// 	"email": "johnjokoo@gmail.com",
/// 	"full_name": "John Doe"
/// }
/// `
///
pub async fn register(data: Data<AppState>, user: Json<RegisterRequest>) -> impl Responder {
    let user_service = &data.service_container.user_service;

    let mut errors = HashMap::new();
    let email = match &user.email {
        Some(email) if EMAIL_REGEX.is_match(email) => {
            // check if email already exists
            let email = email.trim();
            let user = user_service.find_user_by_email(email).await;
            match user {
                Ok(Some(_)) => {
                    errors.insert("email", "Email address already exists.");
                }
                _ => ()
            };
            email
        }
        _ => {
            errors.insert("email", "Email address is required");
            ""
        }
    };
    let full_name = match &user.full_name {
        Some(full_name) if !full_name.is_empty() && full_name.trim().len() >= 3 => {
            full_name
        }
        _ => {
            errors.insert("full_name", "Full name is required");
            ""
        }
    };

    if !errors.is_empty() {
        let response =
            ResponseWrapper::<String>::error_response("Invalid data", 422, errors);
        return HttpResponse::UnprocessableEntity()
            .json(response);
    }

    let result = user_service.create_user(email, full_name).await;

    if let Err(e) = result {
        error!("Error creating user: {}", e);
        let response = ResponseWrapper::<String>::message_response("Error creating user", 500);
        return HttpResponse::InternalServerError()
            .json(response);
    }

    let mut rng = rand::thread_rng();
    let login_code = rng.gen_range(100_000, 999_999);

    let send_mail = email_service::send_otp_email(
        login_code.to_string().as_str(), email,
        full_name,
    );
    if let Err(e) = send_mail {
        error!("Error sending email: {}", e);
        let response = ResponseWrapper::<String>::message_response("Some error occurred.", 500);
        return HttpResponse::InternalServerError()
            .json(response);
    }

    let save_otp = user_service.save_otp(
        result.unwrap().inserted_id.as_object_id().unwrap().to_string().as_str(),
        login_code.to_string().as_str(),
    ).await;

    if let Err(e) = save_otp {
        error!("Error saving otp: {}", e);
        let response = ResponseWrapper::<String>::message_response("Some error occurred.", 500);
        return HttpResponse::InternalServerError()
            .json(response);
    }

    info!("Code: {}", login_code);


    let response = ResponseWrapper::<String>::message_response("User created", 200);
    HttpResponse::Ok()
        .json(response)
}

pub async fn login(data: Data<AppState>, body: Json<LoginRequest>) -> impl Responder {
    let user_service = &data.service_container.user_service;

    let email = match &body.email {
        Some(email) if EMAIL_REGEX.is_match(email) => {
            email
        }
        _ => {
            let response =
                ResponseWrapper::<String>::message_response("Invalid login", 422);
            return HttpResponse::UnprocessableEntity()
                .json(response);
        }
    };

    let user = user_service.find_user_by_email(email).await;
    let user = match user {
        Ok(Some(data)) => {
            data
        }
        Ok(None) => {
            info!("User not found");
            let response =
                ResponseWrapper::<String>::message_response("Invalid login", 404);
            return HttpResponse::NotFound()
                .json(response);
        }
        Err(e) => {
            error!("Error occurred: {:?}", e);
            let response =
                ResponseWrapper::<String>::message_response("Some error occurred", 500);
            return HttpResponse::InternalServerError()
                .json(response);
        }
    };


    let mut rng = rand::thread_rng();
    let login_code: i32 = rng.gen_range(100_000, 999_999);
    info!("Code: {:?}", login_code);

    let user_ref = &user;
    let full_name = user_ref.full_name.as_str();
    let send_mail = email_service::send_otp_email(
        login_code.to_string().as_str(), email,
        full_name,
    );
    if let Err(e) = send_mail {
        error!("Error sending email: {}", e);
        let response = ResponseWrapper::<String>::message_response("Some error occurred.", 500);
        return HttpResponse::InternalServerError()
            .json(response);
    }


    let save_otp = user_service.save_otp(
        user.id.as_ref().unwrap().as_str(),
        login_code.to_string().as_str(),
    ).await;

    if let Err(e) = save_otp {
        error!("Error saving otp: {}", e);
        let response = ResponseWrapper::<String>::message_response("Some error occurred.", 500);
        return HttpResponse::InternalServerError()
            .json(response);
    }

    let response = ResponseWrapper::<User>::data_response(user, "Check email for login code.", 200);
    HttpResponse::Ok()
        .json(response)
}


pub async fn verify_otp(data: Data<AppState>, body: Json<VerifyOtpRequest>) -> impl Responder {
    let user_service = &data.service_container.user_service;

    let user_id = match &body.user_id {
        Some(user_id) if !user_id.trim().is_empty() => {
            user_id
        }
        _ => {
            let response =
                ResponseWrapper::<String>::message_response("Invalid data", 400);
            return HttpResponse::BadRequest()
                .json(response);
        }
    };

    let otp = match &body.otp {
        Some(otp) if OTP_REGEX.is_match(otp) => {
            otp
        }
        _ => {
            debug!("Otp data: invalid pattern: {:?}", &body.otp);
            let response =
                ResponseWrapper::<String>::message_response("Invalid code.", 401);
            return HttpResponse::UnprocessableEntity()
                .json(response);
        }
    };

    let otp_data= match user_service.get_user_otp(user_id).await {
        Ok(Some(otp)) => {
            otp
        }
        _ =>  {
            debug!("Otp data: not found in db");
            let response =
                ResponseWrapper::<String>::message_response("Invalid code.", 401);
            return HttpResponse::Unauthorized()
                .json(response);
        }
    };


    if !otp_data.code.eq(otp) {
        let response = ResponseWrapper::<String>::message_response("Invalid code.", 400);
        return HttpResponse::BadRequest()
            .json(response);
    }

    if otp_data.expiry_time.lt( &Utc::now()) {
        let response = ResponseWrapper::<String>::message_response("Code has expired. Please retry.", 400);
        return HttpResponse::BadRequest()
            .json(response);
    }

    // TODO: generate jwt token
    let response = ResponseWrapper::<String>::data_response("".to_string(), "Login successful.", 200);
    return HttpResponse::Ok()
        .json(response);
}

