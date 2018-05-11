//! Module containg wizard_stores model for query, insert, update

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
}

/// Payload for creating wizard_stores
#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "wizard_stores"]
pub struct NewWizardStore {
    pub user_id: i32,
}

/// Payload for updating wizard_stores
#[derive(Default, Serialize, Deserialize, Insertable, AsChangeset, Debug)]
#[table_name = "wizard_stores"]
pub struct UpdateWizardStore {
    pub store_id: Option<i32>,
    pub name: Option<String>,
    pub short_description: Option<String>,
    pub default_language: Option<String>,
    pub slug: Option<String>,
    pub country: Option<String>,
    pub address: Option<String>,
}
