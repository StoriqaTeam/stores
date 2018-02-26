
/// diesel table for products
table! {
    prod_attr_values (id) {
        id -> Integer,
        prod_id -> Integer,
        attr_id -> Integer,
        value -> VarChar, 
    }
}

/// Payload for querying products
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "prod_attr_values"]
pub struct ProdAttr {
    pub id: i32,
    pub prod_id: i32,
    pub attr_id: i32,
    pub value: String,
}

/// Payload for creating products
#[derive(Serialize, Deserialize, Insertable, Clone)]
#[table_name = "prod_attr_values"]
pub struct NewProdAttr {
    pub prod_id: i32,
    pub attr_id: i32,
    pub value: String
}
