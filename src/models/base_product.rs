//! Module containg base_product model for query, insert, update
use std::time::SystemTime;

use validator::Validate;

use serde_json;

use super::Store;
use models::validation_rules::*;
use models::Product;

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
        rating -> Double,
        slug -> VarChar,
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
    pub rating: f64,
    pub slug: String,
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
    pub slug: Option<String>,
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
    pub rating: Option<f64>,
    pub slug: Option<String>,
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
    pub rating: Option<f64>,
    pub variants: Vec<ElasticVariant>,
    pub category_id: i32,
    pub matched_variants_ids: Option<Vec<i32>>,
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
    pub rating: f64,
    pub slug: String,
    pub variants: Vec<Product>,
}

impl BaseProductWithVariants {
    pub fn new(base_product: BaseProduct, variants: Vec<Product>) -> Self {
        Self {
            id: base_product.id,
            is_active: base_product.is_active,
            store_id: base_product.store_id,
            name: base_product.name,
            short_description: base_product.short_description,
            long_description: base_product.long_description,
            seo_title: base_product.seo_title,
            seo_description: base_product.seo_description,
            currency_id: base_product.currency_id,
            category_id: base_product.category_id,
            views: base_product.views,
            created_at: base_product.created_at,
            updated_at: base_product.updated_at,
            rating: base_product.rating,
            slug: base_product.slug,
            variants,
        }
    }
}
