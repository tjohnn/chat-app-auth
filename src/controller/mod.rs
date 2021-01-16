use serde::Serialize;
use std::collections::HashMap;

pub mod index;
pub mod account;
pub mod messages;

#[derive(Serialize, Debug)]
pub struct ResponseWrapper<'a, T : Serialize> {
    data: Option<T>,
    message: String,
    errors: Option<HashMap<&'a str, &'a str>>,
    status_code: u32
}

impl<'a, T : Serialize> ResponseWrapper<'a, T> {
    fn message_response(message: &str, response_code: u32) -> ResponseWrapper<T> {
        let data: Option<T> = None;
        ResponseWrapper {
            data,
            message: String::from(message),
            errors: None,
            status_code: response_code
        }
    }


    fn data_response(data: T, message: &str, response_code: u32) -> ResponseWrapper<T> {
        ResponseWrapper {
            data: Some(data),
            message: String::from(message),
            errors: None,
            status_code: response_code
        }
    }

    fn error_response(message: &str, response_code: u32, errors: HashMap<&'a str, &'a str>) -> ResponseWrapper<'a, T> {
        let data: Option<T> = None;
        ResponseWrapper {
            data,
            message: String::from(message),
            errors: Some(errors),
            status_code: response_code
        }
    }
}