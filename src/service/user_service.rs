use mongodb::{Collection, results::InsertOneResult};
use serde::Serialize;
use bson::doc;
use bson::oid::ObjectId;
use chrono::{prelude::*, Duration};
use std::error::Error;
use log::debug;

#[derive(Serialize, Debug, Default)]
pub struct User {
    pub id: Option<String>,
    pub email: String,
    pub full_name: String,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>
}

#[derive(Serialize, Debug)]
pub struct Otp {
    pub id: String,
    pub user_id: String,
    pub code: String,
    pub expiry_time: DateTime<Utc>
}

#[derive(Clone)]
pub struct UserService {
    user_collection: Collection,
    otp_collection: Collection
}

impl UserService {
    pub fn new(user_collection: Collection, otp_collection: Collection) -> UserService {
        UserService {
            user_collection,
            otp_collection
        }
    }

    pub async fn create_user(&self, email: &str, full_name: &str) -> Result<InsertOneResult, Box<dyn Error>> {
        let date = Utc::now();
        let result = self.user_collection.insert_one(doc! {
            "email": email,
            "full_name": full_name,
            "created_at": date,
            "updated_at": date
        }, None).await?;
        Ok(result)
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn Error>> {
        let result = self.user_collection.find_one(doc! {
            "email": email
        }, None).await?;
        
        if let Some(doc) = result {
            let user = User {
                id: Some(doc.get_object_id("_id").unwrap().to_string()),
                email: doc.get_str("email").unwrap().to_string(),
                full_name: doc.get_str("full_name").unwrap().to_string(),
                created_at: Some(doc.get_datetime("created_at")?.clone()),
                updated_at: Some(doc.get_datetime("updated_at")?.clone())
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    pub async fn get_user_otp(&self, user_id: &str) -> Result<Option<Otp>, Box<dyn Error>> {
        debug!("User ID: {}", user_id);
        let user_id = ObjectId::with_string(user_id)?;

        let result = self.otp_collection.find_one(doc! {
            "user_id": user_id
        }, None).await?;

        if let Some(doc) = result {
            let otp = Otp {
                id: doc.get_object_id("_id").unwrap().to_string(),
                user_id: doc.get_object_id("user_id").unwrap().to_string(),
                code: doc.get_str("code").unwrap().to_string(),
                expiry_time: doc.get_datetime("expiry_time")?.clone()
            };
            Ok(Some(otp))
        } else {
            debug!("Otp data is empty");
            Ok(None)
        }
    }

    pub async fn save_otp(&self, user_id: &str, otp: &str) -> Result<(), Box<dyn Error>> {
        let result = self.otp_collection.find_one(doc! {
            "user_id": ObjectId::with_string(user_id).unwrap()
        }, None).await?;
        let expiry_time = Utc::now() + Duration::minutes(5);
        match result{
            Some(_) => {
                self.otp_collection.update_one(
                    doc! {
                    "user_id": ObjectId::with_string(user_id).unwrap()
                }, doc! {
                    "$set": doc! {
                        "code": otp,
                        "expiry_time": expiry_time
                    }
                },
                None).await?;
            },
            _ => {
                self.otp_collection.insert_one(doc! {
                    "user_id": ObjectId::with_string(user_id).unwrap(),
                    "code": otp,
                    "expiry_time": expiry_time
                }, None).await?;
            }
        }
        Ok(())
    }
}