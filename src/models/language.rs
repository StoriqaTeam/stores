use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Language {
   English,
   Chinese,
   German,
   Russian,
   Spanish,
   French,
   Korean,
   Portuguese,
   Japanese,
}


impl FromStr for Language {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "English" => Ok(Language::English),
            "Chinese" => Ok(Language::Chinese),
            "German" => Ok(Language::German),
            "Russian" => Ok(Language::Russian),
            "Spanish" => Ok(Language::Spanish),
            "French" => Ok(Language::French),
            "Korean" => Ok(Language::Korean),
            "Portuguese" => Ok(Language::Portuguese),
            "Japanese" => Ok(Language::Japanese),
            _ => Err(()),
        }
    }
}

mod diesel_impl {
    use diesel::Queryable;
    use diesel::expression::AsExpression;
    use diesel::expression::bound::Bound;
    use diesel::pg::Pg;
    use diesel::row::Row;
    use diesel::serialize::{IsNull, ToSql};
    use diesel::serialize::Output;
    use diesel::deserialize::FromSqlRow;
    use diesel::sql_types::*;
    use std::error::Error;
    use std::io::Write;

    use super::{Language};

    impl<'a> AsExpression<VarChar> for &'a Language {
        type Expression = Bound<VarChar, &'a Language>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl AsExpression<VarChar> for Language {
        type Expression = Bound<VarChar, Language>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl ToSql<VarChar, Pg> for Language {
        fn to_sql<W: Write>(
            &self,
            out: &mut Output<W, Pg>,
        ) -> Result<IsNull, Box<Error + Send + Sync>> {
            match *self {
                Language::English => out.write_all(b"English")?,
                Language::Chinese => out.write_all(b"Chinese")?,
                Language::German => out.write_all(b"German")?,
                Language::Russian => out.write_all(b"Russian")?,
                Language::Spanish => out.write_all(b"Spanish")?,
                Language::French => out.write_all(b"French")?,
                Language::Korean => out.write_all(b"Korean")?,
                Language::Portuguese => out.write_all(b"Portuguese")?,
                Language::Japanese => out.write_all(b"Japanese")?,
            }
            Ok(IsNull::No)
        }
    }

    impl FromSqlRow<VarChar, Pg> for Language {
        fn build_from_row<T: Row<Pg>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
            match row.take() {
                Some(b"English") => Ok(Language::English),
                Some(b"Chinese") => Ok(Language::Chinese),
                Some(b"German") => Ok(Language::German),
                Some(b"Russian") => Ok(Language::Russian),
                Some(b"Spanish") => Ok(Language::Spanish),
                Some(b"French") => Ok(Language::French),
                Some(b"Korean") => Ok(Language::Korean),
                Some(b"Portuguese") => Ok(Language::Portuguese),
                Some(b"Japanese") => Ok(Language::Japanese),
                Some(_) => Err("Unrecognized enum variant".into()),
                None => Err("Unrecognized enum variant".into()),
            }
        }
    }

    impl Queryable<VarChar, Pg> for Language {
        type Row = Self;

        fn build(row: Self::Row) -> Self {
            row
        }
    }
}
