//! Models for managing currency exchange
use std::time::SystemTime;

use serde_json;
use validator::Validate;

use models::validation_rules::*;

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

#[derive(Serialize, Deserialize, Insertable, Clone, Debug, Validate)]
#[table_name = "currency_exchange"]
pub struct NewCurrencyExchange {
    #[validate(custom = "validate_currencies")]
    pub rouble: serde_json::Value,
    #[validate(custom = "validate_currencies")]
    pub euro: serde_json::Value,
    #[validate(custom = "validate_currencies")]
    pub dollar: serde_json::Value,
    #[validate(custom = "validate_currencies")]
    pub bitcoin: serde_json::Value,
    #[validate(custom = "validate_currencies")]
    pub etherium: serde_json::Value,
    #[validate(custom = "validate_currencies")]
    pub stq: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CurrencyExchangeValue {
    pub rouble: f64,
    pub euro: f64,
    pub dollar: f64,
    pub bitcoin: f64,
    pub etherium: f64,
    pub stq: f64,
}
