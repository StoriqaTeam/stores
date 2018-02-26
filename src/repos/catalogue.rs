//! Repo for static tables - currencies, languages.

use diesel::query_dsl::RunQueryDsl;

use models::currency::currencies::dsl as Currencies;
use models::language::languages::dsl as Languages;
use models::Language;
use models::Currency;

use super::error::Error;
use super::types::{DbConnection, RepoResult};

/// Catalogue repository for handling Catalogue
pub trait CatalogueRepo {
    /// Returns list of languages
    fn languages(&self) -> RepoResult<Vec<String>>;
    /// Returns list of currencies
    fn currencies(&self) -> RepoResult<Vec<String>>;
}

/// Implementation of Catalogue trait
pub struct CatalogueRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
}

impl<'a> CatalogueRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection) -> Self {
        Self { db_conn }
    }
}

impl<'a> CatalogueRepo for CatalogueRepoImpl<'a> {

     /// Returns list of languages
    fn languages(&self) -> RepoResult<Vec<String>>{
        Languages::languages.load(&**self.db_conn)
        .map_err(|e| Error::from(e))
        .and_then(|langs| Ok(langs.into_iter().map(|lang : Language| lang.name).collect()))
    }
    /// Returns list of currencies
    fn currencies(&self) -> RepoResult<Vec<String>>{
        Currencies::currencies.load(&**self.db_conn)
        .map_err(|e| Error::from(e))
        .and_then(|currencies| Ok(currencies.into_iter().map(|currency : Currency| currency.name).collect()))
    }

}
