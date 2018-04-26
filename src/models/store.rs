//! Module containg store model for query, insert, update
use std::time::SystemTime;

use serde_json;
use validator::Validate;

use models::validation_rules::*;
use models::BaseProductWithVariants;

/// diesel table for stores
table! {
    stores (id) {
        id -> Integer,
        user_id -> Integer,
        is_active -> Bool,
        name -> Jsonb,
        short_description -> Jsonb,
        long_description -> Nullable<Jsonb>,
        slug -> VarChar,
        cover -> Nullable<VarChar>,
        logo -> Nullable<VarChar>,
        phone -> Nullable<VarChar>,
        email -> Nullable<VarChar>,
        address -> Nullable<VarChar>,
        facebook_url -> Nullable<VarChar>,
        twitter_url -> Nullable<VarChar>,
        instagram_url -> Nullable<VarChar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        default_language -> VarChar,
        slogan -> Nullable<VarChar>,
        rating -> Double,
        country -> Nullable<VarChar>,
        product_categories -> Nullable<Jsonb>,
    }
}

/// Payload for querying stores
#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Identifiable)]
pub struct Store {
    pub id: i32,
    pub user_id: i32,
    pub is_active: bool,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub slug: String,
    pub cover: Option<String>,
    pub logo: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub default_language: String,
    pub slogan: Option<String>,
    pub rating: f64,
    pub country: Option<String>,
    pub product_categories: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ElasticStore {
    pub id: i32,
    pub user_id: i32,
    pub name: serde_json::Value,
}

impl From<Store> for ElasticStore {
    fn from(store: Store) -> Self {
        Self {
            id: store.id,
            user_id: store.user_id,
            name: store.name,
        }
    }
}

/// Payload for creating stores
#[derive(Serialize, Deserialize, Insertable, Validate, Clone, Debug)]
#[table_name = "stores"]
pub struct NewStore {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub user_id: i32,
    #[validate(custom = "validate_translation")]
    pub short_description: serde_json::Value,
    #[validate(custom = "validate_translation")]
    pub long_description: Option<serde_json::Value>,
    #[validate(custom = "validate_slug")]
    pub slug: String,
    pub cover: Option<String>,
    pub logo: Option<String>,
    #[validate(custom = "validate_phone")]
    pub phone: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    pub address: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    #[validate(custom = "validate_lang")]
    pub default_language: String,
    pub slogan: Option<String>,
    pub country: Option<String>,
}

/// Payload for updating users
#[derive(Default, Serialize, Deserialize, Insertable, Validate, AsChangeset, Debug)]
#[table_name = "stores"]
pub struct UpdateStore {
    #[validate(custom = "validate_translation")]
    pub name: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub short_description: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub long_description: Option<serde_json::Value>,
    #[validate(custom = "validate_slug")]
    pub slug: Option<String>,
    pub cover: Option<String>,
    pub logo: Option<String>,
    #[validate(custom = "validate_phone")]
    pub phone: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    pub address: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    #[validate(custom = "validate_lang")]
    pub default_language: Option<String>,
    pub slogan: Option<String>,
    pub rating: Option<f64>,
    pub country: Option<String>,
    pub product_categories: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchStore {
    pub name: String,
    pub options: Option<StoresSearchOptions>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoresSearchOptions {
    pub category_id: Option<i32>,
    pub country: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProductCategories {
    pub category_id: i32,
    pub count: i32,
}

impl ProductCategories {
    pub fn new(category_id: i32) -> Self {
        Self { category_id, count: 1 }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoreWithBaseProducts {
    pub id: i32,
    pub name: serde_json::Value,
    pub is_active: bool,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub slug: String,
    pub cover: Option<String>,
    pub logo: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub default_language: String,
    pub slogan: Option<String>,
    pub rating: f64,
    pub country: Option<String>,
    pub base_products: Vec<BaseProductWithVariants>,
}

impl StoreWithBaseProducts {
    pub fn new(store: Store, base_products: Vec<BaseProductWithVariants>) -> Self {
        Self {
            id: store.id,
            name: store.name,
            is_active: store.is_active,
            short_description: store.short_description,
            long_description: store.long_description,
            slug: store.slug,
            cover: store.cover,
            logo: store.logo,
            phone: store.phone,
            email: store.email,
            address: store.address,
            facebook_url: store.facebook_url,
            twitter_url: store.twitter_url,
            instagram_url: store.instagram_url,
            default_language: store.default_language,
            slogan: store.slogan,
            rating: store.rating,
            country: store.country,
            base_products,
        }
    }
}