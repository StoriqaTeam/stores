use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_static_resources::Currency;
use stq_types::{BaseProductId, ProductId, UserId};

use models::{BaseProductRaw, NewProduct, RawProduct, Store, UpdateProduct};
use repos::legacy_acl::*;
use schema::base_products::dsl as BaseProducts;
use schema::products::dsl::*;
use schema::stores::dsl as Stores;

use models::authorization::*;
use repos::acl;
use repos::types::{RepoAcl, RepoResult};

/// Products repository, responsible for handling products
pub struct ProductsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<RepoAcl<RawProduct>>,
}

pub trait ProductsRepo {
    /// Find specific product by ID
    fn find(&self, product_id: ProductId) -> RepoResult<Option<RawProduct>>;

    /// Find specific product by IDs
    fn find_many(&self, product_ids: Vec<ProductId>) -> RepoResult<Vec<RawProduct>>;

    /// Returns list of products, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> RepoResult<Vec<RawProduct>>;

    /// Returns list of products with base id
    fn find_with_base_id(&self, base_id: BaseProductId) -> RepoResult<Vec<RawProduct>>;

    /// Creates new product
    fn create(&self, payload: NewProduct) -> RepoResult<RawProduct>;

    /// Updates specific product
    fn update(&self, product_id: ProductId, payload: UpdateProduct) -> RepoResult<RawProduct>;

    /// Deactivates specific product
    fn deactivate(&self, product_id: ProductId) -> RepoResult<RawProduct>;

    /// Deactivates specific product
    fn deactivate_by_base_product(&self, base_product_id: BaseProductId) -> RepoResult<Vec<RawProduct>>;

    /// Update currency on all products with base_product_id
    fn update_currency(&self, currency: Currency, base_product_id: BaseProductId) -> RepoResult<usize>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ProductsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<RepoAcl<RawProduct>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ProductsRepo for ProductsRepoImpl<'a, T> {
    /// Find specific product by ID
    fn find(&self, product_id_arg: ProductId) -> RepoResult<Option<RawProduct>> {
        debug!("Find in products with id {}.", product_id_arg);
        let query = products.find(product_id_arg).filter(is_active.eq(true)).order(id);
        query
            .get_result(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|product: Option<RawProduct>| {
                if let Some(ref product) = product {
                    acl::check(&*self.acl, Resource::Products, Action::Read, self, Some(product))?;
                };
                Ok(product)
            })
            .map_err(|e: FailureError| e.context(format!("Find product with id: {} error occurred", product_id_arg)).into())
    }

