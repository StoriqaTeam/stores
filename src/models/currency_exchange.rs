//! Models for managing currency exchange
use serde_json;
use std::collections::HashMap;
use std::time::SystemTime;
use stq_static_resources::Currency;
use stq_types::{CurrencyExchangeId, ExchangeRate};

use schema::currency_exchange;

pub type Data = HashMap<Currency, HashMap<Currency, ExchangeRate>>;

#[derive(Clone, Debug, Serialize)]
pub struct CurrencyExchange {
    pub id: CurrencyExchangeId,
    pub data: Data,
    pub created_at: SystemTime,
}

#[derive(Queryable, Insertable, Debug)]
#[table_name = "currency_exchange"]
pub struct DbCurrencyExchange {
    pub id: CurrencyExchangeId,
    pub data: serde_json::Value,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl From<DbCurrencyExchange> for CurrencyExchange {
    fn from(v: DbCurrencyExchange) -> Self {
        Self {
            id: v.id,
            data: serde_json::from_value(v.data).unwrap(),
            created_at: v.created_at,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NewCurrencyExchange {
    pub data: Data,
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "currency_exchange"]
pub struct DbNewCurrencyExchange {
    pub data: serde_json::Value,
}

impl From<NewCurrencyExchange> for DbNewCurrencyExchange {
    fn from(v: NewCurrencyExchange) -> Self {
        Self {
            data: serde_json::to_value(v.data).unwrap(),
        }
    }
}
