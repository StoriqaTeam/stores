//! Module containg base_product model for query, insert, update
use std::time::SystemTime;

use serde_json;
use validator::Validate;

use stq_static_resources::{Currency, ModerationStatus};
use stq_types::{BaseProductId, ProductId, ProductPrice, StoreId};

use models::validation_rules::*;
use models::Product;
use models::Store;
use schema::base_products;

/// Payload for querying base_products
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[belongs_to(Store)]
#[table_name = "base_products"]
pub struct BaseProduct {
    pub id: BaseProductId,
    pub is_active: bool,
    pub store_id: StoreId,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub seo_title: Option<serde_json::Value>,
    pub seo_description: Option<serde_json::Value>,
    pub currency: Currency,
    pub category_id: i32,
    pub views: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub rating: f64,
    pub slug: String,
    pub status: ModerationStatus,
}

/// Payload for creating base_products
#[derive(Serialize, Deserialize, Insertable, Validate, Clone, Debug)]
#[table_name = "base_products"]
pub struct NewBaseProduct {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub store_id: StoreId,
    #[validate(custom = "validate_translation")]
    pub short_description: serde_json::Value,
    #[validate(custom = "validate_translation")]
    pub long_description: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub seo_title: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub seo_description: Option<serde_json::Value>,
    pub currency: Currency,
    pub category_id: i32,
    #[validate(custom = "validate_slug")]
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
    pub currency: Option<Currency>,
    pub category_id: Option<i32>,
    pub rating: Option<f64>,
    #[validate(custom = "validate_slug")]
    pub slug: Option<String>,
    pub status: Option<ModerationStatus>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ElasticProduct {
    pub id: BaseProductId,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub views: i32,
    pub rating: Option<f64>,
    pub variants: Vec<ElasticVariant>,
    pub category_id: i32,
    pub matched_variants_ids: Option<Vec<ProductId>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ElasticVariant {
    pub prod_id: ProductId,
    pub discount: Option<f64>,
    pub price: ProductPrice,
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
    pub id: BaseProductId,
    pub is_active: bool,
    pub store_id: StoreId,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub seo_title: Option<serde_json::Value>,
    pub seo_description: Option<serde_json::Value>,
    pub currency: Currency,
    pub category_id: i32,
    pub views: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub rating: f64,
    pub slug: String,
    pub status: ModerationStatus,
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
            currency: base_product.currency,
            category_id: base_product.category_id,
            views: base_product.views,
            created_at: base_product.created_at,
            updated_at: base_product.updated_at,
            rating: base_product.rating,
            slug: base_product.slug,
            status: base_product.status,
            variants,
        }
    }
}
