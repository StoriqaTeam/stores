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
        meta_field -> Nullable<VarChar>,
        parent_id -> Nullable<Integer>,
    }
}

#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "categories"]
pub struct Category {
    pub id: i32,
    pub name: serde_json::Value,
    pub meta_field: Option<String>,
    pub parent_id: Option<i32>,
}

/// Payload for creating categories
#[derive(Serialize, Deserialize, Insertable, Clone, Validate)]
#[table_name = "categories"]
pub struct NewCategory {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub meta_field: Option<String>,
    pub parent_id: Option<i32>,
}

/// Payload for updating categories
#[derive(Serialize, Deserialize, Insertable, AsChangeset, Validate)]
#[table_name = "categories"]
pub struct UpdateCategory {
    #[validate(custom = "validate_translation")]
    pub name: Option<serde_json::Value>,
    pub meta_field: Option<String>,
    pub parent_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryTree {
    pub id: i32,
    pub name: serde_json::Value,
    pub meta_field: Option<String>,
    pub childs: Vec<CategoryTree>,
}

impl<'a> From<&'a Category> for CategoryTree {
    fn from(cat: &'a Category) -> Self {
        Self {
            id: cat.id,
            name: cat.name.clone(),
            meta_field: cat.meta_field.clone(),
            childs: vec![]
        }
    }
}