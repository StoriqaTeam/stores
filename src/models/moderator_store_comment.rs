//! Module containg moderator_store_comments model for query, insert, update
use std::time::SystemTime;

use stq_types::{StoreId, UserId};

use schema::moderator_store_comments;

/// Payload for querying wizard_stores
#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Identifiable)]
#[table_name = "moderator_store_comments"]
pub struct ModeratorStoreComments {
    pub id: i32,
    pub moderator_id: UserId,
    pub store_id: StoreId,
    pub comments: String,
    pub created_at: SystemTime,
}

/// Payload for creating wizard_stores
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "moderator_store_comments"]
pub struct NewModeratorStoreComments {
    pub moderator_id: UserId,
    pub store_id: StoreId,
    pub comments: String,
}
