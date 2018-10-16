//! Module containing base_product model for query, insert, update
use std::time::SystemTime;

use serde_json;
use validator::Validate;

use stq_static_resources::{Currency, ModerationStatus};
use stq_types::{AttributeId, BaseProductId, ProductId, ProductPrice, StoreId};

use models::validation_rules::*;
use models::{NewProductWithAttributes, Product, ProductWithAttributes, Store};

use schema::base_products;

/// Payload for querying base_products
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[belongs_to(Store)]
#[table_name = "base_products"]
pub struct BaseProduct {
    pub id: BaseProductId,
    pub store_id: StoreId,
    pub is_active: bool,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub category_id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub views: i32,
    pub seo_title: Option<serde_json::Value>,
    pub seo_description: Option<serde_json::Value>,
    pub rating: f64,
    pub slug: String,
    pub status: ModerationStatus,
    pub kafka_update_no: i32,
    pub currency: Currency,
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

/// Payload for creating base product with variants
#[derive(Serialize, Deserialize, Validate, Clone, Debug)]
pub struct NewBaseProductWithVariants {
    #[serde(flatten)]
    pub new_base_product: NewBaseProduct,
    pub variants: Vec<NewProductWithAttributes>,
    pub selected_attributes: Vec<AttributeId>,
}

/// Payload for updating base_products
#[derive(Serialize, Deserialize, Insertable, Validate, AsChangeset, Clone, Debug, Default)]
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

impl UpdateBaseProduct {
    pub fn reset_moderation_status(self) -> Self {
        if self.name.is_some()
            | self.long_description.is_some()
            | self.short_description.is_some()
            | self.slug.is_some()
            | self.seo_title.is_some()
            | self.seo_description.is_some()
        {
            Self {
                status: Some(ModerationStatus::Draft),
                ..self
            }
        } else {
            self
        }
    }

    pub fn update_status(status: ModerationStatus) -> Self {
        Self {
            status: Some(status),
            ..Default::default()
        }
    }
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
    #[serde(flatten)]
    pub base_product: BaseProduct,
    pub variants: Vec<Product>,
}

impl BaseProductWithVariants {
    pub fn new(base_product: BaseProduct, variants: Vec<Product>) -> Self {
        Self { base_product, variants }
    }
}

#[derive(Debug, Clone)]
pub struct CatalogWithAttributes {
    pub base_product: BaseProduct,
    pub variants: Vec<ProductWithAttributes>,
}

impl CatalogWithAttributes {
    pub fn new(base_product: BaseProduct, variants: Vec<ProductWithAttributes>) -> Self {
        Self { base_product, variants }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ModeratorBaseProductSearchTerms {
    pub name: Option<String>,
    pub store_id: Option<i32>,
    pub state: Option<ModerationStatus>,
}