    /// Find specific product by IDs
    fn find_many(&self, product_ids: Vec<ProductId>) -> RepoResult<Vec<RawProduct>> {
        debug!("Find in products {:?}.", product_ids);
        let query = products.filter(id.eq_any(product_ids.clone())).filter(is_active.eq(true)).order(id);

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|products_res: Vec<RawProduct>| {
                for product in &products_res {
                    acl::check(&*self.acl, Resource::Products, Action::Read, self, Some(&product))?;
                }
                Ok(products_res.clone())
            })
            .map_err(move |e: FailureError| e.context(format!("Find in products {:?} error occurred.", product_ids)).into())
    }

    /// Creates new product
    fn create(&self, payload: NewProduct) -> RepoResult<RawProduct> {
        debug!("Create products {:?}.", payload);
        let query_product = diesel::insert_into(products).values(&payload);
        query_product
            .get_result::<RawProduct>(self.db_conn)
            .map_err(From::from)
            .and_then(|prod| acl::check(&*self.acl, Resource::Products, Action::Create, self, Some(&prod)).and_then(|_| Ok(prod)))
            .map_err(|e: FailureError| e.context(format!("Create products {:?} error occurred.", payload)).into())
    }

    /// Returns list of products, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> RepoResult<Vec<RawProduct>> {
        debug!("Find in products from {} count {}.", from, count);
        let query = products
            .filter(is_active.eq(true))
            .filter(id.ge(from))
            .order(id)
            .limit(count.into());

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|products_res: Vec<RawProduct>| {
                for product in &products_res {
                    acl::check(&*self.acl, Resource::Products, Action::Read, self, Some(&product))?;
                }
                Ok(products_res.clone())
            })
            .map_err(|e: FailureError| {
                e.context(format!("Find in products from {} count {} error occurred.", from, count))
                    .into()
            })
    }

    /// Returns list of products with base id
    fn find_with_base_id(&self, base_id_arg: BaseProductId) -> RepoResult<Vec<RawProduct>> {
        debug!("Find in products with id {}.", base_id_arg);
        let query = products
            .filter(base_product_id.eq(base_id_arg))
            .filter(is_active.eq(true))
            .order_by(id);

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|products_res: Vec<RawProduct>| {
                for product in &products_res {
                    acl::check(&*self.acl, Resource::Products, Action::Read, self, Some(&product))?;
                }
                Ok(products_res.clone())
            })
            .map_err(|e: FailureError| {
                e.context(format!("Find in products with id {} error occurred.", base_id_arg))
                    .into()
            })
    }

    /// Updates specific product
    fn update(&self, product_id_arg: ProductId, payload: UpdateProduct) -> RepoResult<RawProduct> {
        debug!("Updating product with id {} and payload {:?}.", product_id_arg, payload);
        self.execute_query(products.find(product_id_arg))
            .and_then(|product: RawProduct| acl::check(&*self.acl, Resource::Products, Action::Update, self, Some(&product)))
            .and_then(|_| {
                let filter = products.filter(id.eq(product_id_arg)).filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<RawProduct>(self.db_conn).map_err(From::from)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Updating product with id {} and payload {:?} error occurred.",
                    product_id_arg, payload
                ))
                .into()
            })
    }

    /// Deactivates specific product
    fn deactivate(&self, product_id_arg: ProductId) -> RepoResult<RawProduct> {
        debug!("Deactivate product with id {}.", product_id_arg);
        self.execute_query(products.find(product_id_arg))
            .and_then(|product: RawProduct| acl::check(&*self.acl, Resource::Products, Action::Delete, self, Some(&product)))
            .and_then(|_| {
                let filter = products.filter(id.eq(product_id_arg)).filter(is_active.eq(true));
                let query = diesel::update(filter).set(is_active.eq(false));
                self.execute_query(query)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Deactivate product with id {} error occurred.", product_id_arg))
                    .into()
            })
    }

    /// Deactivates specific product
    fn deactivate_by_base_product(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<RawProduct>> {
        debug!("Deactivate products by base product id {}.", base_product_id_arg);

        let query = products.filter(base_product_id.eq(base_product_id_arg));

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|results: Vec<RawProduct>| {
                for product in &results {
                    acl::check(&*self.acl, Resource::Products, Action::Delete, self, Some(product))?;
                }

                Ok(results)
            })
            .and_then(|_| {
                let filtered = products.filter(base_product_id.eq(base_product_id_arg)).filter(is_active.eq(true));
                let query_update = diesel::update(filtered).set(is_active.eq(false));
                query_update.get_results(self.db_conn).map_err(From::from)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Deactivate products by base_product_id {} failed", base_product_id_arg))
                    .into()
            })
    }

    /// Update currency on all product with base_product_id
    fn update_currency(&self, currency_arg: Currency, base_product_id_arg: BaseProductId) -> RepoResult<usize> {
        debug!(
            "Setting currency {} on all product with base_product_id {}.",
            currency_arg, base_product_id_arg
        );

        let query = products.filter(base_product_id.eq(base_product_id_arg)).filter(is_active.eq(true));

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|products_res: Vec<RawProduct>| {
                for product in &products_res {
                    acl::check(&*self.acl, Resource::Products, Action::Read, self, Some(&product))?;
                }
                Ok(())
            })
            .and_then(|_| {
                diesel::update(products)
                    .filter(base_product_id.eq(base_product_id_arg))
                    .filter(is_active.eq(true))
                    .set(currency.eq(currency_arg))
                    .execute(self.db_conn)
                    .map_err(From::from)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Setting currency {} on all product with base_product_id {} error occurred.",
                    currency_arg, base_product_id_arg
                ))
                .into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, RawProduct>
    for ProductsRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id: UserId, scope: &Scope, obj: Option<&RawProduct>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(product) = obj {
                    BaseProducts::base_products
                        .filter(BaseProducts::id.eq(product.base_product_id))
                        .inner_join(Stores::stores)
                        .get_result::<(BaseProductRaw, Store)>(self.db_conn)
                        .map(|(_, s)| s.user_id == user_id)
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}
