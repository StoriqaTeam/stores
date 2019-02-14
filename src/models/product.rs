//! Module containg product model for query, insert, update
use std::collections::HashMap;
use std::time::SystemTime;

use serde_json;
use uuid::Uuid;
use validator::Validate;

use stq_static_resources::{Currency, ModerationStatus};
use stq_types::{BaseProductId, CategoryId, ExchangeRate, ProductId, ProductPrice, Quantity, StoreId};

use models::validation_rules::*;
use models::{AttrValue, Attribute, AttributeFilter, BaseProductRaw, ProdAttr, RangeFilter};
use schema::products;

/// Payload for querying products
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[belongs_to(BaseProductRaw, foreign_key = "base_product_id")]
#[table_name = "products"]
pub struct RawProduct {
    pub id: ProductId,
    pub is_active: bool,
    pub discount: Option<f64>,
    pub photo_main: Option<String>,
    pub cashback: Option<f64>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub base_product_id: BaseProductId,
    pub additional_photos: Option<serde_json::Value>,
    /// Seller price
    pub price: ProductPrice,
    pub vendor_code: String,
    /// Seller currency
    pub currency: Currency,
    pub kafka_update_no: i32,
    pub pre_order: bool,
    pub pre_order_days: i32,
    pub uuid: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomerPrice {
    pub price: ProductPrice,
    pub currency: Currency,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Product {
    #[serde(flatten)]
    pub product: RawProduct,
    pub customer_price: CustomerPrice,
}

impl Product {
    pub fn new(product: RawProduct, customer_price: CustomerPrice) -> Self {
        Self { product, customer_price }
    }
}

impl From<RawProduct> for Product {
    /// When no currency convert how seller price
    fn from(other: RawProduct) -> Self {
        let customer_price = CustomerPrice {
            price: other.price,
            currency: other.currency,
        };

        Self {
            product: other,
            customer_price,
        }
    }
}

/// Payload for creating products
#[derive(Serialize, Deserialize, Insertable, Validate, Clone, Debug)]
#[table_name = "products"]
pub struct NewProduct {
    pub base_product_id: Option<BaseProductId>,
    #[validate(range(min = "0.0", max = "1.0"))]
    pub discount: Option<f64>,
    pub photo_main: Option<String>,
    #[validate(custom = "validate_urls")]
    pub additional_photos: Option<serde_json::Value>,
    #[validate(custom = "validate_not_empty")]
    pub vendor_code: String,
    #[validate(range(min = "0.0", max = "1.0"))]
    pub cashback: Option<f64>,
    #[validate(custom = "validate_non_negative_price")]
    pub price: ProductPrice,
    pub currency: Currency,
    pub pre_order: Option<bool>,
    pub pre_order_days: Option<i32>,
    pub uuid: Uuid,
}

/// Payload for creating products
#[derive(Serialize, Deserialize, Validate, Clone, Debug)]
pub struct NewProductWithoutCurrency {
    pub base_product_id: Option<BaseProductId>,
    #[validate(range(min = "0.0", max = "1.0"))]
    pub discount: Option<f64>,
    pub photo_main: Option<String>,
    #[validate(custom = "validate_urls")]
    pub additional_photos: Option<serde_json::Value>,
    #[validate(custom = "validate_not_empty")]
    pub vendor_code: String,
    #[validate(range(min = "0.0", max = "1.0"))]
    pub cashback: Option<f64>,
    #[validate(custom = "validate_non_negative_price")]
    pub price: ProductPrice,
    pub pre_order: Option<bool>,
    pub pre_order_days: Option<i32>,
    pub uuid: Uuid,
}

impl From<(NewProductWithoutCurrency, Currency)> for NewProduct {
    fn from(other: (NewProductWithoutCurrency, Currency)) -> Self {
        Self {
            base_product_id: other.0.base_product_id,
            discount: other.0.discount,
            photo_main: other.0.photo_main,
            additional_photos: other.0.additional_photos,
            vendor_code: other.0.vendor_code,
            cashback: other.0.cashback,
            price: other.0.price,
            currency: other.1,
            pre_order: other.0.pre_order,
            pre_order_days: other.0.pre_order_days,
            uuid: other.0.uuid,
        }
    }
}

/// Payload for creating products and attributes
#[derive(Serialize, Deserialize, Clone, Debug, Validate)]
pub struct NewProductWithAttributes {
    #[validate]
    pub product: NewProductWithoutCurrency,
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
    #[validate(custom = "validate_not_empty")]
    pub vendor_code: Option<String>,
    #[validate(range(min = "0.0", max = "1.0"))]
    pub cashback: Option<f64>,
    #[validate(custom = "validate_non_negative_price")]
    pub price: Option<ProductPrice>,
    pub currency: Option<Currency>,
    pub pre_order: Option<bool>,
    pub pre_order_days: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateProductWithAttributes {
    pub product: Option<UpdateProduct>,
    pub attributes: Option<Vec<AttrValue>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ProductsSearchOptions {
    pub attr_filters: Option<Vec<AttributeFilter>>,
    pub currency_map: Option<HashMap<Currency, ExchangeRate>>,
    pub price_filter: Option<RangeFilter>,
    pub category_id: Option<CategoryId>,
    pub store_id: Option<StoreId>,
    pub categories_ids: Option<Vec<CategoryId>>,
    pub sort_by: Option<ProductsSorting>,
    pub status: Option<ModerationStatus>,
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
    pub status: Option<ModerationStatus>,
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

#[derive(Debug, Clone)]
pub struct ProductWithAttributes {
    pub product: RawProduct,
    pub attributes: Vec<(ProdAttr, Attribute)>,
}

impl ProductWithAttributes {
    pub fn new(product: RawProduct, attributes: Vec<(ProdAttr, Attribute)>) -> Self {
        Self { product, attributes }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetProducts {
    pub ids: Vec<ProductId>,
}
