//! EAV model attributes
use serde_json;
use validator::Validate;

use models::*;
use models::validation_rules::*;

table! {
    attributes {
        id -> Integer,
        name -> Jsonb,
        value_type -> VarChar,
        meta_field -> Nullable<Jsonb>,
    }
}

#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "attributes"]
pub struct Attribute {
    pub id: i32,
    pub name: serde_json::Value,
    pub value_type: AttributeType,
    pub meta_field: Option<serde_json::Value>,
}

/// Payload for creating attributes
#[derive(Serialize, Deserialize, Insertable, Clone, Validate)]
#[table_name = "attributes"]
pub struct NewAttribute {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub value_type: AttributeType,
    pub meta_field: Option<serde_json::Value>,
}

/// Payload for updating attributes
#[derive(Serialize, Deserialize, Insertable, AsChangeset, Validate)]
#[table_name = "attributes"]
pub struct UpdateAttribute {
    #[validate(custom = "validate_translation")]
    pub name: Option<serde_json::Value>,
    pub meta_field: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttrValue {
    pub attr_id: i32,
    pub value: String,
    pub meta_field: Option<String>,
}

impl From<ProdAttr> for AttrValue {
    fn from(pr: ProdAttr) -> Self {
        Self {
            attr_id: pr.attr_id,
            value: pr.value,
            meta_field: pr.meta_field,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AttributeType {
    Str,
    Float,
}

mod diesel_impl {
    use std::error::Error;
    use std::io::Write;
    use std::str;

    use diesel::pg::Pg;
    use diesel::row::Row;
    use diesel::expression::bound::Bound;
    use diesel::expression::AsExpression;
    use diesel::types::{FromSqlRow, IsNull, NotNull, SingleValue, ToSql};
    use diesel::serialize::Output;
    use diesel::deserialize::Queryable;
    use diesel::sql_types::VarChar;

    use super::AttributeType;

    impl NotNull for AttributeType {}
    impl SingleValue for AttributeType {}

    impl FromSqlRow<VarChar, Pg> for AttributeType {
        fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>> {
            match row.take() {
                Some(b"str") => Ok(AttributeType::Str),
                Some(b"float") => Ok(AttributeType::Float),
                Some(value) => Err(format!(
                    "Unrecognized enum variant for AttributeType: {}",
                    str::from_utf8(value).unwrap_or("unreadable value")
                ).into()),
                None => Err("Unexpected null for non-null column `role`".into()),
            }
        }
    }

    impl Queryable<VarChar, Pg> for AttributeType {
        type Row = AttributeType;
        fn build(row: Self::Row) -> Self {
            row
        }
    }

    impl ToSql<VarChar, Pg> for AttributeType {
        fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> Result<IsNull, Box<Error + Send + Sync>> {
            match *self {
                AttributeType::Str => out.write_all(b"str")?,
                AttributeType::Float => out.write_all(b"float")?,
            }
            Ok(IsNull::No)
        }
    }

    impl AsExpression<VarChar> for AttributeType {
        type Expression = Bound<VarChar, AttributeType>;
        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl<'a> AsExpression<VarChar> for &'a AttributeType {
        type Expression = Bound<VarChar, &'a AttributeType>;
        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

}
