use std::collections::{BTreeMap, HashMap};

use diesel;
use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::dsl::exists;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use failure::Fail;
use failure::Error as FailureError;

use stq_acl::*;

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::base_product::base_products::dsl::*;
use models::store::stores::dsl as Stores;
use models::{BaseProduct, BaseProductWithVariants, ElasticProduct, NewBaseProduct, Product, Store, UpdateBaseProduct,
             UpdateBaseProductViews};

/// BaseProducts repository, responsible for handling base_products
pub struct BaseProductsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, BaseProduct>>,
}

pub trait BaseProductsRepo {
    /// Find specific base_product by ID
    fn find(&self, base_product_id: i32) -> RepoResult<Option<BaseProduct>>;

    /// Returns list of base_products, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> RepoResult<Vec<BaseProduct>>;

    /// Returns list of base_products by store id and exclude base_product_id_arg, limited by 10
    fn get_products_of_the_store(
        &self,
        store_id: i32,
        skip_base_product_id: Option<i32>,
        from: i32,
        count: i32,
    ) -> RepoResult<Vec<BaseProduct>>;

    /// Counts products by store id
    fn count_with_store_id(&self, store_id: i32) -> RepoResult<i32>;

    /// Creates new base_product
    fn create(&self, payload: NewBaseProduct) -> RepoResult<BaseProduct>;

    /// Updates specific base_product
    fn update(&self, base_product_id: i32, payload: UpdateBaseProduct) -> RepoResult<BaseProduct>;

    /// Update views on specific base_product
    fn update_views(&self, base_product_id: i32) -> RepoResult<Option<BaseProduct>>;

    /// Deactivates specific base_product
    fn deactivate(&self, base_product_id: i32) -> RepoResult<BaseProduct>;

    /// Checks that slug already exists
    fn slug_exists(&self, slug_arg: String) -> RepoResult<bool>;

