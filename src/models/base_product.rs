//! Module containg base_product model for query, insert, update
use std::time::SystemTime;

use validator::Validate;

use serde_json;

use super::Store;
use models::{AttrValue, Product};
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
        seo_title -> Nullable<Jsonb>,
        seo_description -> Nullable<Jsonb>,
        currency_id -> Integer,
        category_id -> Integer,
        views -> Integer,
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
    pub seo_title: Option<serde_json::Value>,
    pub seo_description: Option<serde_json::Value>,
    pub currency_id: i32,
    pub category_id: i32,
    pub views: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Payload for creating base_products
#[derive(Serialize, Deserialize, Insertable, Validate, Clone, Debug)]
#[table_name = "base_products"]
pub struct NewBaseProduct {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub store_id: i32,
    #[validate(custom = "validate_translation")]
    pub short_description: serde_json::Value,
    #[validate(custom = "validate_translation")]
    pub long_description: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub seo_title: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub seo_description: Option<serde_json::Value>,
    pub currency_id: i32,
    pub category_id: i32,
}

/// Payload for updating base_products
#[derive(Serialize, Deserialize, Insertable, Validate, AsChangeset, Clone, Debug)]
#[table_name = "base_products"]
pub struct UpdateBaseProduct {
    #[validate(custom = "validate_translation")]
    pub name: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub short_description: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub long_description: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub seo_title: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub seo_description: Option<serde_json::Value>,
    pub currency_id: Option<i32>,
    pub category_id: Option<i32>,
}

/// Payload for updating views on base product
#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[table_name = "base_products"]
pub struct UpdateBaseProductViews {
    pub views: i32,
}

impl From<BaseProduct> for UpdateBaseProductViews {
    fn from(base_product: BaseProduct) -> Self {
        Self {
            views: base_product.views + 1,
        }
    }
}

impl<'a> From<&'a BaseProduct> for UpdateBaseProductViews {
    fn from(base_product: &'a BaseProduct) -> Self {
        Self {
            views: base_product.views + 1,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ElasticProduct {
    pub id: i32,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub views: i32,
    pub variants: Vec<ElasticVariant>,
    pub category_id: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ElasticVariant {
    pub prod_id: i32,
    pub discount: Option<f64>,
    pub price: f64,
    pub attrs: Vec<ElasticAttrValue>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ElasticAttrValue {
    pub attr_id: i32,
    pub str_val: Option<String>,
    pub float_val: Option<f64>,
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
