use models::AttributeType;

/// diesel table for product attributes
table! {
    prod_attr_values (id) {
        id -> Integer,
        prod_id -> Integer,
        base_prod_id -> Integer,
        attr_id -> Integer,
        value -> VarChar,
        value_type -> VarChar,
        meta_field -> Nullable<VarChar>,
    }
}

/// Payload for querying product attributes
#[derive(Debug, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "prod_attr_values"]
pub struct ProdAttr {
    pub id: i32,
    pub prod_id: i32,
    pub base_prod_id: i32,
    pub attr_id: i32,
    pub value: String,
    pub value_type: AttributeType,
    pub meta_field: Option<String>,
}

/// Payload for creating product attributes
#[derive(Serialize, Deserialize, Insertable, Clone)]
#[table_name = "prod_attr_values"]
pub struct NewProdAttr {
    pub prod_id: i32,
    pub base_prod_id: i32,
    pub attr_id: i32,
    pub value: String,
    pub value_type: AttributeType,
    pub meta_field: Option<String>,
}

impl NewProdAttr {
    pub fn new(
        prod_id: i32,
        base_prod_id: i32,
        attr_id: i32,
        value: String,
        value_type: AttributeType,
        meta_field: Option<String>,
    ) -> Self {
        Self {
            prod_id,
            base_prod_id,
            attr_id,
            value,
            value_type,
            meta_field,
        }
    }
}

/// Payload for updating product attributes
#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
#[table_name = "prod_attr_values"]
pub struct UpdateProdAttr {
    pub prod_id: i32,
    pub base_prod_id: i32,
    pub attr_id: i32,
    pub value: String,
    pub meta_field: Option<String>,
}

impl UpdateProdAttr {
    pub fn new(prod_id: i32, base_prod_id: i32, attr_id: i32, value: String, meta_field: Option<String>) -> Self {
        Self {
            prod_id,
            base_prod_id,
            attr_id,
            value,
            meta_field,
        }
    }
}
