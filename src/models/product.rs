//! Module containg product model for query, insert, update
use std::time::SystemTime;

use validator::Validate;
use serde_json;
use diesel::prelude::*;
use stq_acl::WithScope;

use models::base_product::base_products::dsl as BaseProducts;
use models::{AttrValue, AttributeFilter, BaseProduct, Scope, RangeFilter};
use models::validation_rules::*;
use repos::types::DbConnection;

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchOptions {
    pub attr_filters: Vec<AttributeFilter>,
    pub price_filter: Option<RangeFilter>,
    pub categories_ids: Vec<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

impl WithScope<Scope> for Product {
    fn is_in_scope(&self, scope: &Scope, user_id: i32, conn: Option<&DbConnection>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(conn) = conn {
                    BaseProducts::base_products
                        .find(self.base_product_id)
                        .get_result::<BaseProduct>(&**conn)
                        .and_then(|base_product: BaseProduct| Ok(base_product.is_in_scope(scope, user_id, Some(conn))))
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
                    BaseProducts::base_products
                        .find(self.base_product_id)
                        .get_result::<BaseProduct>(&**conn)
                        .and_then(|base_product: BaseProduct| Ok(base_product.is_in_scope(scope, user_id, Some(conn))))
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}
