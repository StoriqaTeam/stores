//! Products Services, presents CRUD operations with product
use std::cell::RefCell;

use futures_cpupool::CpuPool;
use diesel::Connection;


use models::product::{NewProduct, Product, UpdateProduct};
use repos::products::{ProductsRepo, ProductsRepoImpl};
use super::types::ServiceFuture;
use super::error::Error;
use repos::types::DbPool;
use repos::acl::{Acl, ApplicationAcl, RolesCacheImpl, UnAuthanticatedACL};



pub trait ProductsService {
    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<Product>;
    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<Product>;
    /// Creates new product
    fn create(&self, payload: NewProduct) -> ServiceFuture<Product>;
    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Product>>;
    /// Updates specific product
    fn update(&self, product_id: i32, payload: UpdateProduct) -> ServiceFuture<Product>;
}

/// Products services, responsible for Product-related CRUD operations
pub struct ProductsServiceImpl {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub roles_cache: RolesCacheImpl,
    pub user_id: Option<i32>,
}

impl ProductsServiceImpl {
    pub fn new(
        db_pool: DbPool,
        cpu_pool: CpuPool,
        roles_cache: RolesCacheImpl,
        user_id: Option<i32>,
    ) -> Self {
        
        Self {
            db_pool,
            cpu_pool,
            roles_cache,
            user_id
        }
    }
}

impl ProductsService for ProductsServiceImpl {
    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(RefCell::new(UnAuthanticatedACL::new())) as Box<RefCell<Acl>>), |id| {
                        (Box::new(RefCell::new(ApplicationAcl::new(roles_cache.clone(), id))) as Box<RefCell<Acl>>)
                    });
                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    products_repo.find(product_id).map_err(Error::from)
                })
        }))
    }

    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(RefCell::new(UnAuthanticatedACL::new())) as Box<RefCell<Acl>>), |id| {
                        (Box::new(RefCell::new(ApplicationAcl::new(roles_cache.clone(), id))) as Box<RefCell<Acl>>)
                    });
                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    products_repo
                        .deactivate(product_id)
                        .map_err(|e| Error::from(e))
                })
        }))
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Product>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(RefCell::new(UnAuthanticatedACL::new())) as Box<RefCell<Acl>>), |id| {
                        (Box::new(RefCell::new(ApplicationAcl::new(roles_cache.clone(), id))) as Box<RefCell<Acl>>)
                    });
                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    products_repo.list(from, count).map_err(|e| Error::from(e))
                })
        }))
    }

    /// Creates new product
    fn create(&self, payload: NewProduct) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(RefCell::new(UnAuthanticatedACL::new())) as Box<RefCell<Acl>>), |id| {
                        (Box::new(RefCell::new(ApplicationAcl::new(roles_cache.clone(), id))) as Box<RefCell<Acl>>)
                    });
                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    conn.transaction::<Product, Error, _>(move || {
                        products_repo
                            .name_exists(payload.name.to_string())
                            .map(move |exists| (payload, exists))
                            .map_err(Error::from)
                            .and_then(|(payload, exists)| match exists {
                                false => Ok(payload),
                                true => Err(Error::Database("Product already exists".into())),
                            })
                            .and_then(move |new_product| {
                                products_repo
                                    .create(new_product)
                                    .map_err(|e| Error::from(e))
                            })
                            //rollback if error
                    })
                })
        }))
    }

    /// Updates specific product
    fn update(&self, product_id: i32, payload: UpdateProduct) -> ServiceFuture<Product> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id.clone();
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Database(format!("Connection error {}", e)))
                .and_then(move |conn| {
                    let acl = user_id.map_or((Box::new(RefCell::new(UnAuthanticatedACL::new())) as Box<RefCell<Acl>>), |id| {
                        (Box::new(RefCell::new(ApplicationAcl::new(roles_cache.clone(), id))) as Box<RefCell<Acl>>)
                    });
                    let products_repo = ProductsRepoImpl::new(&conn, acl);
                    products_repo
                        .find(product_id.clone())
                        .and_then(move |_user| products_repo.update(product_id, payload))
                        .map_err(|e| Error::from(e))
                })
                //rollback if error
        }))
    }
}
