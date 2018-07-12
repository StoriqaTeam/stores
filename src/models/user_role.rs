//! Models for managing Roles
use stq_types::{StoresRole, UserId};

table! {
    user_roles (id) {
        id -> Integer,
        user_id -> Integer,
        role -> VarChar,
    }
}

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "user_roles"]
pub struct UserRole {
    pub id: i32,
    pub user_id: UserId,
    pub role: StoresRole,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "user_roles"]
pub struct NewUserRole {
    pub user_id: UserId,
    pub role: StoresRole,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "user_roles"]
pub struct OldUserRole {
    pub user_id: UserId,
    pub role: StoresRole,
}
