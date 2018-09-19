//! Module containg store model for query, insert, update
use std::time::SystemTime;

use serde_json;
use validator::Validate;

use stq_static_resources::ModerationStatus;
use stq_types::{StoreId, UserId};

use models::validation_rules::*;
use models::BaseProductWithVariants;
use schema::stores;

/// Payload for querying stores
#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Identifiable)]
pub struct Store {
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
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub slogan: Option<String>,
    pub default_language: String,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub rating: f64,
    pub country: Option<String>,
    pub product_categories: Option<serde_json::Value>,
    pub status: ModerationStatus,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub place_id: Option<String>,
    pub kafka_update_no: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ElasticStore {
    pub id: StoreId,
    pub user_id: UserId,
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
    pub user_id: UserId,
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
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub place_id: Option<String>,
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
    pub status: Option<ModerationStatus>,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub place_id: Option<String>,
}

impl UpdateStore {
    pub fn update_product_categories(product_categories: Option<serde_json::Value>, old_cat_id: i32, new_cat_id: i32) -> Self {
        let prod_cats = if let Some(prod_cats) = product_categories {
            let mut product_categories = serde_json::from_value::<Vec<ProductCategories>>(prod_cats).unwrap_or_default();
            let mut new_prod_cats = vec![];
            let mut new_cat_exists = false;
            for pc in &mut product_categories {
                if pc.category_id == new_cat_id {
                    pc.count += 1;
                    new_cat_exists = true;
                }
                if pc.category_id == old_cat_id {
                    pc.count -= 1;
                }
                new_prod_cats.push(pc.clone());
            }
            if !new_cat_exists {
                new_prod_cats.push(ProductCategories::new(new_cat_id));
            }
            new_prod_cats
        } else {
            let pc = ProductCategories::new(new_cat_id);
            vec![pc]
        };

        let product_categories = serde_json::to_value(prod_cats).ok();

        Self {
            product_categories,
            ..UpdateStore::default()
        }
    }
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
    pub id: StoreId,
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
    pub status: ModerationStatus,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub place_id: Option<String>,
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
            status: store.status,
            administrative_area_level_1: store.administrative_area_level_1,
            administrative_area_level_2: store.administrative_area_level_2,
            locality: store.locality,
            political: store.political,
            postal_code: store.postal_code,
            route: store.route,
            street_number: store.street_number,
            place_id: store.place_id,
            base_products,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModeratorStoreSearchTerms {
    pub name: Option<String>,
    pub store_manager_id: Option<UserId>,
    pub state: Option<ModerationStatus>,
}
