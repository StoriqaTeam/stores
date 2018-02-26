//! Module containg store model for query, insert, update
use std::time::SystemTime;

use validator::Validate;

use super::authorization::*;
use repos::types::DbConnection;

/// diesel table for stores
table! {
    stores (id) {
        id -> Integer,
        user_id -> Integer,
        is_active -> Bool,
        name -> VarChar,
        currency_id -> Integer,
        short_description -> VarChar,
        long_description -> Nullable<VarChar>,
        slug -> VarChar,
        cover -> Nullable<VarChar>,
        logo -> Nullable<VarChar>,
        phone -> Nullable<VarChar>,
        email -> Nullable<VarChar>,
        address -> Nullable<VarChar>,
        facebook_url -> Nullable<VarChar>,
        twitter_url -> Nullable<VarChar>,
        instagram_url -> Nullable<VarChar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        default_language -> Integer,
        slogan -> Nullable<VarChar>,
    }
}

/// Payload for querying stores
#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Identifiable)]
pub struct Store {
    pub id: i32,
    pub user_id: i32,
    pub is_active: bool,
    pub name: String,
    pub currency_id: i32,
    pub short_description: String,
    pub long_description: Option<String>,
    pub slug: String,
    pub cover: Option<String>,
    pub logo: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub default_language: i32,
    pub slogan: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, ElasticType)]
pub struct ElasticStore {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
}

impl From<Store> for ElasticStore {
    fn from(store: Store) -> Self {
        Self {
            id: store.id,
            user_id: store.user_id,
            name: store.name,
        }
    }
}

/// Payload for creating stores
#[derive(Serialize, Deserialize, Insertable, Validate, Clone)]
#[table_name = "stores"]
pub struct NewStore {
    pub name: String,
    pub user_id: i32,
    pub currency_id: i32,
    pub short_description: String,
    pub long_description: Option<String>,
    pub slug: String,
    pub cover: Option<String>,
    pub logo: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub default_language: i32,
    pub slogan: Option<String>,
}

/// Payload for updating users
#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
#[table_name = "stores"]
pub struct UpdateStore {
    pub name: Option<String>,
    pub currency_id: Option<i32>,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub slug: Option<String>,
    pub cover: Option<String>,
    pub logo: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub default_language: Option<i32>,
    pub slogan: Option<String>,
}

impl WithScope for Store {
    fn is_in_scope(&self, scope: &Scope, user_id: i32, _conn: Option<&DbConnection>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => self.user_id == user_id,
        }
    }
}

impl WithScope for NewStore {
    fn is_in_scope(&self, scope: &Scope, user_id: i32, _conn: Option<&DbConnection>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => self.user_id == user_id,
        }
    }
}