    /// Convert data from elastic to PG models
    fn convert_from_elastic(&self, el_products: Vec<ElasticProduct>) -> RepoResult<Vec<BaseProductWithVariants>>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> BaseProductsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, BaseProduct>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(|e| e.into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> BaseProductsRepo
    for BaseProductsRepoImpl<'a, T>
{
    /// Find specific base_product by ID
    fn find(&self, base_product_id_arg: i32) -> RepoResult<Option<BaseProduct>> {
        debug!("Find in base products with id {}.", base_product_id_arg);
        let query = base_products.find(base_product_id_arg);
        query
            .get_result(self.db_conn)
            .optional()
            .map_err(|e| e.into())
            .and_then(|base_product: Option<BaseProduct>| {
                if let Some(ref base_product) = base_product {
                    acl::check(&*self.acl, &Resource::BaseProducts, &Action::Read, self, Some(base_product))?;
                };
                Ok(base_product)
            })
            .map_err(|e: FailureError| e.context(format!("Find base product by id: {} error occured", base_product_id_arg)).into())
        
    }

    /// Counts products by store id
    fn count_with_store_id(&self, store_id_arg: i32) -> RepoResult<i32> {
        debug!("Counts products with store id {}.", store_id_arg);
        let query = base_products.filter(is_active.eq(true)).filter(store_id.eq(store_id_arg)).count();

        query
            .get_result(self.db_conn)
            .optional()
            .map(|count: Option<i64>| {
                if let Some(count) = count {
                    count as i32
                } else {
                    0
                }
            })
            .map_err(|e| e.context(format!("Counts products by store id: {} error occured", store_id_arg)).into())
    }

    /// Creates new base_product
    fn create(&self, payload: NewBaseProduct) -> RepoResult<BaseProduct> {
        debug!("Create base product {:?}.", payload);
        let query_base_product = diesel::insert_into(base_products).values(&payload);
        query_base_product
            .get_result::<BaseProduct>(self.db_conn)
            .map_err(|e| e.into())
            .and_then(|base_prod| {
                acl::check(&*self.acl, &Resource::BaseProducts, &Action::Create, self, Some(&base_prod)).and_then(|_| Ok(base_prod))
            })
            .map_err(|e: FailureError| e.context(format!("Creates new base_product {:?} error occured", payload)).into())
    }

    /// Returns list of base_products, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> RepoResult<Vec<BaseProduct>> {
        debug!("Find in base products with ids from {} count {}.", from, count);
        let query = base_products
            .filter(is_active.eq(true))
            .filter(id.ge(from))
            .order(id)
            .limit(count.into());

        query
            .get_results(self.db_conn)
            .map_err(|e| e.into())
            .and_then(|base_products_res: Vec<BaseProduct>| {
                for base_product in &base_products_res {
                    acl::check(&*self.acl, &Resource::BaseProducts, &Action::Read, self, Some(&base_product))?;
                }
                Ok(base_products_res)
            })
            .map_err(|e: FailureError| e.context(format!("Find in base products with ids from {} count {} error occured", from, count)).into())
    }

    /// Returns list of base_products by store id and skip skip_base_product_id, limited by from and count
    fn get_products_of_the_store(
        &self,
        store_id_arg: i32,
        skip_base_product_id: Option<i32>,
        from: i32,
        count: i32,
    ) -> RepoResult<Vec<BaseProduct>> {
        debug!(
            "Find in base products with store id {} skip {:?} from {} count {}.",
            store_id_arg, skip_base_product_id, from, count
        );
        let query = if let Some(skip_base_product_id) = skip_base_product_id {
            base_products
                .filter(is_active.eq(true))
                .filter(store_id.eq(store_id_arg))
                .filter(id.ne(skip_base_product_id))
                .filter(id.ge(from))
                .order(id)
                .limit(count.into())
                .into_boxed()
        } else {
            base_products
                .filter(is_active.eq(true))
                .filter(store_id.eq(store_id_arg))
                .filter(id.ge(from))
                .order(id)
                .limit(count.into())
                .into_boxed()
        };

        query
            .get_results(self.db_conn)
            .map_err(|e| e.into())
            .and_then(|base_products_res: Vec<BaseProduct>| {
                for base_product in &base_products_res {
                    acl::check(&*self.acl, &Resource::BaseProducts, &Action::Read, self, Some(&base_product))?;
                }
                Ok(base_products_res)
            })
            .map_err(|e: FailureError| e.context(format!(
                "Find in base products with store id {} skip {:?} from {} count {}.",
                store_id_arg, skip_base_product_id, from, count
            )).into())
    }

    /// Updates specific base_product
    fn update(&self, base_product_id_arg: i32, payload: UpdateBaseProduct) -> RepoResult<BaseProduct> {
        debug!("Updating base product with id {} and payload {:?}.", base_product_id_arg, payload);
        self.execute_query(base_products.find(base_product_id_arg))
            .and_then(|base_product: BaseProduct| {
                acl::check(&*self.acl, &Resource::BaseProducts, &Action::Update, self, Some(&base_product))
            })
            .and_then(|_| {
                let filter = base_products.filter(id.eq(base_product_id_arg)).filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<BaseProduct>(self.db_conn).map_err(|e| e.into())
            })
            .map_err(|e: FailureError| e.context(format!("Updating base product with id {} and payload {:?} failed.", base_product_id_arg, payload)).into())
    }

    /// Update views on specific base_product
    fn update_views(&self, base_product_id_arg: i32) -> RepoResult<Option<BaseProduct>> {
        debug!("Updating views of base product with id {}.", base_product_id_arg);
        let query = base_products.find(base_product_id_arg).filter(is_active.eq(true));
        query
            .get_result(self.db_conn)
            .optional()
            .map_err(|e| e.into())
            .and_then(|base_product: Option<BaseProduct>| {
                if let Some(base_product) = base_product {
                    let filter = base_products.filter(id.eq(base_product_id_arg)).filter(is_active.eq(true));
                    let payload: UpdateBaseProductViews = base_product.into();
                    let query = diesel::update(filter).set(&payload);
                    query.get_result::<BaseProduct>(self.db_conn).optional().map_err(|e| e.into())
                } else {
                    Ok(None)
                }
            })
            .map_err(|e: FailureError| e.context(format!("Updating views of base product with id {} failed", base_product_id_arg)).into())
    }

    /// Deactivates specific base_product
    fn deactivate(&self, base_product_id_arg: i32) -> RepoResult<BaseProduct> {
        debug!("Deactivate base product with id {}.", base_product_id_arg);
        self.execute_query(base_products.find(base_product_id_arg))
            .and_then(|base_product: BaseProduct| {
                acl::check(&*self.acl, &Resource::BaseProducts, &Action::Delete, self, Some(&base_product))
            })
            .and_then(|_| {
                let filter = base_products.filter(id.eq(base_product_id_arg)).filter(is_active.eq(true));
                let query = diesel::update(filter).set(is_active.eq(false));
                self.execute_query(query)
            })
            .map_err(|e: FailureError| e.context(format!("Deactivate base product with id {} failed", base_product_id_arg)).into())
    }

    /// Checks that slug already exists
    fn slug_exists(&self, slug_arg: String) -> RepoResult<bool> {
        debug!("Check if store slug {} exists.", slug_arg);
        let query = diesel::select(exists(base_products.filter(slug.eq(slug_arg.clone()))));
        query
            .get_result(self.db_conn)
            .map_err(|e| e.into())
            .and_then(|exists| acl::check(&*self.acl, &Resource::BaseProducts, &Action::Read, self, None).and_then(|_| Ok(exists)))
            .map_err(move |e: FailureError| e.context(format!("Check if store slug {} exists failed", slug_arg)).into())
    }

    /// Convert data from elastic to PG models
    fn convert_from_elastic(&self, el_products: Vec<ElasticProduct>) -> RepoResult<Vec<BaseProductWithVariants>> {
        acl::check(&*self.acl, &Resource::BaseProducts, &Action::Read, self, None).and_then(|_|{
            let base_products_ids = el_products.iter().map(|b| b.id).collect::<Vec<i32>>();
            debug!(
                "Converting data from elastic to PG models for base_products with ids: {:?}",
                base_products_ids
            );
            let hashed_ids = base_products_ids
                .clone()
                .into_iter()
                .enumerate()
                .map(|(n, id_arg)| (id_arg, n))
                .collect::<HashMap<_, _>>();

            let base_products_query = base_products.filter(id.eq_any(base_products_ids));
            let base_products_list: Vec<BaseProduct> = 
                base_products_query
                    .get_results(self.db_conn)?;

            // sorting in elastic order
            let base_products_list = base_products_list
                .into_iter()
                .fold(BTreeMap::<usize, BaseProduct>::new(), |mut tree_map, bp| {
                    let n = hashed_ids[&bp.id];
                    tree_map.insert(n, bp);
                    tree_map
                })
                .into_iter()
                .map(|(_, base_product)| base_product)
                .collect::<Vec<BaseProduct>>();

            let variants_ids = el_products
                .iter()
                .flat_map(|p| {
                    if let Some(matched_ids) = p.clone().matched_variants_ids {
                        matched_ids
                    } else {
                        p.variants.iter().map(|variant| variant.prod_id).collect()
                    }
                })
                .collect::<Vec<i32>>();

            let variants = Product::belonging_to(&base_products_list)
                .get_results(self.db_conn)?
                .into_iter()
                .filter(|prod: &Product| variants_ids.iter().any(|id_arg| *id_arg == prod.id))
                .grouped_by(&base_products_list);

            Ok(base_products_list
                .into_iter()
                .zip(variants)
                .map(|(base, vars)| BaseProductWithVariants::new(base, vars))
                .collect())
        })
        .map_err(|e: FailureError| e.context(format!("Convert data from elastic to PG models failed")).into())
        
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, BaseProduct>
    for BaseProductsRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id: i32, scope: &Scope, obj: Option<&BaseProduct>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(base_prod) = obj {
                    Stores::stores
                        .find(base_prod.store_id)
                        .get_result::<Store>(self.db_conn)
                        .and_then(|store: Store| Ok(store.user_id == user_id))
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}
