use std::collections::HashMap;
use std::str::FromStr;

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use repos::legacy_acl::*;
use stq_static_resources::Currency;

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::currency_exchange::currency_exchange::dsl::*;
use models::{CurrencyExchange, NewCurrencyExchange};

/// CurrencyExchange repository, responsible for handling prod_attr_values
pub struct CurrencyExchangeRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, CurrencyExchange>>,
}

pub trait CurrencyExchangeRepo {
    /// Get latest currency exchanges
    fn get_latest(&self) -> RepoResult<Option<CurrencyExchange>>;

    /// Get latest currency exchanges for currency_id
    fn get_exchange_for_currency(&self, currency_id: i32) -> RepoResult<Option<HashMap<i32, f64>>>;

    /// Adds latest currency to table
    fn update(&self, payload: NewCurrencyExchange) -> RepoResult<CurrencyExchange>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CurrencyExchangeRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, CurrencyExchange>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CurrencyExchangeRepo
    for CurrencyExchangeRepoImpl<'a, T>
{
    /// Get latest currency exchanges
    fn get_latest(&self) -> RepoResult<Option<CurrencyExchange>> {
        debug!("Find latest currency.");
        let query = currency_exchange.order_by(id.desc()).limit(1);

        query
            .first(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|currency_exchange_arg: Option<CurrencyExchange>| {
                if let Some(ref currency_exchange_arg) = currency_exchange_arg {
                    acl::check(
                        &*self.acl,
                        &Resource::CurrencyExchange,
                        &Action::Read,
                        self,
                        Some(currency_exchange_arg),
                    )?;
                };
                Ok(currency_exchange_arg)
            })
            .map_err(|e: FailureError| e.context("Find latest currency error occured").into())
    }

    /// Get latest currency exchanges for currency_id
    fn get_exchange_for_currency(&self, currency_id: i32) -> RepoResult<Option<HashMap<i32, f64>>> {
        self.get_latest().map(|currencies| {
            currencies.and_then(|currencies| {
                match currency_id {
                    x if x == (Currency::Rouble as i32) => currencies.rouble,
                    x if x == (Currency::Euro as i32) => currencies.euro,
                    x if x == (Currency::Dollar as i32) => currencies.dollar,
                    x if x == (Currency::Bitcoin as i32) => currencies.bitcoin,
                    x if x == (Currency::Etherium as i32) => currencies.etherium,
                    x if x == (Currency::Stq as i32) => currencies.stq,
                    _ => return None,
                }.as_object()
                    .map(|m| {
                        let mut map = HashMap::<i32, f64>::new();
                        for (key, val) in m {
                            if let Ok(key) = Currency::from_str(key) {
                                if let Some(val) = val.as_f64() {
                                    map.insert(key as i32, val);
                                }
                            }
                        }
                        map
                    })
            })
        })
    }

    /// Adds latest currency to table
    fn update(&self, payload: NewCurrencyExchange) -> RepoResult<CurrencyExchange> {
        debug!("Add latest currency {:?}.", payload);
        let query = diesel::insert_into(currency_exchange).values(&payload);
        query
            .get_result::<CurrencyExchange>(self.db_conn)
            .map_err(From::from)
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
            .map_err(|e: FailureError| e.context("Adds latest currency to table error occured").into())
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
