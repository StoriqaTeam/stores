use stq_static_resources::AttributeType;
use stq_types::{BaseProductId, ProductId};

use schema::prod_attr_values;

/// Payload for querying product attributes
#[derive(Debug, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "prod_attr_values"]
pub struct ProdAttr {
    pub id: i32,
    pub prod_id: ProductId,
    pub attr_id: i32,
    pub value: String,
    pub value_type: AttributeType,
    pub meta_field: Option<String>,
    pub base_prod_id: BaseProductId,
}

/// Payload for creating product attributes
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "prod_attr_values"]
pub struct NewProdAttr {
    pub prod_id: ProductId,
    pub base_prod_id: BaseProductId,
    pub attr_id: i32,
    pub value: String,
    pub value_type: AttributeType,
    pub meta_field: Option<String>,
}

impl NewProdAttr {
    pub fn new(
        prod_id: ProductId,
        base_prod_id: BaseProductId,
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
#[derive(Serialize, Deserialize, Insertable, AsChangeset, Debug)]
#[table_name = "prod_attr_values"]
pub struct UpdateProdAttr {
    pub prod_id: ProductId,
    pub base_prod_id: BaseProductId,
    pub attr_id: i32,
    pub value: String,
    pub meta_field: Option<String>,
}

impl UpdateProdAttr {
    pub fn new(prod_id: ProductId, base_prod_id: BaseProductId, attr_id: i32, value: String, meta_field: Option<String>) -> Self {
        Self {
            prod_id,
            base_prod_id,
            attr_id,
            value,
            meta_field,
        }
    }
}
