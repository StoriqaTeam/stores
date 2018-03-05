//! EAV model attributes

table! {
    attributes {
        id -> Integer,
        name -> VarChar,
        ui_type -> VarChar,
    }
}

#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[table_name = "attributes"]
pub struct Attribute {
    pub id: i32,
    pub name: String,
    pub ui_type: WidgetType,
}

/// Payload for creating attributes
#[derive(Serialize, Deserialize, Insertable, Clone)]
#[table_name = "attributes"]
pub struct NewAttribute {
    pub name: String,
    pub ui_type: WidgetType,
}

/// Payload for updating attributes
#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
#[table_name = "attributes"]
pub struct UpdateAttribute {
    pub name: Option<String>,
    pub ui_type: Option<WidgetType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WidgetType {
    ComboBox,
    CheckBox,
    TextBox,
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

    use super::WidgetType;

    impl NotNull for WidgetType {}
    impl SingleValue for WidgetType {}

    impl FromSqlRow<VarChar, Pg> for WidgetType {
        fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>> {
            match row.take() {
                Some(b"combobox") => Ok(WidgetType::ComboBox),
                Some(b"checkbox") => Ok(WidgetType::CheckBox),
                Some(b"textbox") => Ok(WidgetType::TextBox),
                Some(value) => Err(format!(
                    "Unrecognized enum variant for WidgetType: {}",
                    str::from_utf8(value).unwrap_or("unreadable value")
                ).into()),
                None => Err("Unexpected null for non-null column `role`".into()),
            }
        }
    }

    impl Queryable<VarChar, Pg> for WidgetType {
        type Row = WidgetType;
        fn build(row: Self::Row) -> Self {
            row
        }
    }

    impl ToSql<VarChar, Pg> for WidgetType {
        fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> Result<IsNull, Box<Error + Send + Sync>> {
            match *self {
                WidgetType::ComboBox => out.write_all(b"combobox")?,
                WidgetType::CheckBox => out.write_all(b"checkbox")?,
                WidgetType::TextBox => out.write_all(b"textbox")?,
            }
            Ok(IsNull::No)
        }
    }

    impl AsExpression<VarChar> for WidgetType {
        type Expression = Bound<VarChar, WidgetType>;
        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl<'a> AsExpression<VarChar> for &'a WidgetType {
        type Expression = Bound<VarChar, &'a WidgetType>;
        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

}
