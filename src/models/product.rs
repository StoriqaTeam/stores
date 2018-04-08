//! Module containg product model for query, insert, update
use std::time::SystemTime;

use validator::Validate;
use serde_json;

use models::{AttrValue, AttributeFilter, Category, RangeFilter};
use models::validation_rules::*;

/// diesel table for products
table! {
    products (id) {
        id -> Integer,
        base_product_id -> Integer,
        is_active -> Bool,
        discount -> Nullable<Double>,
        photo_main -> Nullable<VarChar>,
        additional_photos -> Nullable<Jsonb>,
        vendor_code -> Nullable<VarChar>,
        cashback -> Nullable<Double>,
        price -> Double,
        created_at -> Timestamp, // UTC 0, generated at db level
        updated_at -> Timestamp, // UTC 0, generated at db level
    }
}

/// Payload for querying products
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[belongs_to(Store)]
pub struct Product {
    pub id: i32,
    pub base_product_id: i32,
    pub is_active: bool,
    pub discount: Option<f64>,
    pub photo_main: Option<String>,
    pub additional_photos: Option<serde_json::Value>,
    pub vendor_code: Option<String>,
    pub cashback: Option<f64>,
    pub price: f64,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Payload for creating products
#[derive(Serialize, Deserialize, Insertable, Validate, Clone, Debug)]
#[table_name = "products"]
pub struct NewProduct {
    pub base_product_id: i32,
    #[validate(custom = "validate_non_negative")]
    pub discount: Option<f64>,
    pub photo_main: Option<String>,
    #[validate(custom = "validate_urls")]
    pub additional_photos: Option<serde_json::Value>,
    pub vendor_code: Option<String>,
    #[validate(custom = "validate_non_negative")]
    pub cashback: Option<f64>,
    #[validate(custom = "validate_non_negative")]
    pub price: f64,
}

/// Payload for creating products and attributes
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewProductWithAttributes {
    pub product: NewProduct,
    pub attributes: Vec<AttrValue>,
}

/// Payload for updating products
#[derive(Serialize, Deserialize, Insertable, Validate, AsChangeset, Clone, Debug)]
#[table_name = "products"]
pub struct UpdateProduct {
    #[validate(custom = "validate_non_negative")]
    pub discount: Option<f64>,
    pub photo_main: Option<String>,
    #[validate(custom = "validate_urls")]
    pub additional_photos: Option<serde_json::Value>,
    pub vendor_code: Option<String>,
    #[validate(custom = "validate_non_negative")]
    pub cashback: Option<f64>,
    #[validate(custom = "validate_non_negative")]
    pub price: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateProductWithAttributes {
    pub product: UpdateProduct,
    pub attributes: Vec<AttrValue>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SearchOptions {
    pub attr_filters: Vec<AttributeFilter>,
    pub price_range: Option<RangeFilter>,
    pub categories_ids: Vec<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SearchFiltersWithoutCategory {
    pub price_range: Option<RangeFilter>,
    pub categories: Category,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SearchFiltersInCategory {
    pub attr_filters: Vec<AttributeFilter>,
    pub price_range: Option<RangeFilter>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SearchProductsByName {
    pub name: String,
    pub options: Option<SearchOptions>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MostViewedProducts {
    pub options: Option<SearchOptions>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MostDiscountProducts {
    pub options: Option<SearchOptions>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SearchProductWithoutCategory {
    pub name: String,
    pub price_range: Option<RangeFilter>,
}

impl From<SearchProductWithoutCategory> for SearchProductsByName {
    fn from(prod: SearchProductWithoutCategory) -> Self {
        let options = SearchOptions {
            price_range: prod.price_range,
            ..Default::default()
        };

        Self {
            name: prod.name,
            options: Some(options),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SearchProductInCategory {
    pub name: String,
    pub category_id: i32,
    pub attr_filters: Vec<AttributeFilter>,
    pub price_range: Option<RangeFilter>,
}

impl From<SearchProductInCategory> for SearchProductsByName {
    fn from(prod: SearchProductInCategory) -> Self {
        let options = SearchOptions {
            price_range: prod.price_range,
            attr_filters: prod.attr_filters,
            categories_ids: vec![prod.category_id],
        };

        Self {
            name: prod.name,
            options: Some(options),
        }
    }
}
