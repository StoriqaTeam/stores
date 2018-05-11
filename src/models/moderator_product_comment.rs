//! Module containg moderator_product_comments model for query, insert, update
use std::time::SystemTime;

/// diesel table for moderator_product_comments
table! {
    moderator_product_comments (id) {
        id -> Integer,
        moderator_id -> Integer,
        base_product_id -> Integer,
        comments -> VarChar,
        created_at -> Timestamp,
    }
}

/// Payload for querying wizard_stores
#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Identifiable)]
#[table_name = "moderator_product_comments"]
pub struct ModeratorProductComments {
    pub id: i32,
    pub moderator_id: i32,
    pub base_product_id: i32,
    pub comments: String,
    pub created_at: SystemTime,
}

/// Payload for creating wizard_stores
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "moderator_product_comments"]
pub struct NewModeratorProductComments {
    pub moderator_id: i32,
    pub base_product_id: i32,
    pub comments: String,
}
