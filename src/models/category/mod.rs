//! Models contains all structures that are used in different
//! modules of the app
//! EAV model categories
use serde_json;
use validator::Validate;

use models::validation_rules::*;
use models::Attribute;

pub mod category_attribute;
pub use self::category_attribute::*;

table! {
    categories {
        id -> Integer,
        name -> Jsonb,
        meta_field -> Nullable<Jsonb>,
        parent_id -> Nullable<Integer>,
        level -> Integer,
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
    pub level: i32,
}

/// Payload for creating categories
#[derive(Serialize, Deserialize, Insertable, Clone, Validate, Debug)]
#[table_name = "categories"]
pub struct NewCategory {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub meta_field: Option<serde_json::Value>,
    pub parent_id: Option<i32>,
    #[validate(range(min = "1", max = "3"))]
    pub level: i32,
}

/// Payload for updating categories
#[derive(Serialize, Deserialize, Insertable, AsChangeset, Validate, Debug)]
#[table_name = "categories"]
pub struct UpdateCategory {
    #[validate(custom = "validate_translation")]
    pub name: Option<serde_json::Value>,
    pub meta_field: Option<serde_json::Value>,
    pub parent_id: Option<i32>,
    #[validate(range(min = "1", max = "3"))]
    pub level: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i32,
    pub name: serde_json::Value,
    pub meta_field: Option<serde_json::Value>,
    pub level: i32,
    pub parent_id: Option<i32>,
    pub children: Vec<Category>,
    pub attributes: Option<Vec<Attribute>>,
}

impl Default for Category {
    fn default() -> Self {
        Self {
            id: 0,
            name: serde_json::from_str("[{\"lang\" : \"en\", \"text\" : \"root\"}]").unwrap(),
            meta_field: None,
            children: vec![],
            level: 0,
            parent_id: None,
            attributes: None,
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
            attributes: None,
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
            attributes: None,
        }
    }
}
