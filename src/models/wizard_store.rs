//! Module containg wizard_stores model for query, insert, update
use validator::Validate;

use models::validation_rules::*;

/// diesel table for wizard_stores
table! {
    wizard_stores (id) {
        id -> Integer,
        user_id -> Integer,
        store_id -> Nullable<Integer>,
        name -> Nullable<VarChar>,
        short_description -> Nullable<VarChar>,
        default_language -> Nullable<VarChar>,
        slug -> Nullable<VarChar>,
        country -> Nullable<VarChar>,
        address -> Nullable<VarChar>,
        administrative_area_level_1 -> Nullable<VarChar>,
        administrative_area_level_2 -> Nullable<VarChar>,
        locality -> Nullable<VarChar>,
        political -> Nullable<VarChar>,
        postal_code -> Nullable<VarChar>,
        route -> Nullable<VarChar>,
        street_number -> Nullable<VarChar>,
        place_id -> Nullable<VarChar>,
    }
}

/// Payload for querying wizard_stores
#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Identifiable, Default)]
#[table_name = "wizard_stores"]
pub struct WizardStore {
    pub id: i32,
    pub user_id: i32,
    pub store_id: Option<i32>,
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
}

/// Payload for creating wizard_stores
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "wizard_stores"]
pub struct NewWizardStore {
    pub user_id: i32,
}

/// Payload for updating wizard_stores
#[derive(Default, Serialize, Deserialize, Insertable, AsChangeset, Validate, Debug)]
#[table_name = "wizard_stores"]
pub struct UpdateWizardStore {
    pub store_id: Option<i32>,
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
