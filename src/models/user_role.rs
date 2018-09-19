//! Models for managing Roles

use std::time;

use stq_types::{StoresRole, UserId};

use schema::user_roles;

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "user_roles"]
pub struct UserRole {
    pub id: i32,
    pub user_id: UserId,
    pub role: StoresRole,
    pub created_at: time::SystemTime,
    pub updated_at: time::SystemTime,
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
