//! Repo for static tables - currencies, languages.

use diesel::query_dsl::RunQueryDsl;

use models::currency::currencies::dsl as Currencies;
use models::language::languages::dsl as Languages;
use models::{Language, Currency};

use super::error::Error;
use super::types::{DbConnection, RepoResult};

/// Catalogue repository for handling Catalogue
pub trait CatalogueRepo {
    /// Returns list of languages
    fn languages(&self) -> RepoResult<Vec<Language>>;
    /// Returns list of currencies
    fn currencies(&self) -> RepoResult<Vec<Currency>>;
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
    fn languages(&self) -> RepoResult<Vec<Language>>{
        Languages::languages.load(&**self.db_conn)
        .map_err(|e| Error::from(e))
    }
    /// Returns list of currencies
    fn currencies(&self) -> RepoResult<Vec<Currency>>{
        Currencies::currencies.load(&**self.db_conn)
        .map_err(|e| Error::from(e))
    }

}
