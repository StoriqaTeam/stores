use chrono::{DateTime, Utc};
use uuid::Uuid;

use stq_static_resources::{Currency, ModerationStatus};
use stq_types::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogResponse {
    pub categories: Vec<CatalogResponseCategory>,
    pub stores: Vec<CatalogResponseStore>,
    pub base_products: Vec<CatalogResponseBaseProduct>,
    pub products: Vec<CatalogResponseProduct>,
    pub prod_attrs: Vec<CatalogResponseProdAttr>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogResponseCategory {
    pub id: CategoryId,
    pub name: serde_json::Value,
    pub parent_id: Option<CategoryId>,
    pub level: i32,
    pub meta_field: Option<serde_json::Value>,
    pub is_active: bool,
    pub uuid: Uuid,
    pub slug: CategorySlug,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogResponseStore {
    pub id: StoreId,
    pub user_id: UserId,
    pub is_active: bool,
    pub slug: String,
    pub cover: Option<String>,
    pub logo: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub slogan: Option<String>,
    pub default_language: String,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub rating: f64,
    pub country: Option<String>,
    pub status: ModerationStatus,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub country_code: Option<Alpha3>,
    pub uuid: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogResponseBaseProduct {
    pub id: BaseProductId,
    pub store_id: StoreId,
    pub is_active: bool,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub category_id: CategoryId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub views: i32,
    pub seo_title: Option<serde_json::Value>,
    pub seo_description: Option<serde_json::Value>,
    pub rating: f64,
    pub slug: BaseProductSlug,
    pub status: ModerationStatus,
    pub currency: Currency,
    pub uuid: Uuid,
    pub length_cm: Option<i32>,
    pub width_cm: Option<i32>,
    pub height_cm: Option<i32>,
    pub weight_g: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogResponseProduct {
    pub id: ProductId,
    pub is_active: bool,
    pub discount: Option<f64>,
    pub photo_main: Option<String>,
    pub cashback: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub base_product_id: BaseProductId,
    pub additional_photos: Option<serde_json::Value>,
    pub price: ProductPrice,
    pub currency: Currency,
    pub vendor_code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogResponseProdAttr {
    pub product_id: ProductId,
    pub name: serde_json::Value,
    pub value: AttributeValueCode,
}

impl CatalogResponse {
    pub fn find_base_product_by_id(&self, base_product_id: BaseProductId) -> Option<CatalogResponseBaseProduct> {
        self.base_products.clone().into_iter().find(|bp| bp.id == base_product_id)
    }

    pub fn find_store_by_id(&self, store_id: StoreId) -> Option<CatalogResponseStore> {
        self.stores.clone().into_iter().find(|s| s.id == store_id)
    }

    pub fn find_prod_attrs_by_product_id(&self, product_id: ProductId) -> Vec<CatalogResponseProdAttr> {
        self.prod_attrs
            .clone()
            .into_iter()
            .filter(|pa| pa.product_id == product_id)
            .collect()
    }
}
