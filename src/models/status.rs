//! Enum for Statuss available in ACLs

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub enum Status {
    Draft,
    Moderation,
    Decline,
    Published,
}

mod diesel_impl {
    use std::error::Error;
    use std::io::Write;
    use std::str;

    use diesel::deserialize::Queryable;
    use diesel::expression::bound::Bound;
    use diesel::expression::AsExpression;
    use diesel::pg::Pg;
    use diesel::row::Row;
    use diesel::serialize::Output;
    use diesel::sql_types::VarChar;
    use diesel::types::{FromSqlRow, IsNull, NotNull, SingleValue, ToSql};

    use super::Status;

    impl NotNull for Status {}
    impl SingleValue for Status {}

    impl FromSqlRow<VarChar, Pg> for Status {
        fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>> {
            match row.take() {
                Some(b"draft") => Ok(Status::Draft),
                Some(b"moderation") => Ok(Status::Moderation),
                Some(b"decline") => Ok(Status::Decline),
                Some(b"published") => Ok(Status::Published),
                Some(value) => Err(format!(
                    "Unrecognized enum variant for Status: {}",
                    str::from_utf8(value).unwrap_or("unreadable value")
                ).into()),
                None => Err("Unexpected null for non-null column `Status`".into()),
            }
        }
    }

    impl Queryable<VarChar, Pg> for Status {
        type Row = Status;
        fn build(row: Self::Row) -> Self {
            row
        }
    }

    impl ToSql<VarChar, Pg> for Status {
        fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> Result<IsNull, Box<Error + Send + Sync>> {
            match *self {
                Status::Draft => out.write_all(b"draft")?,
                Status::Moderation => out.write_all(b"moderation")?,
                Status::Decline => out.write_all(b"decline")?,
                Status::Published => out.write_all(b"published")?,
            }
            Ok(IsNull::No)
        }
    }

    impl AsExpression<VarChar> for Status {
        type Expression = Bound<VarChar, Status>;
        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl<'a> AsExpression<VarChar> for &'a Status {
        type Expression = Bound<VarChar, &'a Status>;
        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

}
