//! Module containg wizard_stores model for query, insert, update
use validator::Validate;

use stq_types::{StoreId, UserId};

use models::validation_rules::*;
use schema::wizard_stores;

/// Payload for querying wizard_stores
#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Identifiable, Default)]
#[table_name = "wizard_stores"]
pub struct WizardStore {
    pub id: i32,
    pub user_id: UserId,
    pub store_id: Option<StoreId>,
    pub name: Option<String>,
    pub short_description: Option<String>,
    pub default_language: Option<String>,
    pub slug: Option<String>,
    pub country: Option<String>,
    pub address: Option<String>,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub place_id: Option<String>,
    pub completed: bool,
}

/// Payload for creating wizard_stores
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "wizard_stores"]
pub struct NewWizardStore {
    pub user_id: UserId,
}

/// Payload for updating wizard_stores
#[derive(Default, Serialize, Deserialize, Insertable, AsChangeset, Validate, Debug)]
#[table_name = "wizard_stores"]
pub struct UpdateWizardStore {
    pub store_id: Option<StoreId>,
    pub name: Option<String>,
    pub short_description: Option<String>,
    pub default_language: Option<String>,
    #[validate(custom = "validate_slug")]
    pub slug: Option<String>,
    pub country: Option<String>,
    pub address: Option<String>,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub place_id: Option<String>,
}
