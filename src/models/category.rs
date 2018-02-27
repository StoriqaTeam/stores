//! EAV model categories
use super::AttributeType;

table! {
    categories {
        id -> Integer,
        category -> Jsonb,
    }
}

pub type CategoryId = i32;

/// Payload for creating stores
#[derive(Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub sub_categories: Vec<Category>,
    pub attributes: Vec<AttributeType>,
}
