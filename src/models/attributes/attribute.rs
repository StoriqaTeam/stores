//! EAV model attributes

table! {
    attributes {
        id -> Integer,
        name -> VarChar,
        ty -> VarChar,
    }
}

#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "attributes"]
pub struct Attribute {
    pub id: i32,
    pub name: String,
    pub ty: AttributeType
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "attribute_type")]
pub enum AttributeType {
    Str,
    Float
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "attribute_type", content = "attribute_value")]
pub enum AttributeValue {
    Str(String),
    Float(f32)
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