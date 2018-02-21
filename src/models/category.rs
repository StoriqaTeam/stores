//! EAV model categories
use std::fmt;

table! {
    categories {
        id -> Integer,
        category -> Jsonb,
    }
}

pub type CategoryId = i32;
pub type AttributeName = String;

/// Payload for creating stores
#[derive(Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub sub_categories: Vec<Category>,
    pub attributes: Vec<AttributeName>
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttributeValue {
    #[serde(rename="attribute_name")]
    pub name: AttributeName,
    pub value: AttributeType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "attribute_type", content = "attribute_value")]
pub enum AttributeType {
    Bool(bool),
    Enum(String),
    Num(f32),
}


impl fmt::Display for AttributeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = match *self {
            AttributeType::Bool(_) => "bool",
            AttributeType::Enum(_) =>  "enum",
            AttributeType::Num(_) =>  "num",
        };
        write!(f, "{}", res)
    }
}

