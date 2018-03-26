use diesel::prelude::*;
use stq_acl::WithScope;

use models::product::products::dsl as Products;
use models::{AttributeType, Product, Scope};
use repos::types::DbConnection;

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
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
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
#[derive(Serialize, Deserialize, Insertable, AsChangeset, Debug)]
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

impl WithScope<Scope> for ProdAttr {
    fn is_in_scope(&self, scope: &Scope, user_id: i32, conn: Option<&DbConnection>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(conn) = conn {
                    Products::products
                        .find(self.prod_id)
                        .get_result::<Product>(&**conn)
                        .and_then(|product: Product| Ok(product.is_in_scope(scope, user_id, Some(conn))))
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}

impl WithScope<Scope> for NewProdAttr {
    fn is_in_scope(&self, scope: &Scope, user_id: i32, conn: Option<&DbConnection>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(conn) = conn {
                    Products::products
                        .find(self.prod_id)
                        .get_result::<Product>(&**conn)
                        .and_then(|product: Product| Ok(product.is_in_scope(scope, user_id, Some(conn))))
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}
