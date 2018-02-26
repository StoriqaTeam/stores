//! Models for managing currencies

table! {
    currencies (id) {
        id -> Integer,
        name -> VarChar,
    }
}

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "currencies"]
pub struct Currency {
    pub id: i32,
    pub name: String,
}
