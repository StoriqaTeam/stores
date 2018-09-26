//! ModeratorProductComments Services, presents CRUD operations with wizard_stores
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use r2d2::ManageConnection;

use stq_types::{BaseProductId, StoreId};

use super::types::ServiceFuture;
use models::*;
use repos::ReposFactory;
use services::Service;

pub trait ModeratorCommentsService {
    /// Returns latest moderator product comment by base product iD
    fn get_latest_for_product(&self, base_product_id: BaseProductId) -> ServiceFuture<Option<ModeratorProductComments>>;
    /// Creates new moderator product comment
    fn create_product_comment(&self, payload: NewModeratorProductComments) -> ServiceFuture<ModeratorProductComments>;
    /// Returns latest moderator comment by store iD
    fn get_latest_for_store(&self, store_id: StoreId) -> ServiceFuture<Option<ModeratorStoreComments>>;
    /// Creates new moderator store comment
    fn create_store_comment(&self, payload: NewModeratorStoreComments) -> ServiceFuture<ModeratorStoreComments>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ModeratorCommentsService for Service<T, M, F>
{
    /// Returns latest moderator product comment by base product iD
    fn get_latest_for_product(&self, base_product_id: BaseProductId) -> ServiceFuture<Option<ModeratorProductComments>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let moderator_product_repo = repo_factory.create_moderator_product_comments_repo(&*conn, user_id);
            moderator_product_repo.find_by_base_product_id(base_product_id).map_err(|e| {
                e.context("Service ModeratorComments, get_latest_for_product endpoint error occured.")
                    .into()
            })
        })
    }

    /// Creates new moderator product comment
    fn create_product_comment(&self, payload: NewModeratorProductComments) -> ServiceFuture<ModeratorProductComments> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        self.spawn_on_pool(move |conn| {
            let moderator_product_repo = repo_factory.create_moderator_product_comments_repo(&*conn, user_id);
            conn.transaction::<ModeratorProductComments, FailureError, _>(move || moderator_product_repo.create(payload))
                .map_err(|e| {
                    e.context("Service ModeratorComments, create_product_comment endpoint error occured.")
                        .into()
                })
        })
    }

    /// Returns latest moderator comment by store iD
    fn get_latest_for_store(&self, store_id: StoreId) -> ServiceFuture<Option<ModeratorStoreComments>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let moderator_store_repo = repo_factory.create_moderator_store_comments_repo(&*conn, user_id);
            moderator_store_repo.find_by_store_id(store_id).map_err(|e| {
                e.context("Service ModeratorComments, get_latest_for_store endpoint error occured.")
                    .into()
            })
        })
    }

    /// Creates new moderator store comment
    fn create_store_comment(&self, payload: NewModeratorStoreComments) -> ServiceFuture<ModeratorStoreComments> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        self.spawn_on_pool(move |conn| {
            let moderator_store_repo = repo_factory.create_moderator_store_comments_repo(&*conn, user_id);
            conn.transaction::<ModeratorStoreComments, FailureError, _>(move || moderator_store_repo.create(payload))
                .map_err(|e| {
                    e.context("Service ModeratorComments, create_store_comment endpoint error occured.")
                        .into()
                })
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use tokio_core::reactor::Core;

    use stq_types::*;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    fn create_product_comments_payload() -> NewModeratorProductComments {
        NewModeratorProductComments {
            moderator_id: MOCK_USER_ID,
            base_product_id: BaseProductId(1),
            comments: "new comment".to_string(),
        }
    }

    fn create_store_comments_payload() -> NewModeratorStoreComments {
        NewModeratorStoreComments {
            moderator_id: MOCK_USER_ID,
            store_id: StoreId(1),
            comments: "new comment".to_string(),
        }
    }

    #[test]
    fn test_get_product_comment() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_latest_for_product(BaseProductId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().base_product_id, BaseProductId(1));
    }

    #[test]
    fn test_create_product_comment() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let payload = create_product_comments_payload();
        let work = service.create_product_comment(payload.clone());
        let result = core.run(work).unwrap();
        assert_eq!(result.comments, payload.comments);
    }

    #[test]
    fn test_get_store_comment() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_latest_for_store(StoreId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().store_id, StoreId(1));
    }

    #[test]
    fn test_create_store_comment() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let payload = create_store_comments_payload();
        let work = service.create_store_comment(payload.clone());
        let result = core.run(work).unwrap();
        assert_eq!(result.comments, payload.comments);
    }

}
