//! Models for managing Roles
use std::time::SystemTime;

use serde_json;

use stq_types::{RoleId, StoresRole, UserId};

use schema::user_roles;

#[derive(Serialize, Queryable, Debug)]
pub struct UserRole {
    pub user_id: UserId,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub name: StoresRole,
    pub data: Option<serde_json::Value>,
    pub id: RoleId,
}

#[derive(Clone, Debug, Serialize, Deserialize, Insertable)]
#[table_name = "user_roles"]
pub struct NewUserRole {
    pub id: Option<RoleId>,
    pub user_id: UserId,
    pub name: StoresRole,
    pub data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoveUserRole {
    pub user_id: UserId,
    pub name: StoresRole,
}
