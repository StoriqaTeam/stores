//! EAV model attributes

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "attribute_type")]
pub enum AttributeTypeValue {
    Bool {
        attribute_name: String,
        attribute_value: bool,
    },
    Enum {
        attribute_name: String,
        attribute_value: String,
    },
    Num {
        attribute_name: String,
        attribute_value: f32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "attribute_type", content = "attribute_name")]
pub enum AttributeType {
    Bool(String),
    Enum(String),
    Num(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AttributeFilter {
    EqualBool {
        attribute_name: String,
        attribute_value: bool,
    },
    EqualEnum {
        attribute_name: String,
        attribute_value: String,
    },
    MinNum {
        attribute_name: String,
        attribute_value: f32,
    },
    MaxNum {
        attribute_name: String,
        attribute_value: f32,
    },
    EqualNum {
        attribute_name: String,
        attribute_value: f32,
    },
    RangeNum {
        attribute_name: String,
        attribute_value_min: f32,
        attribute_value_max: f32,
    },
}
