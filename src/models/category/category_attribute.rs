/// diesel table for category attributes
table! {
    cat_attr_values (id) {
        id -> Integer,
        cat_id -> Integer,
        attr_id -> Integer,
    }
}

/// Payload for querying category attributes
#[derive(Debug, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "cat_attr_values"]
pub struct CatAttr {
    pub id: i32,
    pub cat_id: i32,
    pub attr_id: i32,
}

/// Payload for creating category attributes
#[derive(Serialize, Deserialize, Insertable, Clone)]
#[table_name = "cat_attr_values"]
pub struct NewCatAttr {
    pub cat_id: i32,
    pub attr_id: i32,
}

/// Payload for updating category attributes
#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
#[table_name = "cat_attr_values"]
pub struct OldCatAttr {
    pub cat_id: i32,
    pub attr_id: i32,
}
