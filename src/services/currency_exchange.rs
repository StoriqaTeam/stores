//! CurrencyExchange Services, presents CRUD operations with user_roles

use futures_cpupool::CpuPool;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future::*;
use r2d2::{ManageConnection, Pool};

use errors::ControllerError;

use super::types::ServiceFuture;
use models::{CurrencyExchange, NewCurrencyExchange};
use repos::ReposFactory;

pub trait CurrencyExchangeService {
    /// Returns latest currencies exchange
    fn get_latest(&self) -> ServiceFuture<Option<CurrencyExchange>>;
    /// Updates currencies exchange
    fn update(&self, payload: NewCurrencyExchange) -> ServiceFuture<CurrencyExchange>;
}

/// CurrencyExchange services, responsible for UserRole-related CRUD operations
pub struct CurrencyExchangeServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub repo_factory: F,
    pub user_id: Option<i32>,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CurrencyExchangeServiceImpl<T, M, F>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, user_id: Option<i32>, repo_factory: F) -> Self {
        Self {
            db_pool,
            cpu_pool,
            repo_factory,
            user_id,
        }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > CurrencyExchangeService for CurrencyExchangeServiceImpl<T, M, F>
{
    /// Returns latest currencies exchange
    fn get_latest(&self) -> ServiceFuture<Option<CurrencyExchange>> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(ControllerError::Connection).into())
                        .and_then(move |conn| {
                            let currency_exchange_repo = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                            currency_exchange_repo.get_latest()
                        })
                })
                .map_err(|e| e.context("Service CurrencyExchange, get_latest endpoint error occured.").into()),
        )
    }
    /// Updates currencies exchange
    fn update(&self, payload: NewCurrencyExchange) -> ServiceFuture<CurrencyExchange> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(ControllerError::Connection).into())
                        .and_then(move |conn| {
                            let currency_exchange_repo = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                            currency_exchange_repo.update(payload)
                        })
                })
                .map_err(|e| e.context("Service CurrencyExchange, update endpoint error occured.").into()),
        )
    }
}

#[cfg(test)]
pub mod tests {
    use futures_cpupool::CpuPool;
    use r2d2;
    use serde_json;
    use tokio_core::reactor::Core;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    fn create_currency_exchange_service() -> CurrencyExchangeServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        CurrencyExchangeServiceImpl {
            db_pool: db_pool,
            cpu_pool: cpu_pool,
            repo_factory: MOCK_REPO_FACTORY,
            user_id: Some(1),
        }
    }

    pub fn create_new_currency_exchange() -> NewCurrencyExchange {
        NewCurrencyExchange {
            rouble: serde_json::from_str("{}").unwrap(),
            euro: serde_json::from_str("{}").unwrap(),
            dollar: serde_json::from_str("{}").unwrap(),
            bitcoin: serde_json::from_str("{}").unwrap(),
            etherium: serde_json::from_str("{}").unwrap(),
            stq: serde_json::from_str("{}").unwrap(),
        }
    }

    #[test]
    fn test_get_latest() {
        let mut core = Core::new().unwrap();
        let service = create_currency_exchange_service();
        let work = service.get_latest();
        let result = core.run(work);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_update_currency() {
        let mut core = Core::new().unwrap();
        let service = create_currency_exchange_service();
        let new_currency_exchange = create_new_currency_exchange();
        let work = service.update(new_currency_exchange);
        let result = core.run(work);
        assert_eq!(result.is_ok(), true);
    }

}
