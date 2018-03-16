//! Module containg base_product model for query, insert, update
use std::time::SystemTime;

use validator::Validate;
use diesel::prelude::*;
use serde_json;
use stq_acl::WithScope;

use super::Store;
use repos::types::DbConnection;
use models::{AttrValue, Product, Scope};
use models::store::stores::dsl as Stores;
use models::validation_rules::*;

/// diesel table for base_products
table! {
    base_products (id) {
        id -> Integer,
        is_active -> Bool,
        store_id -> Integer,
        name -> Jsonb,
        short_description -> Jsonb,
        long_description -> Nullable<Jsonb>,
        currency_id -> Integer,
        category_id -> Integer,
        created_at -> Timestamp, // UTC 0, generated at db level
        updated_at -> Timestamp, // UTC 0, generated at db level
    }
}

/// Payload for querying base_products
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[belongs_to(Store)]
pub struct BaseProduct {
    pub id: i32,
    pub is_active: bool,
    pub store_id: i32,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub currency_id: i32,
    pub category_id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Payload for creating base_products
#[derive(Serialize, Deserialize, Insertable, Validate, Clone)]
#[table_name = "base_products"]
pub struct NewBaseProduct {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub store_id: i32,
    #[validate(custom = "validate_translation")]
    pub short_description: serde_json::Value,
    #[validate(custom = "validate_translation")]
    pub long_description: Option<serde_json::Value>,
    pub currency_id: i32,
    pub category_id: i32,
}

/// Payload for updating base_products
#[derive(Serialize, Deserialize, Insertable, Validate, AsChangeset, Clone)]
#[table_name = "base_products"]
pub struct UpdateBaseProduct {
    #[validate(custom = "validate_translation")]
    pub name: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub short_description: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub long_description: Option<serde_json::Value>,
    pub currency_id: Option<i32>,
    pub category_id: Option<i32>,
}

impl WithScope<Scope> for BaseProduct {
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

impl WithScope<Scope> for NewBaseProduct {
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ElasticProduct {
    pub id: i32,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub variants: Vec<ElasticVariant>,
    pub category_id: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ElasticVariant {
    pub id: i32,
    pub attrs: Vec<AttrValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseProductWithVariants {
    pub base_product: BaseProduct,
    pub variants: Vec<VariantsWithAttributes>,
}

impl BaseProductWithVariants {
    pub fn new(base_product: BaseProduct, variants: Vec<VariantsWithAttributes>) -> Self {
        Self {
            base_product,
            variants,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariantsWithAttributes {
    pub product: Product,
    pub attrs: Vec<AttrValue>,
}

impl VariantsWithAttributes {
    pub fn new(product: Product, attrs: Vec<AttrValue>) -> Self {
        Self { product, attrs }
    }
}
