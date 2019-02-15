use chrono::{DateTime, Utc};
use uuid::Uuid;

use stq_static_resources::{Currency, ModerationStatus};
use stq_types::*;

use models::attributes::attribute::Attribute;
use models::attributes::attribute_product::ProdAttr;
use models::base_product::BaseProduct;
use models::category::RawCategory;
use models::product::RawProduct;
use models::store::Store;

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

impl From<RawCategory> for CatalogResponseCategory {
    fn from(category: RawCategory) -> Self {
        Self {
            id: category.id,
            name: category.name,
            parent_id: category.parent_id,
            level: category.level,
            meta_field: category.meta_field,
            is_active: category.is_active,
            uuid: category.uuid,
            slug: category.slug,
        }
    }
}

impl From<Store> for CatalogResponseStore {
    fn from(store: Store) -> Self {
        Self {
            id: store.id,
            user_id: store.user_id,
            is_active: store.is_active,
            slug: store.slug,
            cover: store.cover,
            logo: store.logo,
            phone: store.phone,
            email: store.email,
            address: store.address,
            facebook_url: store.facebook_url,
            twitter_url: store.twitter_url,
            instagram_url: store.instagram_url,
            created_at: store.created_at.into(),
            updated_at: store.updated_at.into(),
            slogan: store.slogan,
            default_language: store.default_language,
            name: store.name,
            short_description: store.short_description,
            long_description: store.long_description,
            rating: store.rating,
            country: store.country,
            status: store.status,
            administrative_area_level_1: store.administrative_area_level_1,
            administrative_area_level_2: store.administrative_area_level_2,
            locality: store.locality,
            political: store.political,
            postal_code: store.postal_code,
            route: store.route,
            street_number: store.street_number,
            country_code: store.country_code,
            uuid: store.uuid,
        }
    }
}

impl From<BaseProduct> for CatalogResponseBaseProduct {
    fn from(base_product: BaseProduct) -> Self {
        Self {
            id: base_product.id,
            store_id: base_product.store_id,
            is_active: base_product.is_active,
            name: base_product.name,
            short_description: base_product.short_description,
            long_description: base_product.long_description,
            category_id: base_product.category_id,
            created_at: base_product.created_at.into(),
            updated_at: base_product.updated_at.into(),
            views: base_product.views,
            seo_title: base_product.seo_title,
            seo_description: base_product.seo_description,
            rating: base_product.rating,
            slug: base_product.slug,
            status: base_product.status,
            currency: base_product.currency,
            uuid: base_product.uuid,
            length_cm: base_product.length_cm,
            width_cm: base_product.width_cm,
            height_cm: base_product.height_cm,
            weight_g: base_product.weight_g,
        }
    }
}

impl From<RawProduct> for CatalogResponseProduct {
    fn from(product: RawProduct) -> Self {
        Self {
            id: product.id,
            is_active: product.is_active,
            discount: product.discount,
            photo_main: product.photo_main,
            cashback: product.cashback,
            created_at: product.created_at.into(),
            updated_at: product.updated_at.into(),
            base_product_id: product.base_product_id,
            additional_photos: product.additional_photos,
            price: product.price,
            currency: product.currency,
            vendor_code: product.vendor_code,
        }
    }
}

impl From<(ProdAttr, Attribute)> for CatalogResponseProdAttr {
    fn from(tuple: (ProdAttr, Attribute)) -> Self {
        let (prod_attr, attr) = tuple;

        Self {
            product_id: prod_attr.prod_id,
            name: attr.name,
            value: prod_attr.value,
        }
    }
}
