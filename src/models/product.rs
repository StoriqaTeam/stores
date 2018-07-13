//! Module containg product model for query, insert, update
use std::collections::HashMap;
use std::time::SystemTime;

use serde_json;
use validator::Validate;

use stq_types::{BaseProductId, CurrencyId, ProductId, ProductPrice, Quantity, StoreId};

use models::validation_rules::*;
use models::{AttrValue, AttributeFilter, BaseProduct, RangeFilter};

/// diesel table for products
table! {
    products (id) {
        id -> Integer,
        base_product_id -> Integer,
        is_active -> Bool,
        discount -> Nullable<Double>,
        photo_main -> Nullable<VarChar>,
        additional_photos -> Nullable<Jsonb>,
        vendor_code -> VarChar,
        cashback -> Nullable<Double>,
        price -> Double,
        currency_id -> Nullable<Integer>,
        created_at -> Timestamp, // UTC 0, generated at db level
        updated_at -> Timestamp, // UTC 0, generated at db level
    }
}

/// Payload for querying products
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[belongs_to(BaseProduct)]
#[table_name = "products"]
pub struct Product {
    pub id: ProductId,
    pub base_product_id: BaseProductId,
    pub is_active: bool,
    pub discount: Option<f64>,
    pub photo_main: Option<String>,
    pub additional_photos: Option<serde_json::Value>,
    pub vendor_code: String,
    pub cashback: Option<f64>,
    pub price: ProductPrice,
    pub currency_id: Option<CurrencyId>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Payload for creating products
#[derive(Serialize, Deserialize, Insertable, Validate, Clone, Debug)]
#[table_name = "products"]
pub struct NewProduct {
    pub base_product_id: BaseProductId,
    #[validate(range(min = "0.0", max = "1.0"))]
    pub discount: Option<f64>,
    pub photo_main: Option<String>,
    #[validate(custom = "validate_urls")]
    pub additional_photos: Option<serde_json::Value>,
    #[validate(length(min = "1"))]
    pub vendor_code: String,
    #[validate(range(min = "0.0", max = "1.0"))]
    pub cashback: Option<f64>,
    #[validate(custom = "validate_non_negative")]
    pub price: ProductPrice,
    pub currency_id: Option<CurrencyId>,
}

/// Payload for creating products and attributes
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewProductWithAttributes {
    pub product: NewProduct,
    pub attributes: Vec<AttrValue>,
}

/// Payload for updating products
#[derive(Serialize, Deserialize, Insertable, Validate, AsChangeset, Clone, Debug, Default)]
#[table_name = "products"]
pub struct UpdateProduct {
    #[validate(range(min = "0.0", max = "1.0"))]
    pub discount: Option<f64>,
    pub photo_main: Option<String>,
    #[validate(custom = "validate_urls")]
    pub additional_photos: Option<serde_json::Value>,
    pub vendor_code: Option<String>,
    #[validate(range(min = "0.0", max = "1.0"))]
    pub cashback: Option<f64>,
    #[validate(custom = "validate_non_negative")]
    pub price: Option<ProductPrice>,
    pub currency_id: Option<CurrencyId>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateProductWithAttributes {
    pub product: Option<UpdateProduct>,
    pub attributes: Option<Vec<AttrValue>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ProductsSearchOptions {
    pub attr_filters: Option<Vec<AttributeFilter>>,
    pub currency_id: Option<CurrencyId>,
    pub currency_map: Option<HashMap<CurrencyId, f64>>,
    pub price_filter: Option<RangeFilter>,
    pub category_id: Option<i32>,
    pub store_id: Option<StoreId>,
    pub categories_ids: Option<Vec<i32>>,
    pub sort_by: Option<ProductsSorting>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SearchProductsByName {
    pub name: String,
    pub options: Option<ProductsSearchOptions>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AutoCompleteProductName {
    pub name: String,
    pub store_id: Option<StoreId>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MostViewedProducts {
    pub options: Option<ProductsSearchOptions>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ProductsSorting {
    Views,
    PriceAsc,
    PriceDesc,
    Discount,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MostDiscountProducts {
    pub options: Option<ProductsSearchOptions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CartProduct {
    pub product_id: ProductId,
    pub quantity: Quantity,
}
