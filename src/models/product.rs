//! Module containg product model for query, insert, update
use std::time::SystemTime;

use validator::Validate;
use diesel::prelude::*;

use super::Language;
use super::Store;
use super::authorization::*;
use repos::types::DbConnection;
use models::store::stores::dsl as Stores;
use models::AttributeFilter;

/// diesel table for products
table! {
    products (id) {
        id -> Integer,
        store_id -> Integer,
        is_active -> Bool,
        name -> VarChar,
        short_description -> VarChar,
        long_description -> Nullable<VarChar>,
        price -> Double,
        currency_id -> Integer,
        discount -> Nullable<Float>,
        photo_main -> Nullable<VarChar>,
        vendor_code -> Nullable<VarChar>,
        cashback -> Nullable<Float>,
        default_language -> Varchar,
        created_at -> Timestamp, // UTC 0, generated at db level
        updated_at -> Timestamp, // UTC 0, generated at db level
    }
}

/// Payload for querying products
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[belongs_to(Store)]
pub struct Product {
    pub id: i32,
    pub store_id: i32,
    pub is_active: bool,
    pub name: String,
    pub short_description: String,
    pub long_description: Option<String>,
    pub price: f64,
    pub currency_id: i32,
    pub discount: Option<f32>,
    pub photo_main: Option<String>,
    pub vendor_code: Option<String>,
    pub cashback: Option<f32>,
    pub default_language: Language,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Payload for creating products
#[derive(Serialize, Deserialize, Insertable, Validate, Clone)]
#[table_name = "products"]
pub struct NewProduct {
    pub name: String,
    pub store_id: i32,
    pub currency_id: i32,
    pub short_description: String,
    pub long_description: Option<String>,
    pub price: f64,
    pub discount: Option<f32>,
    pub photo_main: Option<String>,
    pub vendor_code: Option<String>,
    pub cashback: Option<f32>,
    pub default_language: Language,
}

/// Payload for updating products
#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
#[table_name = "products"]
pub struct UpdateProduct {
    pub name: Option<String>,
    pub currency_id: Option<i32>,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub price: Option<f64>,
    pub discount: Option<f32>,
    pub photo_main: Option<String>,
    pub vendor_code: Option<String>,
    pub cashback: Option<f32>,
    pub default_language: Option<Language>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ElasticProduct {
    pub id: i32,
    pub name: String,
    pub short_description: String,
    pub long_description: Option<String>,
}

impl From<Product> for ElasticProduct {
    fn from(product: Product) -> Self {
        Self {
            id: product.id,
            name: product.name,
            short_description: product.short_description,
            long_description: product.long_description,
        }
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct SearchProduct {
    pub name: Option<String>,
    pub attr_filters: Option<Vec<AttributeFilter>>,
}

impl WithScope for Product {
    fn is_in_scope(&self, scope: &Scope, user_id: i32, conn: Option<&DbConnection>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(conn) = conn {
                    Stores::stores
                        .find(self.store_id)
                        .get_result::<Store>(&**conn)
                        .and_then(|store: Store| Ok(store.user_id == user_id))
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}

impl WithScope for NewProduct {
    fn is_in_scope(&self, scope: &Scope, user_id: i32, conn: Option<&DbConnection>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(conn) = conn {
                    Stores::stores
                        .find(self.store_id)
                        .get_result::<Store>(&**conn)
                        .and_then(|store: Store| Ok(store.user_id == user_id))
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}
