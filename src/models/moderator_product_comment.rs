//! Module containg moderator_product_comments model for query, insert, update
use std::time::SystemTime;

use stq_types::{BaseProductId, UserId};

use schema::moderator_product_comments;

/// Payload for querying wizard_stores
#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Identifiable)]
#[table_name = "moderator_product_comments"]
pub struct ModeratorProductComments {
    pub id: i32,
    pub moderator_id: UserId,
    pub base_product_id: BaseProductId,
    pub comments: String,
    pub created_at: SystemTime,
}

/// Payload for creating wizard_stores
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "moderator_product_comments"]
pub struct NewModeratorProductComments {
    pub moderator_id: UserId,
    pub base_product_id: BaseProductId,
    pub comments: String,
}
