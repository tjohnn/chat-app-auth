use mongodb::Database;
use crate::service::user_service::UserService;

pub mod controller;
pub mod routes;
pub mod service;


#[macro_use]
pub extern crate lazy_static;

const USER_COLLECTION_NAME: &str = "users";
const OTP_COLLECTION_NAME: &str = "otps";

// This struct represents state
#[derive(Clone)]
pub struct AppState {
    pub service_container: ServiceContainer,
}

impl AppState {

}

#[derive(Clone)]
pub struct ServiceContainer {
    pub user_service: UserService,
}

impl ServiceContainer {
    pub fn new(database: Database) -> ServiceContainer {
        let user_collection = database.collection(USER_COLLECTION_NAME);
        let otp_collection = database.collection(OTP_COLLECTION_NAME);
        ServiceContainer {
            user_service: UserService::new(user_collection, otp_collection)
        }
    }
}