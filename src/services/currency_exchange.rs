//! CurrencyExchange Services, presents CRUD operations with user_roles

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use r2d2::ManageConnection;

use super::types::ServiceFuture;
use models::{CurrencyExchange, NewCurrencyExchange};
use repos::ReposFactory;
use services::Service;

pub trait CurrencyExchangeService {
    /// Returns latest currencies exchange
    fn get_latest_currencies(&self) -> ServiceFuture<Option<CurrencyExchange>>;
    /// Updates currencies exchange
    fn update_currencies(&self, payload: NewCurrencyExchange) -> ServiceFuture<CurrencyExchange>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CurrencyExchangeService for Service<T, M, F>
{
    /// Returns latest currencies exchange
    fn get_latest_currencies(&self) -> ServiceFuture<Option<CurrencyExchange>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let currency_exchange_repo = repo_factory.create_currency_exchange_repo(&*conn, user_id);
            currency_exchange_repo.get_latest().map_err(|e| {
                e.context("Service CurrencyExchange, get_latest_currencies endpoint error occurred.")
                    .into()
            })
        })
    }
    /// Updates currencies exchange
    fn update_currencies(&self, payload: NewCurrencyExchange) -> ServiceFuture<CurrencyExchange> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let currency_exchange_repo = repo_factory.create_currency_exchange_repo(&*conn, user_id);
            currency_exchange_repo
                .update(payload)
                .map_err(|e| e.context("Service CurrencyExchange, update endpoint error occurred.").into())
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use tokio_core::reactor::Core;

    use stq_static_resources::Currency;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    pub fn create_new_currency_exchange() -> NewCurrencyExchange {
        NewCurrencyExchange {
            data: Currency::enum_iter().map(|cur| (cur, Default::default())).collect(),
        }
    }

    #[test]
    fn test_get_latest() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_latest_currencies();
        let result = core.run(work);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_update_currency() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_currency_exchange = create_new_currency_exchange();
        let work = service.update_currencies(new_currency_exchange);
        let result = core.run(work);
        assert_eq!(result.is_ok(), true);
    }

}
