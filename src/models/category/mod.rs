//! Models contains all structures that are used in different
//! modules of the app
//! EAV model categories
pub mod category_attribute;

use serde_json;
use validator::Validate;

use stq_types::CategoryId;

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
}

/// Used to insert a category into the table
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "categories"]
pub struct InsertCategory {
    pub name: serde_json::Value,
    pub parent_id: CategoryId,
    pub level: i32,
    pub meta_field: Option<serde_json::Value>,
}

/// Payload for creating categories
#[derive(Serialize, Deserialize, Clone, Validate, Debug)]
pub struct NewCategory {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub parent_id: CategoryId,
    pub meta_field: Option<serde_json::Value>,
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: CategoryId,
    pub name: serde_json::Value,
    pub meta_field: Option<serde_json::Value>,
    pub level: i32,
    pub parent_id: Option<CategoryId>,
    pub children: Vec<Category>,
    pub attributes: Vec<Attribute>,
}

impl Category {
    pub const MAX_LEVEL_NESTING: i32 = 3;
}

impl Default for Category {
    fn default() -> Self {
        Self {
            id: CategoryId(0),
            name: serde_json::from_str("[{\"lang\" : \"en\", \"text\" : \"root\"}]").unwrap(),
            meta_field: None,
            children: vec![],
            level: 0,
            parent_id: None,
            attributes: vec![],
        }
    }
}

impl<'a> From<&'a RawCategory> for Category {
    fn from(cat: &'a RawCategory) -> Self {
        Self {
            id: cat.id,
            name: cat.name.clone(),
            meta_field: cat.meta_field.clone(),
            children: vec![],
            parent_id: cat.parent_id,
            level: cat.level,
            attributes: vec![],
        }
    }
}

impl From<RawCategory> for Category {
    fn from(cat: RawCategory) -> Self {
        Self {
            id: cat.id,
            name: cat.name.clone(),
            meta_field: cat.meta_field.clone(),
            children: vec![],
            parent_id: cat.parent_id,
            level: cat.level,
            attributes: vec![],
        }
    }
}
