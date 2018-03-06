//! EAV model attributes

table! {
    attributes {
        id -> Integer,
        name -> VarChar,
        meta_field -> Nullable<VarChar>,
    }
}

#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "attributes"]
pub struct Attribute {
    pub id: i32,
    pub name: String,
    pub meta_field: Option<String>,
}

/// Payload for creating attributes
#[derive(Serialize, Deserialize, Insertable, Clone)]
#[table_name = "attributes"]
pub struct NewAttribute {
    pub name: String,
    pub meta_field: Option<String>,
}

/// Payload for updating attributes
#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
#[table_name = "attributes"]
pub struct UpdateAttribute {
    pub name: Option<String>,
    pub meta_field: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, ElasticType)]
pub struct ElasticAttribute {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, ElasticType)]
pub struct SearchAttribute {
    pub name: String,
}
