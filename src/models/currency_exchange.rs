//! Models for managing currency exchange
use std::time::SystemTime;

use serde_json;

table! {
    currency_exchange (id) {
        id -> Integer,
        rouble -> Jsonb,
        euro -> Jsonb,
        dollar -> Jsonb,
        bitcoin -> Jsonb,
        etherium -> Jsonb,
        stq -> Jsonb,
        created_at -> Timestamp, // UTC 0, generated at db level
        updated_at -> Timestamp, // UTC 0, generated at db level
    }
}

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "currency_exchange"]
pub struct CurrencyExchange {
    pub id: i32,
    pub rouble: serde_json::Value,
    pub euro: serde_json::Value,
    pub dollar: serde_json::Value,
    pub bitcoin: serde_json::Value,
    pub etherium: serde_json::Value,
    pub stq: serde_json::Value,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "currency_exchange"]
pub struct NewCurrencyExchange {
    pub rouble: serde_json::Value,
    pub euro: serde_json::Value,
    pub dollar: serde_json::Value,
    pub bitcoin: serde_json::Value,
    pub etherium: serde_json::Value,
    pub stq: serde_json::Value,
}
