//! Module containg store model for query, insert, update
use std::time::SystemTime;

use validator::Validate;


/// diesel table for stores
table! {
    stores (id) {
        id -> Integer,
        is_active -> Bool,
        name -> VarChar,
        currency_id -> Integer,
        short_description -> VarChar,
        long_description -> Nullable<VarChar>,
        slug -> VarChar,
        cover -> Nullable<VarChar>,
        logo -> Nullable<VarChar>,
        phone -> VarChar,
        email -> VarChar,
        address -> VarChar,
        facebook_url -> Nullable<VarChar>,
        twitter_url -> Nullable<VarChar>,
        instagram_url -> Nullable<VarChar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

/// Payload for querying stores
#[derive(Debug, Serialize, Deserialize, Queryable, Clone)]
pub struct Store {
    pub id: i32,
    pub is_active: bool,
    pub name: String,
    pub currency_id: i32,
    pub short_description: String,
    pub long_description: Option<String>,
    pub slug: String,
    pub cover: Option<String>,
    pub logo: Option<String>,
    pub phone: String,
    pub email: String,
    pub address: String,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Payload for creating stores
#[derive(Serialize, Deserialize, Insertable, Validate, Clone)]
#[table_name = "stores"]
pub struct NewStore {
    pub name: String,
    pub currency_id: i32,
    pub short_description: String,
    pub long_description: Option<String>,
    pub slug: String,
    pub cover: Option<String>,
    pub logo: Option<String>,
    pub phone: String,
    pub email: String,
    pub address: String,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
}

/// Payload for updating users
#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
#[table_name = "stores"]
pub struct UpdateStore {
    pub name: String,
    pub currency_id: Option<i32>,
    pub short_description: Option<String>,
    pub long_description: Option<Option<String>>,
    pub slug: Option<String>,
    pub cover: Option<Option<String>>,
    pub logo: Option<Option<String>>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub facebook_url: Option<Option<String>>,
    pub twitter_url: Option<Option<String>>,
    pub instagram_url: Option<Option<String>>,
}
