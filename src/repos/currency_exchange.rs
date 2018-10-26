use std::collections::HashMap;

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_static_resources::Currency;
use stq_types::{ExchangeRate, UserId};

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::{CurrencyExchange, DbCurrencyExchange, DbNewCurrencyExchange, NewCurrencyExchange};
use repos::legacy_acl::*;
use schema::currency_exchange::dsl::*;

/// CurrencyExchange repository, responsible for handling prod_attr_values
pub struct CurrencyExchangeRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, CurrencyExchange>>,
}

pub trait CurrencyExchangeRepo {
    /// Get latest currency exchanges
    fn get_latest(&self) -> RepoResult<Option<CurrencyExchange>>;

    /// Get latest currency exchanges for currency
    fn get_exchange_for_currency(&self, currency: Currency) -> RepoResult<Option<HashMap<Currency, ExchangeRate>>>;

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
        let query = currency_exchange.order_by(created_at.desc()).limit(1);

        query
            .first(self.db_conn)
            .optional()
            .map(|v: Option<DbCurrencyExchange>| v.map(CurrencyExchange::from))
            .map_err(From::from)
            .and_then(|currency_exchange_arg: Option<CurrencyExchange>| {
                if let Some(ref currency_exchange_arg) = currency_exchange_arg {
                    acl::check(
                        &*self.acl,
                        Resource::CurrencyExchange,
                        Action::Read,
                        self,
                        Some(currency_exchange_arg),
                    )?;
                };
                Ok(currency_exchange_arg)
            }).map_err(|e: FailureError| e.context("Find latest currency error occurred").into())
    }

    /// Get latest rates for currency
    fn get_exchange_for_currency(&self, currency: Currency) -> RepoResult<Option<HashMap<Currency, ExchangeRate>>> {
        self.get_latest()
            .map(|v| v.and_then(|mut all_rates| all_rates.data.remove(&currency)))
    }

    /// Adds latest currency to table
    fn update(&self, payload: NewCurrencyExchange) -> RepoResult<CurrencyExchange> {
        debug!("Add latest currency {:?}.", payload);
        let payload = DbNewCurrencyExchange::from(payload);
        let query = diesel::insert_into(currency_exchange).values(&payload);
        query
            .get_result::<DbCurrencyExchange>(self.db_conn)
            .map(CurrencyExchange::from)
            .map_err(From::from)
            .and_then(|currency_exchange_arg| {
                acl::check(
                    &*self.acl,
                    Resource::CurrencyExchange,
                    Action::Create,
                    self,
                    Some(&currency_exchange_arg),
                )?;
                Ok(currency_exchange_arg)
            }).map_err(|e: FailureError| e.context("Adds latest currency to table error occurred").into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, CurrencyExchange>
    for CurrencyExchangeRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&CurrencyExchange>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}
