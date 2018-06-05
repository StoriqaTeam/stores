//! ModeratorProductComments Services, presents CRUD operations with wizard_stores
use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use failure::Error as FailureError;
use futures::future::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use errors::ControllerError;


use super::types::ServiceFuture;
use models::*;
use repos::ReposFactory;

pub trait ModeratorCommentsService {
    /// Returns latest moderator product comment by base product iD
    fn get_latest_for_product(&self, base_product_id: i32) -> ServiceFuture<Option<ModeratorProductComments>>;
    /// Creates new moderator product comment
    fn create_product_comment(&self, payload: NewModeratorProductComments) -> ServiceFuture<ModeratorProductComments>;
    /// Returns latest moderator comment by store iD
    fn get_latest_for_store(&self, store_id: i32) -> ServiceFuture<Option<ModeratorStoreComments>>;
    /// Creates new moderator store comment
    fn create_store_comment(&self, payload: NewModeratorStoreComments) -> ServiceFuture<ModeratorStoreComments>;
}

/// Moderator comments services
pub struct ModeratorCommentsServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub user_id: Option<i32>,
    pub repo_factory: F,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ModeratorCommentsServiceImpl<T, M, F>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, user_id: Option<i32>, repo_factory: F) -> Self {
        Self {
            db_pool,
            cpu_pool,
            user_id,
            repo_factory,
        }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ModeratorCommentsService for ModeratorCommentsServiceImpl<T, M, F>
{
    /// Returns latest moderator product comment by base product iD
    fn get_latest_for_product(&self, base_product_id: i32) -> ServiceFuture<Option<ModeratorProductComments>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                    .map_err(|e| ControllerError::Connection(e.into()).into())

                .and_then(move |conn| {
                    let moderator_product_repo = repo_factory.create_moderator_product_comments_repo(&*conn, user_id);
                    moderator_product_repo
                        .find_by_base_product_id(base_product_id)
                        
                })
        })
        .map_err(|e| e.context("Service ModeratorComments, get_latest_for_product endpoint error occured.").into())
        )
    }

    /// Creates new moderator product comment
    fn create_product_comment(&self, payload: NewModeratorProductComments) -> ServiceFuture<ModeratorProductComments> {
        let cpu_pool = self.cpu_pool.clone();
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();
        Box::new(
            cpu_pool.spawn_fn(move || {
                db_pool
                    .get()
                    .map_err(|e| ControllerError::Connection(e.into()).into())

                    .and_then(move |conn| {
                        let moderator_product_repo = repo_factory.create_moderator_product_comments_repo(&*conn, user_id);
                        conn.transaction::<ModeratorProductComments, FailureError, _>(move || {
                            moderator_product_repo.create(payload)
                        })
                    })
            })
            .map_err(|e| e.context("Service ModeratorComments, create_product_comment endpoint error occured.").into())
        )
    }

    /// Returns latest moderator comment by store iD
    fn get_latest_for_store(&self, store_id: i32) -> ServiceFuture<Option<ModeratorStoreComments>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                    .map_err(|e| ControllerError::Connection(e.into()).into())

                .and_then(move |conn| {
                    let moderator_store_repo = repo_factory.create_moderator_store_comments_repo(&*conn, user_id);
                    moderator_store_repo.find_by_store_id(store_id)
                })
        })
        .map_err(|e| e.context("Service ModeratorComments, get_latest_for_store endpoint error occured.").into())
        )
    }

    /// Creates new moderator store comment
    fn create_store_comment(&self, payload: NewModeratorStoreComments) -> ServiceFuture<ModeratorStoreComments> {
        let cpu_pool = self.cpu_pool.clone();
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();
        Box::new(
            cpu_pool.spawn_fn(move || {
                db_pool
                    .get()
                    .map_err(|e| ControllerError::Connection(e.into()).into())

                    .and_then(move |conn| {
                        let moderator_store_repo = repo_factory.create_moderator_store_comments_repo(&*conn, user_id);
                        conn.transaction::<ModeratorStoreComments, FailureError, _>(move || {
                            moderator_store_repo.create(payload)
                        })
                    })
            })
        .map_err(|e| e.context("Service ModeratorComments, create_store_comment endpoint error occured.").into())
        )
    }
}

#[cfg(test)]
pub mod tests {
    use futures_cpupool::CpuPool;
    use r2d2;
    use tokio_core::reactor::Core;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    fn create_moderator_comments_service(
        user_id: Option<i32>,
    ) -> ModeratorCommentsServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        ModeratorCommentsServiceImpl {
            db_pool: db_pool,
            cpu_pool: cpu_pool,
            user_id: user_id,
            repo_factory: MOCK_REPO_FACTORY,
        }
    }

    fn create_product_comments_payload() -> NewModeratorProductComments {
        NewModeratorProductComments {
            moderator_id: MOCK_USER_ID,
            base_product_id: 1,
            comments: "new comment".to_string(),
        }
    }

    fn create_store_comments_payload() -> NewModeratorStoreComments {
        NewModeratorStoreComments {
            moderator_id: MOCK_USER_ID,
            store_id: 1,
            comments: "new comment".to_string(),
        }
    }

    #[test]
    fn test_get_product_comment() {
        let mut core = Core::new().unwrap();
        let service = create_moderator_comments_service(Some(MOCK_USER_ID));
        let work = service.get_latest_for_product(1);
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().base_product_id, 1);
    }

    #[test]
    fn test_create_product_comment() {
        let mut core = Core::new().unwrap();
        let service = create_moderator_comments_service(Some(MOCK_USER_ID));
        let payload = create_product_comments_payload();
        let work = service.create_product_comment(payload.clone());
        let result = core.run(work).unwrap();
        assert_eq!(result.comments, payload.comments);
    }

    #[test]
    fn test_get_store_comment() {
        let mut core = Core::new().unwrap();
        let service = create_moderator_comments_service(Some(MOCK_USER_ID));
        let work = service.get_latest_for_store(1);
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().store_id, 1);
    }

    #[test]
    fn test_create_store_comment() {
        let mut core = Core::new().unwrap();
        let service = create_moderator_comments_service(Some(MOCK_USER_ID));
        let payload = create_store_comments_payload();
        let work = service.create_store_comment(payload.clone());
        let result = core.run(work).unwrap();
        assert_eq!(result.comments, payload.comments);
    }

}
