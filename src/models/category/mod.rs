//! Models contains all structures that are used in different
//! modules of the app
//! EAV model categories
pub mod category_attribute;

use std::cmp::Ordering;

use serde_json;
use uuid::Uuid;
use validator::Validate;

use stq_types::{BaseProductId, CategoryId, CategorySlug};

pub use self::category_attribute::*;
use models::validation_rules::*;
use models::Attribute;
use schema::categories;

/// RawCategory is an object stored in PG, used only for Category tree creation,
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "categories"]
pub struct RawCategory {
    pub id: CategoryId,
    pub name: serde_json::Value,
    pub parent_id: Option<CategoryId>,
    pub level: i32,
    pub meta_field: Option<serde_json::Value>,
    pub is_active: bool,
    pub uuid: Uuid,
    pub slug: CategorySlug,
}

impl Eq for RawCategory {}

impl PartialEq for RawCategory {
    fn eq(&self, other: &RawCategory) -> bool {
        self.id == other.id
    }
}

impl Ord for RawCategory {
    fn cmp(&self, other: &RawCategory) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for RawCategory {
    fn partial_cmp(&self, other: &RawCategory) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Used to insert a category into the table
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "categories"]
pub struct InsertCategory {
    pub name: serde_json::Value,
    pub parent_id: CategoryId,
    pub level: i32,
    pub meta_field: Option<serde_json::Value>,
    pub is_active: bool,
    pub uuid: Uuid,
    pub slug: Option<CategorySlug>,
}

/// Payload for creating categories
#[derive(Serialize, Deserialize, Clone, Validate, Debug)]
pub struct NewCategory {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub parent_id: CategoryId,
    pub meta_field: Option<serde_json::Value>,
    pub uuid: Uuid,
    #[validate(custom = "validate_slug")]
    pub slug: Option<CategorySlug>,
}

/// Payload for updating categories
#[derive(Serialize, Deserialize, Insertable, AsChangeset, Validate, Debug)]
#[table_name = "categories"]
pub struct UpdateCategory {
    #[validate(custom = "validate_translation")]
    pub name: Option<serde_json::Value>,
    pub meta_field: Option<serde_json::Value>,
    pub parent_id: Option<CategoryId>,
    #[validate(range(min = "1", max = "3"))]
    pub level: Option<i32>,
    #[validate(custom = "validate_slug")]
    pub slug: Option<CategorySlug>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: CategoryId,
    pub is_active: bool,
    pub name: serde_json::Value,
    pub meta_field: Option<serde_json::Value>,
    pub level: i32,
    pub parent_id: Option<CategoryId>,
    pub children: Vec<Category>,
    pub attributes: Vec<Attribute>,
    pub slug: CategorySlug,
}

impl Category {
    pub const MAX_LEVEL_NESTING: i32 = 3;
}

impl Default for Category {
    fn default() -> Self {
        Self {
            id: CategoryId(0),
            is_active: true,
            name: serde_json::from_str("[{\"lang\" : \"en\", \"text\" : \"root\"}]").unwrap(),
            meta_field: None,
            children: vec![],
            level: 0,
            parent_id: None,
            attributes: vec![],
            slug: CategorySlug(String::default()),
        }
    }
}

impl<'a> From<&'a RawCategory> for Category {
    fn from(cat: &'a RawCategory) -> Self {
        Self {
            id: cat.id,
            is_active: cat.is_active,
            name: cat.name.clone(),
            meta_field: cat.meta_field.clone(),
            children: vec![],
            parent_id: cat.parent_id,
            level: cat.level,
            attributes: vec![],
            slug: cat.slug.clone(),
        }
    }
}

impl From<RawCategory> for Category {
    fn from(cat: RawCategory) -> Self {
        Self {
            id: cat.id,
            is_active: cat.is_active,
            name: cat.name,
            meta_field: cat.meta_field,
            children: vec![],
            parent_id: cat.parent_id,
            level: cat.level,
            attributes: vec![],
            slug: cat.slug,
        }
    }
}

/// Payload for replace category
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CategoryReplacePayload {
    pub current_category: CategoryId,
    pub new_category: CategoryId,
    pub base_product_ids: Option<Vec<BaseProductId>>,
}
