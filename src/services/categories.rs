//! Categories Services, presents CRUD operations with categories

use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use failure::Error as FailureError;
use futures_cpupool::CpuPool;
use futures::future::*;
use r2d2::{ManageConnection, Pool};

use errors::ControllerError;

use super::types::ServiceFuture;
use models::{Attribute, NewCatAttr, OldCatAttr};
use models::{Category, NewCategory, UpdateCategory};
use repos::ReposFactory;
use repos::types::RepoResult;

pub trait CategoriesService {
    /// Returns category by ID
    fn get(&self, category_id: i32) -> ServiceFuture<Option<Category>>;
    /// Creates new category
    fn create(&self, payload: NewCategory) -> ServiceFuture<Category>;
    /// Updates specific category
    fn update(&self, category_id: i32, payload: UpdateCategory) -> ServiceFuture<Category>;
    /// Returns all categories as a tree
    fn get_all(&self) -> ServiceFuture<Category>;
    /// Returns all category attributes belonging to category
    fn find_all_attributes(&self, category_id_arg: i32) -> ServiceFuture<Vec<Attribute>>;
    /// Creates new category attribute
    fn add_attribute_to_category(&self, payload: NewCatAttr) -> ServiceFuture<()>;
    /// Deletes category attribute
    fn delete_attribute_from_category(&self, payload: OldCatAttr) -> ServiceFuture<()>;
}

/// Categories services, responsible for Category-related CRUD operations
pub struct CategoriesServiceImpl<
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
    > CategoriesServiceImpl<T, M, F>
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
    > CategoriesService for CategoriesServiceImpl<T, M, F>
{
    /// Returns category by ID
    fn get(&self, category_id: i32) -> ServiceFuture<Option<Category>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ControllerError::Connection(e.into()).into())
                .and_then(move |conn| {
                    let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                    categories_repo.find(category_id)
                })
        })
        .map_err(|e| e.context("Service Categories, get endpoint error occured.").into())
        )
    }

    /// Creates new category
    fn create(&self, new_category: NewCategory) -> ServiceFuture<Category> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ControllerError::Connection(e.into()).into())

                .and_then(move |conn| {
                    let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                    conn.transaction::<(Category), FailureError, _>(move || {
                        categories_repo.create(new_category)
                    })
                })
        })
        .map_err(|e| e.context("Service Categories, create endpoint error occured.").into())
        )
    }

    /// Updates specific category
    fn update(&self, category_id: i32, payload: UpdateCategory) -> ServiceFuture<Category> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ControllerError::Connection(e.into()).into())

                .and_then(move |conn| {
                    let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                    categories_repo.update(category_id, payload)
                })
        })
        .map_err(|e| e.context("Service Categories, update endpoint error occured.").into())
        )
    }

    /// Returns category by ID
    fn get_all(&self) -> ServiceFuture<Category> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ControllerError::Connection(e.into()).into())

                .and_then(move |conn| {
                    let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                    categories_repo.get_all()
                })
        })
        .map_err(|e| e.context("Service Categories, get_all endpoint error occured.").into())
        )
    }

    /// Returns all category attributes belonging to category
    fn find_all_attributes(&self, category_id_arg: i32) -> ServiceFuture<Vec<Attribute>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ControllerError::Connection(e.into()).into())

                .and_then(move |conn| {
                    let category_attrs_repo = repo_factory.create_category_attrs_repo(&*conn, user_id);
                    let attrs_repo = repo_factory.create_attributes_repo(&*conn, user_id);
                    category_attrs_repo
                        .find_all_attributes(category_id_arg)
                        .or_else(|_| Ok(vec![]))
                        .and_then(|cat_attrs| {
                            cat_attrs
                                .into_iter()
                                .map(|cat_attr| attrs_repo
                                    .find(cat_attr.attr_id)
                                    .and_then(|attr| {
                                        if let Some(attr) = attr {
                                            Ok(attr)
                                        } else {
                                            error!("No such attribute with id : {}", cat_attr.attr_id);
                                            Err(ControllerError::NotFound.into())
                                        }
                                    })
                                )
                                .collect::<RepoResult<Vec<Attribute>>>()
                        })
                        
                })
        })
        .map_err(|e| e.context("Service Categories, find_all_attributes endpoint error occured.").into())
        )
    }

    /// Creates new category attribute
    fn add_attribute_to_category(&self, payload: NewCatAttr) -> ServiceFuture<()> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ControllerError::Connection(e.into()).into())

                .and_then(move |conn| {
                    let category_attrs_repo = repo_factory.create_category_attrs_repo(&*conn, user_id);
                    category_attrs_repo.create(payload)
                })
        })
        .map_err(|e| e.context("Service Categories, add_attribute_to_category endpoint error occured.").into())
        )
    }

    /// Deletes category attribute
    fn delete_attribute_from_category(&self, payload: OldCatAttr) -> ServiceFuture<()> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ControllerError::Connection(e.into()).into())

                .and_then(move |conn| {
                    let category_attrs_repo = repo_factory.create_category_attrs_repo(&*conn, user_id);
                    category_attrs_repo.delete(payload)
                })
        })
        .map_err(|e| e.context("Service Categories, delete_attribute_from_category endpoint error occured.").into())
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

    fn create_categories_service(user_id: Option<i32>) -> CategoriesServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        CategoriesServiceImpl {
            db_pool: db_pool,
            cpu_pool: cpu_pool,
            user_id: user_id,
            repo_factory: MOCK_REPO_FACTORY,
        }
    }

    pub fn create_new_categories(name: &str) -> NewCategory {
        NewCategory {
            name: serde_json::from_str(name).unwrap(),
            meta_field: None,
            parent_id: Some(1),
            level: 0,
        }
    }

    pub fn create_update_categories(name: &str) -> UpdateCategory {
        UpdateCategory {
            name: Some(serde_json::from_str(name).unwrap()),
            meta_field: None,
            parent_id: Some(1),
            level: Some(0),
        }
    }

    #[test]
    fn test_get_categories() {
        let mut core = Core::new().unwrap();
        let service = create_categories_service(Some(MOCK_USER_ID));
        let work = service.get(1);
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, 1);
    }

    #[test]
    fn test_create_categories() {
        let mut core = Core::new().unwrap();
        let service = create_categories_service(Some(MOCK_USER_ID));
        let new_categories = create_new_categories(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.create(new_categories);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, MOCK_BASE_PRODUCT_ID);
    }

    #[test]
    fn test_update() {
        let mut core = Core::new().unwrap();
        let service = create_categories_service(Some(MOCK_USER_ID));
        let new_categories = create_update_categories(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.update(1, new_categories);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
    }

}
