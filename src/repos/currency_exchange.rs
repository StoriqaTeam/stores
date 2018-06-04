use std::convert::From;

use diesel;
use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;

use stq_acl::*;

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::currency_exchange::currency_exchange::dsl::*;
use models::{CurrencyExchange, NewCurrencyExchange};
use repos::error::RepoError as Error;

/// CurrencyExchange repository, responsible for handling prod_attr_values
pub struct CurrencyExchangeRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, Error, CurrencyExchange>>,
}

pub trait CurrencyExchangeRepo {
    /// Get latest currency exchanges
    fn get_latest(&self) -> RepoResult<CurrencyExchange>;

    /// Adds latest currency to table
    fn update(&self, payload: NewCurrencyExchange) -> RepoResult<CurrencyExchange>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CurrencyExchangeRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, Error, CurrencyExchange>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CurrencyExchangeRepo
    for CurrencyExchangeRepoImpl<'a, T>
{
    /// Get latest currency exchanges
    fn get_latest(&self) -> RepoResult<CurrencyExchange> {
        debug!("Find latest currency.");
        let query = currency_exchange.order_by(id.desc()).limit(1);

        query
            .get_results(self.db_conn)
            .map_err(Error::from)
            .and_then(|currency_exchange_arg: Vec<CurrencyExchange>| {
                if let Some(c) = currency_exchange_arg.into_iter().nth(0) {
                    acl::check(&*self.acl, &Resource::CurrencyExchange, &Action::Read, self, Some(&c))?;
                    Ok(c)
                } else {
                    Err(Error::NotFound)
                }
            })
    }

    /// Adds latest currency to table
    fn update(&self, payload: NewCurrencyExchange) -> RepoResult<CurrencyExchange> {
        debug!("Add latest currency {:?}.", payload);
        let query = diesel::insert_into(currency_exchange).values(&payload);
        query
            .get_result::<CurrencyExchange>(self.db_conn)
            .map_err(Error::from)
            .and_then(|currency_exchange_arg| {
                acl::check(
                    &*self.acl,
                    &Resource::CurrencyExchange,
                    &Action::Create,
                    self,
                    Some(&currency_exchange_arg),
                )?;
                Ok(currency_exchange_arg)
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, CurrencyExchange>
    for CurrencyExchangeRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: i32, scope: &Scope, _obj: Option<&CurrencyExchange>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
