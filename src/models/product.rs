//! Module containg product model for query, insert, update
use std::time::SystemTime;

use validator::Validate;
use diesel::prelude::*;
use serde_json;

use super::Store;
use repos::types::DbConnection;
use models::store::stores::dsl as Stores;
use models::{AttrValue, AttributeFilter};
use models::validation_rules::*;
use stq_acl::WithScope;
use models::Scope;

/// diesel table for products
table! {
    products (id) {
        id -> Integer,
        store_id -> Integer,
        is_active -> Bool,
        name -> Jsonb,
        short_description -> Jsonb,
        long_description -> Nullable<Jsonb>,
        price -> Double,
        currency_id -> Integer,
        discount -> Nullable<Float>,
        photo_main -> Nullable<VarChar>,
        vendor_code -> Nullable<VarChar>,
        cashback -> Nullable<Float>,
        category_id -> Integer,
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
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub price: f64,
    pub currency_id: i32,
    pub discount: Option<f32>,
    pub photo_main: Option<String>,
    pub vendor_code: Option<String>,
    pub cashback: Option<f32>,
    pub category_id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Payload for creating products
#[derive(Serialize, Deserialize, Insertable, Validate, Clone)]
#[table_name = "products"]
pub struct NewProduct {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub store_id: i32,
    pub currency_id: i32,
    #[validate(custom = "validate_translation")]
    pub short_description: serde_json::Value,
    #[validate(custom = "validate_translation")]
    pub long_description: Option<serde_json::Value>,
    #[validate(custom = "validate_non_negative")]
    pub price: f64,
    #[validate(custom = "validate_non_negative")]
    pub discount: Option<f32>,
    pub photo_main: Option<String>,
    pub vendor_code: Option<String>,
    #[validate(custom = "validate_non_negative")]
    pub cashback: Option<f32>,
    pub category_id: i32,
}

/// Payload for creating products and attributes
#[derive(Serialize, Deserialize, Clone)]
pub struct NewProductWithAttributes {
    pub product: NewProduct,
    pub attributes: Vec<AttrValue>,
}

/// Payload for updating products
#[derive(Serialize, Deserialize, Insertable, Validate, AsChangeset, Clone)]
#[table_name = "products"]
pub struct UpdateProduct {
    #[validate(custom = "validate_translation")]
    pub name: Option<serde_json::Value>,
    pub currency_id: Option<i32>,
    #[validate(custom = "validate_translation")]
    pub short_description: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub long_description: Option<serde_json::Value>,
    #[validate(custom = "validate_non_negative")]
    pub price: Option<f64>,
    #[validate(custom = "validate_non_negative")]
    pub discount: Option<f32>,
    pub photo_main: Option<String>,
    pub vendor_code: Option<String>,
    #[validate(custom = "validate_non_negative")]
    pub cashback: Option<f32>,
    pub category_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateProductWithAttributes {
    pub product: UpdateProduct,
    pub attributes: Vec<AttrValue>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ElasticProduct {
    pub id: i32,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub properties: Vec<AttrValue>,
    pub category_id: i32
}

impl ElasticProduct {
    pub fn new(product: Product, attrs: Vec<AttrValue>) -> Self {
        Self {
            id: product.id,
            name: product.name,
            short_description: product.short_description,
            long_description: product.long_description,
            properties: attrs,
            category_id: product.category_id
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchProduct {
    pub name: String,
    pub attr_filters: Vec<AttributeFilter>,
    pub category_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchProductElastic {
    pub name: String,
    pub attr_filters: Vec<(i32, AttributeFilter)>,
    pub category_id: Option<i32>,
}

impl SearchProductElastic {
    pub fn new(name: String, attr_filters: Vec<(i32, AttributeFilter)>, category_id: Option<i32>) -> Self {
        Self { name, attr_filters, category_id }
    }
}

impl WithScope<Scope> for Product {
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

impl WithScope<Scope> for NewProduct {
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
