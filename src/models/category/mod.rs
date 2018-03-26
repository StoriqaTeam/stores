//! Models contains all structures that are used in different
//! modules of the app
//! EAV model categories
use serde_json;
use validator::Validate;

use models::validation_rules::*;

pub mod category_attribute;

pub use self::category_attribute::*;

table! {
    categories {
        id -> Integer,
        name -> Jsonb,
        meta_field -> Nullable<Jsonb>,
        parent_id -> Nullable<Integer>,
    }
}

/// RawCategory is an object stored in PG, used only for Category tree creation,
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "categories"]
pub struct RawCategory {
    pub id: i32,
    pub name: serde_json::Value,
    pub meta_field: Option<serde_json::Value>,
    pub parent_id: Option<i32>,
}

/// Payload for creating categories
#[derive(Serialize, Deserialize, Insertable, Clone, Validate, Debug)]
#[table_name = "categories"]
pub struct NewCategory {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub meta_field: Option<serde_json::Value>,
    pub parent_id: Option<i32>,
}

/// Payload for updating categories
#[derive(Serialize, Deserialize, Insertable, AsChangeset, Validate, Debug)]
#[table_name = "categories"]
pub struct UpdateCategory {
    #[validate(custom = "validate_translation")]
    pub name: Option<serde_json::Value>,
    pub meta_field: Option<serde_json::Value>,
    pub parent_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i32,
    pub name: serde_json::Value,
    pub meta_field: Option<serde_json::Value>,
    pub children: Vec<Category>,
}

impl Default for Category {
    fn default() -> Self {
        Self {
            id: 0,
            name: serde_json::from_str("[{\"lang\" : \"en\", \"text\" : \"root\"}]").unwrap(),
            meta_field: None,
            children: vec![],
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
        }
    }
}
