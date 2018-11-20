use std::collections::{BTreeMap, HashMap};

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::dsl::exists;
use diesel::dsl::sql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::sql_types::{Bool, VarChar};
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;

use stq_static_resources::ModerationStatus;
use stq_types::{BaseProductId, CategoryId, ProductId, StoreId, UserId};

use models::authorization::*;
use models::{
    Attribute, BaseProduct, BaseProductWithVariants, CatalogWithAttributes, Direction, ElasticProduct, ModeratorBaseProductSearchResults,
    ModeratorBaseProductSearchTerms, MostDiscountProducts, MostViewedProducts, NewBaseProduct, Ordering, PaginationParams, ProdAttr,
    Product, ProductWithAttributes, RawProduct, Store, UpdateBaseProduct, Visibility,
};

use errors;
use repos::acl;
use repos::legacy_acl::*;
use repos::types::{RepoAcl, RepoResult};
use schema::attributes::dsl as DslAttributes;
use schema::base_products::dsl::*;
use schema::prod_attr_values::dsl as DslProdAttr;
use schema::products::dsl as Products;
use schema::stores::dsl as Stores;

/// BaseProducts repository, responsible for handling base_products
pub struct BaseProductsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<RepoAcl<BaseProduct>>,
}

#[derive(Debug, Default)]
pub struct BaseProductsSearchTerms {
    pub is_active: Option<bool>,
    pub category_id: Option<CategoryId>,
    pub category_ids: Option<Vec<CategoryId>>,
}

pub trait BaseProductsRepo {
    /// Get base_product count
    fn count(&self, visibility: Visibility) -> RepoResult<i64>;

    /// Find specific base_product by ID
    fn find(&self, base_product_id: BaseProductId, visibility: Visibility) -> RepoResult<Option<BaseProduct>>;

    /// Find base_products by ids
    fn find_many(&self, base_product_ids: Vec<BaseProductId>) -> RepoResult<Vec<BaseProduct>>;

    /// Search many products by search terms
    fn search(&self, search_terms: BaseProductsSearchTerms) -> RepoResult<Vec<BaseProduct>>;

    /// Returns list of base_products, limited by `from` and `count` parameters
    fn list(&self, from: BaseProductId, count: i32, visibility: Visibility) -> RepoResult<Vec<BaseProduct>>;

    /// Returns most viewed list of base_products, limited by `from` and `offset` parameters
    fn most_viewed(&self, search_product: MostViewedProducts, count: i32, offset: i32) -> RepoResult<Vec<BaseProductWithVariants>>;

    /// Returns most discount list of base_products, limited by `from` and `offset` parameters
    fn most_discount(&self, search_product: MostDiscountProducts, count: i32, offset: i32) -> RepoResult<Vec<BaseProductWithVariants>>;

    /// Returns list of base_products by store id and exclude base_product_id_arg, limited by 10
    fn get_products_of_the_store(
        &self,
        store_id: StoreId,
        skip_base_product_id: Option<BaseProductId>,
        from: BaseProductId,
        count: i32,
        visibility: Visibility,
    ) -> RepoResult<Vec<BaseProduct>>;

    /// Counts products by store id
    fn count_with_store_id(&self, store_id: StoreId, visibility: Visibility) -> RepoResult<i32>;

    /// Creates new base_product
    fn create(&self, payload: NewBaseProduct) -> RepoResult<BaseProduct>;

    /// Updates specific base_product
    fn update(&self, base_product_id: BaseProductId, payload: UpdateBaseProduct) -> RepoResult<BaseProduct>;

    /// Update views on specific base_product
    fn update_views(&self, base_product_id: BaseProductId) -> RepoResult<Option<BaseProduct>>;

    /// Deactivates specific base_product
    fn deactivate(&self, base_product_id: BaseProductId) -> RepoResult<BaseProduct>;

    /// Deactivates base_products by store_id
    fn deactivate_by_store(&self, store_id: StoreId) -> RepoResult<Vec<BaseProduct>>;

    /// Checks that slug already exists
    fn slug_exists(&self, slug_arg: String) -> RepoResult<bool>;

    /// Convert data from elastic to PG models
    fn convert_from_elastic(&self, el_products: Vec<ElasticProduct>) -> RepoResult<Vec<BaseProductWithVariants>>;

    /// Search base product limited by pagination parameters
    fn moderator_search(
        &self,
        pagination_params: PaginationParams<BaseProductId>,
        term: ModeratorBaseProductSearchTerms,
    ) -> RepoResult<ModeratorBaseProductSearchResults>;

    /// Set moderation status for base_product_ids
    fn set_moderation_status(&self, base_product_ids: Vec<BaseProductId>, status: ModerationStatus) -> RepoResult<Vec<BaseProduct>>;

    /// Set moderation status for base_product_ids from store manager
    fn update_moderation_statuses(&self, base_product_ids: Vec<BaseProductId>, status: ModerationStatus) -> RepoResult<Vec<BaseProduct>>;

    /// Set moderation status for base_product_id from store manager
    fn update_moderation_status(&self, base_product_id: BaseProductId, status: ModerationStatus) -> RepoResult<BaseProduct>;

    /// Getting all base products with variants
    fn get_all_catalog(&self) -> RepoResult<Vec<CatalogWithAttributes>>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> BaseProductsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<RepoAcl<BaseProduct>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> BaseProductsRepo
    for BaseProductsRepoImpl<'a, T>
{
    /// Get base_product count
    fn count(&self, visibility: Visibility) -> RepoResult<i64> {
        debug!("Count base products with visibility = {:?}", visibility);

        let query = match visibility {
            Visibility::Active => base_products.filter(is_active.eq(true)).into_boxed(),
            Visibility::Published => base_products
                .filter(is_active.eq(true).and(status.eq(ModerationStatus::Published)))
                .into_boxed(),
        };

        acl::check(&*self.acl, Resource::BaseProducts, Action::Read, self, None)
            .and_then(|_| query.count().get_result(self.db_conn).map_err(From::from))
            .map_err(|e| FailureError::from(e).context("Count base products error occurred").into())
    }

    /// Find specific base_product by ID
    fn find(&self, base_product_id_arg: BaseProductId, visibility: Visibility) -> RepoResult<Option<BaseProduct>> {
        debug!(
            "Find in base products with id {}, visibility = {:?}",
            base_product_id_arg, visibility
        );

        let query = match visibility {
            Visibility::Active => base_products.filter(is_active.eq(true)).into_boxed(),
            Visibility::Published => base_products
                .filter(is_active.eq(true).and(status.eq(ModerationStatus::Published)))
                .into_boxed(),
        };

        query
            .filter(id.eq(base_product_id_arg))
            .first(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|base_product: Option<BaseProduct>| {
                if let Some(ref base_product) = base_product {
                    acl::check_with_rule(
                        &*self.acl,
                        Resource::BaseProducts,
                        Action::Read,
                        self,
                        Rule::ModerationStatus(base_product.status),
                        Some(base_product),
                    )?;
                };

                Ok(base_product)
            }).map_err(|e: FailureError| {
                e.context(format!("Find base product by id: {} error occurred", base_product_id_arg))
                    .into()
            })
    }

    /// Find base_products by ids
    fn find_many(&self, base_product_ids: Vec<BaseProductId>) -> RepoResult<Vec<BaseProduct>> {
        debug!("Find many base products.");
        let query = base_products.filter(id.eq_any(base_product_ids));

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|results: Vec<BaseProduct>| {
                for base_product in results.iter() {
                    acl::check_with_rule(
                        &*self.acl,
                        Resource::BaseProducts,
                        Action::Read,
                        self,
                        Rule::ModerationStatus(base_product.status),
                        Some(base_product),
                    )?;
                }
                Ok(results)
            }).map_err(|e: FailureError| e.context(format!("Find many base products error occurred")).into())
    }

    /// Search many products by search terms
    fn search(&self, search_terms: BaseProductsSearchTerms) -> RepoResult<Vec<BaseProduct>> {
        debug!("Find many base products with search terms.");

        type BoxedExpr = Box<BoxableExpression<base_products, Pg, SqlType = Bool>>;

        let mut query: BoxedExpr = Box::new(id.eq(id));

        if let Some(is_active_filter) = search_terms.is_active {
            query = Box::new(query.and(is_active.eq(is_active_filter)));
        }

        if let Some(category_id_filter) = search_terms.category_id {
            query = Box::new(query.and(category_id.eq(category_id_filter)));
        }

        if let Some(category_ids_filter) = search_terms.category_ids {
            query = Box::new(query.and(category_id.eq_any(category_ids_filter)));
        }

        base_products
            .filter(query)
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|results: Vec<BaseProduct>| {
                for result in results.iter() {
                    acl::check(&*self.acl, Resource::BaseProducts, Action::Read, self, Some(result))?;
                }
                Ok(results)
            }).map_err(|e: FailureError| e.context(format!("Find many base products by search terms error occurred")).into())
    }

    /// Counts products by store id
    fn count_with_store_id(&self, store_id_arg: StoreId, visibility: Visibility) -> RepoResult<i32> {
        debug!("Counts products with store id {}, visibility = {:?}", store_id_arg, visibility);

        let query = match visibility {
            Visibility::Active => base_products.filter(is_active.eq(true)).into_boxed(),
            Visibility::Published => base_products
                .filter(is_active.eq(true).and(status.eq(ModerationStatus::Published)))
                .into_boxed(),
        };

        query
            .filter(store_id.eq(store_id_arg))
            .count()
            .get_result(self.db_conn)
            .optional()
            .map(|count: Option<i64>| if let Some(count) = count { count as i32 } else { 0 })
            .map_err(|e| {
                e.context(format!("Counts products by store id: {} error occurred", store_id_arg))
                    .into()
            })
    }

    /// Creates new base_product
    fn create(&self, payload: NewBaseProduct) -> RepoResult<BaseProduct> {
        debug!("Create base product {:?}.", payload);
        let query_base_product = diesel::insert_into(base_products).values(&payload);
        query_base_product
            .get_result::<BaseProduct>(self.db_conn)
            .map_err(From::from)
            .and_then(|base_prod| {
                acl::check(&*self.acl, Resource::BaseProducts, Action::Create, self, Some(&base_prod)).and_then(|_| Ok(base_prod))
            }).map_err(|e: FailureError| e.context(format!("Creates new base_product {:?} error occurred", payload)).into())
    }

    /// Returns list of base_products, limited by `from` and `count` parameters
    fn list(&self, from: BaseProductId, count: i32, visibility: Visibility) -> RepoResult<Vec<BaseProduct>> {
        debug!(
            "Find in base products with ids from {} count {} with visibility = {:?}",
            from, count, visibility
        );

        let query = match visibility {
            Visibility::Active => base_products.filter(is_active.eq(true)).into_boxed(),
            Visibility::Published => base_products
                .filter(is_active.eq(true).and(status.eq(ModerationStatus::Published)))
                .into_boxed(),
        };

        query
            .filter(id.ge(from))
            .order(id)
            .limit(count.into())
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|base_products_res: Vec<BaseProduct>| {
                for base_product in &base_products_res {
                    acl::check_with_rule(
                        &*self.acl,
                        Resource::BaseProducts,
                        Action::Read,
                        self,
                        Rule::ModerationStatus(base_product.status),
                        Some(base_product),
                    )?;
                }
                Ok(base_products_res)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "Find in base products with ids from {} count {} error occurred",
                    from, count
                )).into()
            })
    }

    /// Returns list of base_products by store id and skip skip_base_product_id, limited by from and count
    fn get_products_of_the_store(
        &self,
        store_id_arg: StoreId,
        skip_base_product_id: Option<BaseProductId>,
        from: BaseProductId,
        count: i32,
        visibility: Visibility,
    ) -> RepoResult<Vec<BaseProduct>> {
        debug!(
            "Find in base products with store id = {}, skip = {:?}, from id = {}, count = {}, visibility = {:?}",
            store_id_arg, skip_base_product_id, from, count, visibility
        );

        let mut query = match visibility {
            Visibility::Active => base_products.filter(is_active.eq(true)).into_boxed(),
            Visibility::Published => base_products
                .filter(is_active.eq(true).and(status.eq(ModerationStatus::Published)))
                .into_boxed(),
        };

        query = query.filter(store_id.eq(store_id_arg));

        if let Some(skip_base_product_id) = skip_base_product_id {
            query = query.filter(id.ne(skip_base_product_id));
        }

        query = query.filter(id.ge(from)).order(id).limit(count.into());

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|base_products_res: Vec<BaseProduct>| {
                for base_product in &base_products_res {
                    acl::check_with_rule(
                        &*self.acl,
                        Resource::BaseProducts,
                        Action::Read,
                        self,
                        Rule::ModerationStatus(base_product.status),
                        Some(base_product),
                    )?;
                }
                Ok(base_products_res)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "Find in base products with store id {} skip {:?} from {} count {}.",
                    store_id_arg, skip_base_product_id, from, count
                )).into()
            })
    }

    /// Updates specific base_product
    fn update(&self, base_product_id_arg: BaseProductId, payload: UpdateBaseProduct) -> RepoResult<BaseProduct> {
        debug!("Updating base product with id {} and payload {:?}.", base_product_id_arg, payload);
        self.execute_query(base_products.find(base_product_id_arg))
            .and_then(|base_product: BaseProduct| {
                acl::check_with_rule(
                    &*self.acl,
                    Resource::BaseProducts,
                    Action::Update,
                    self,
                    Rule::ModerationStatus(base_product.status),
                    Some(&base_product),
                )
            }).and_then(|_| {
                let filter = base_products.filter(id.eq(base_product_id_arg)).filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<BaseProduct>(self.db_conn).map_err(From::from)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "Updating base product with id {} and payload {:?} failed.",
                    base_product_id_arg, payload
                )).into()
            })
    }

    /// Update views on specific base_product
    fn update_views(&self, base_product_id_arg: BaseProductId) -> RepoResult<Option<BaseProduct>> {
        debug!("Updating views of base product with id {}.", base_product_id_arg);
        let filter = base_products
            .filter(id.eq(base_product_id_arg))
            .filter(is_active.eq(true))
            .filter(status.eq(ModerationStatus::Published));
        let query = diesel::update(filter).set(views.eq(views + 1));
        query
            .get_result::<BaseProduct>(self.db_conn)
            .optional()
            .map_err(From::from)
            .map_err(|e: FailureError| {
                e.context(format!("Updating views of base product with id {} failed", base_product_id_arg))
                    .into()
            })
    }

    /// Deactivates specific base_product
    fn deactivate(&self, base_product_id_arg: BaseProductId) -> RepoResult<BaseProduct> {
        debug!("Deactivate base product with id {}.", base_product_id_arg);
        self.execute_query(base_products.find(base_product_id_arg))
            .and_then(|base_product: BaseProduct| acl::check(&*self.acl, Resource::BaseProducts, Action::Delete, self, Some(&base_product)))
            .and_then(|_| {
                let filter = base_products.filter(id.eq(base_product_id_arg)).filter(is_active.eq(true));
                let query = diesel::update(filter).set(is_active.eq(false));
                self.execute_query(query)
            }).map_err(|e: FailureError| {
                e.context(format!("Deactivate base product with id {} failed", base_product_id_arg))
                    .into()
            })
    }

    /// Deactivates base_products by store_id
    fn deactivate_by_store(&self, store_id_arg: StoreId) -> RepoResult<Vec<BaseProduct>> {
        debug!("Deactivate base products by store id {}.", store_id_arg);

        let query = base_products.filter(store_id.eq(store_id_arg));

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|results: Vec<BaseProduct>| {
                for base_product in &results {
                    acl::check(&*self.acl, Resource::BaseProducts, Action::Delete, self, Some(base_product))?;
                }

                Ok(results)
            }).and_then(|_| {
                let filtered = base_products.filter(store_id.eq(store_id_arg)).filter(is_active.eq(true));
                let query_update = diesel::update(filtered).set(is_active.eq(false));
                query_update.get_results(self.db_conn).map_err(From::from)
            }).map_err(|e: FailureError| {
                e.context(format!("Deactivate base products by store_id {} failed", store_id_arg))
                    .into()
            })
    }

    /// Checks that slug already exists
    fn slug_exists(&self, slug_arg: String) -> RepoResult<bool> {
        debug!("Check if store slug {} exists.", slug_arg);
        let query = diesel::select(exists(base_products.filter(slug.eq(slug_arg.clone()))));
        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .map_err(move |e: FailureError| e.context(format!("Check if store slug {} exists failed", slug_arg)).into())
    }

    /// Convert data from elastic to PG models
    fn convert_from_elastic(&self, el_products: Vec<ElasticProduct>) -> RepoResult<Vec<BaseProductWithVariants>> {
        acl::check(&*self.acl, Resource::BaseProducts, Action::Read, self, None)
            .and_then(|_| {
                let base_products_ids = el_products.iter().map(|b| b.id).collect::<Vec<BaseProductId>>();
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
                let base_products_list: Vec<BaseProduct> = base_products_query.get_results(self.db_conn)?;

                // sorting in elastic order
                let base_products_list = base_products_list
                    .into_iter()
                    .fold(BTreeMap::<usize, BaseProduct>::new(), |mut tree_map, bp| {
                        let n = hashed_ids[&bp.id];
                        tree_map.insert(n, bp);
                        tree_map
                    }).into_iter()
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
                    }).collect::<Vec<ProductId>>();

                let variants = RawProduct::belonging_to(&base_products_list)
                    .get_results(self.db_conn)?
                    .into_iter()
                    .filter(|prod: &RawProduct| variants_ids.iter().any(|id_arg| *id_arg == prod.id))
                    .grouped_by(&base_products_list);

                Ok(base_products_list
                    .into_iter()
                    .zip(variants)
                    .map(|(base, vars)| {
                        let mut vars: Vec<Product> = vars.into_iter().map(Product::from).collect();
                        vars.sort_by(|a, b| a.product.id.cmp(&b.product.id));

                        BaseProductWithVariants::new(base, vars)
                    }).collect())
            }).map_err(|e: FailureError| e.context("Convert data from elastic to PG models failed").into())
    }

    /// Returns most viewed list of base_products, limited by `from` and `count` parameters
    fn most_viewed(&self, search_product: MostViewedProducts, count: i32, offset: i32) -> RepoResult<Vec<BaseProductWithVariants>> {
        acl::check(&*self.acl, Resource::BaseProducts, Action::Read, self, None)
            .and_then(|_| {
                debug!("Querying for most viewed base products.");

                let mut base_products_query = base_products
                    .filter(is_active.eq(true))
                    .filter(status.eq(ModerationStatus::Published))
                    .into_boxed();

                if let Some(options) = search_product.options {
                    if let Some(store_id_arg) = options.store_id {
                        base_products_query = base_products_query.filter(store_id.eq(store_id_arg));
                    }
                }

                base_products_query = base_products_query.order_by(views.desc()).offset(offset.into()).limit(count.into());

                let base_products_list: Vec<BaseProduct> = base_products_query.get_results(self.db_conn)?;
                for item in base_products_list.iter() {
                    acl::check_with_rule(
                        &*self.acl,
                        Resource::BaseProducts,
                        Action::Read,
                        self,
                        Rule::ModerationStatus(item.status),
                        Some(&item),
                    )?;
                }

                let variants = RawProduct::belonging_to(&base_products_list)
                    .get_results(self.db_conn)?
                    .into_iter()
                    .filter(|product: &RawProduct| product.is_active)
                    .grouped_by(&base_products_list);

                Ok(base_products_list
                    .into_iter()
                    .zip(variants)
                    .map(|(base, vars)| {
                        let vars = vars.into_iter().map(Product::from).collect();
                        BaseProductWithVariants::new(base, vars)
                    }).collect())
            }).map_err(|e: FailureError| e.context("Querying for most viewed base products failed").into())
    }

    /// Returns most discount list of base_products, limited by `from` and `count` parameters
    fn most_discount(&self, search_product: MostDiscountProducts, count: i32, offset: i32) -> RepoResult<Vec<BaseProductWithVariants>> {
        acl::check(&*self.acl, Resource::BaseProducts, Action::Read, self, None)
            .and_then(|_| {
                debug!("Querying for most discount products.");

                let products_query = Products::products
                    .filter(Products::is_active.eq(true))
                    .filter(Products::discount.is_not_null())
                    .order_by(Products::discount.desc())
                    .offset(offset.into())
                    .limit(count.into());

                let variants = products_query.get_results::<RawProduct>(self.db_conn)?;

                let base_products_ids = variants.iter().map(|p| p.base_product_id).collect::<Vec<BaseProductId>>();

                let hashed_ids = base_products_ids
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(n, id_arg)| (id_arg, n))
                    .collect::<HashMap<_, _>>();

                let mut base_products_query = base_products
                    .filter(id.eq_any(base_products_ids))
                    .filter(status.eq(ModerationStatus::Published))
                    .into_boxed();

                if let Some(options) = search_product.options {
                    if let Some(store_id_arg) = options.store_id {
                        base_products_query = base_products_query.filter(store_id.eq(store_id_arg));
                    }
                }

                let base_products_list: Vec<BaseProduct> = base_products_query.get_results(self.db_conn)?;
                for item in base_products_list.iter() {
                    acl::check_with_rule(
                        &*self.acl,
                        Resource::BaseProducts,
                        Action::Read,
                        self,
                        Rule::ModerationStatus(item.status),
                        Some(&item),
                    )?;
                }

                // sorting in products order
                let base_products_list = base_products_list
                    .into_iter()
                    .fold(BTreeMap::<usize, BaseProduct>::new(), |mut tree_map, bp| {
                        let n = hashed_ids[&bp.id];
                        tree_map.insert(n, bp);
                        tree_map
                    }).into_iter()
                    .map(|(_, base_product)| base_product)
                    .collect::<Vec<BaseProduct>>();

                Ok(base_products_list
                    .into_iter()
                    .zip(variants)
                    .map(|(base, var)| BaseProductWithVariants::new(base, vec![Product::from(var)]))
                    .collect())
            }).map_err(|e: FailureError| e.context("Querying for most discount base products failed").into())
    }

    /// Search base product limited by pagination parameters
    fn moderator_search(
        &self,
        pagination_params: PaginationParams<BaseProductId>,
        term: ModeratorBaseProductSearchTerms,
    ) -> RepoResult<ModeratorBaseProductSearchResults> {
        let PaginationParams {
            direction,
            limit,
            ordering,
            skip,
            start,
        } = pagination_params;

        let total_count_query = base_products
            .filter(is_active.eq(true).and(by_moderator_search_terms(&term)))
            .count();

        let mut query = base_products.filter(is_active.eq(true)).into_boxed();

        if let Some(from_id) = start {
            query = match direction {
                Direction::Forward => query.filter(id.gt(from_id)),
                Direction::Reverse => query.filter(id.lt(from_id)),
            };
        }

        if skip > 0 {
            query = query.offset(skip);
        }

        if limit > 0 {
            query = query.limit(limit);
        }

        query = query.filter(by_moderator_search_terms(&term));

        query = match ordering {
            Ordering::Ascending => query.order(id.asc()),
            Ordering::Descending => query.order(id.desc()),
        };

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|base_products_res: Vec<BaseProduct>| {
                for base_product in &base_products_res {
                    acl::check_with_rule(
                        &*self.acl,
                        Resource::BaseProducts,
                        Action::Read,
                        self,
                        Rule::ModerationStatus(base_product.status),
                        Some(base_product),
                    )?;
                }

                total_count_query
                    .get_result::<i64>(self.db_conn)
                    .map(move |total_count| ModeratorBaseProductSearchResults {
                        base_products: base_products_res,
                        total_count: total_count as u32,
                    }).map_err(From::from)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "moderator search for base_products error occurred (pagination params: {:?}, search terms: {:?})",
                    pagination_params, term
                )).into()
            })
    }

    /// Set moderation status for base_product
    fn set_moderation_status(&self, base_product_ids: Vec<BaseProductId>, status_arg: ModerationStatus) -> RepoResult<Vec<BaseProduct>> {
        let query = base_products.filter(id.eq_any(base_product_ids.clone()));

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|bs: Vec<BaseProduct>| {
                for base in &bs {
                    acl::check_with_rule(
                        &*self.acl,
                        Resource::BaseProducts,
                        Action::Moderate,
                        self,
                        Rule::ModerationStatus(base.status),
                        Some(&base),
                    )?;
                }
                Ok(bs)
            }).and_then(|_| {
                let filter = base_products.filter(id.eq_any(base_product_ids.clone()));
                let query = diesel::update(filter).set(status.eq(status_arg));

                query.get_results(self.db_conn).map_err(From::from)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "Set moderation status for base_product {:?} error occurred",
                    base_product_ids
                )).into()
            })
    }

    /// Set moderation status for base_product_ids from store manager
    fn update_moderation_statuses(
        &self,
        base_product_ids: Vec<BaseProductId>,
        status_arg: ModerationStatus,
    ) -> RepoResult<Vec<BaseProduct>> {
        let query = base_products.filter(id.eq_any(base_product_ids.clone()));

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|bs: Vec<BaseProduct>| {
                for base in &bs {
                    acl::check_with_rule(
                        &*self.acl,
                        Resource::BaseProducts,
                        Action::Update,
                        self,
                        Rule::ModerationStatus(base.status),
                        Some(&base),
                    )?;
                }
                Ok(bs)
            }).and_then(|_| {
                let filter = base_products.filter(id.eq_any(base_product_ids.clone()));
                let query = diesel::update(filter).set(status.eq(status_arg));

                query.get_results(self.db_conn).map_err(From::from)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "Update moderation status for base_products {:?} error occurred",
                    base_product_ids
                )).into()
            })
    }

    /// Set moderation status for base_product_id from store manager
    fn update_moderation_status(&self, base_product_id_arg: BaseProductId, status_arg: ModerationStatus) -> RepoResult<BaseProduct> {
        let mut results = self.update_moderation_statuses(vec![base_product_id_arg], status_arg)?;

        if let Some(base_product) = results.pop() {
            Ok(base_product)
        } else {
            Err(errors::Error::NotFound.into())
        }
    }

    /// Getting all base products with variants
    fn get_all_catalog(&self) -> RepoResult<Vec<CatalogWithAttributes>> {
        debug!("Getting all base products with variants.");

        let all_base_products = base_products
            .filter(is_active.eq(true))
            .filter(status.eq(ModerationStatus::Published))
            .order(id)
            .get_results(self.db_conn)
            .map_err(From::from)
            .map_err(|e: FailureError| e.context("Getting all base products with variants."))?;

        let all_products = RawProduct::belonging_to(&all_base_products)
            .filter(Products::is_active.eq(true))
            .get_results(self.db_conn)
            .map_err(From::from)
            .map_err(|e: FailureError| e.context("Getting all variants."))?
            .grouped_by(&all_base_products);

        all_base_products
            .into_iter()
            .zip(all_products)
            .map(|(base, variants): (BaseProduct, Vec<RawProduct>)| {
                let prod_ids = variants.iter().map(|v| v.id).collect::<Vec<ProductId>>();

                let query = DslProdAttr::prod_attr_values
                    .filter(DslProdAttr::prod_id.eq_any(prod_ids))
                    .inner_join(DslAttributes::attributes);

                query
                    .get_results::<(ProdAttr, Attribute)>(self.db_conn)
                    .map_err(From::from)
                    .and_then(|attributes| {
                        let mut variants_attributes = vec![];
                        for variant in variants {
                            let search_attributes = attributes.clone();
                            let prod_attributes =
                                search_attributes
                                    .into_iter()
                                    .filter(|v| v.0.prod_id == variant.id)
                                    .collect::<Vec<(ProdAttr, Attribute)>>();
                            let product = ProductWithAttributes::new(variant, prod_attributes);

                            variants_attributes.push(product);
                        }

                        Ok(CatalogWithAttributes::new(base, variants_attributes))
                    })
            }).collect::<RepoResult<Vec<_>>>()
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, BaseProduct>
    for BaseProductsRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id: UserId, scope: &Scope, obj: Option<&BaseProduct>) -> bool {
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

fn by_moderator_search_terms(term: &ModeratorBaseProductSearchTerms) -> Box<BoxableExpression<base_products, Pg, SqlType = Bool>> {
    let mut expr: Box<BoxableExpression<base_products, Pg, SqlType = Bool>> = Box::new(id.eq(id));

    if let Some(term_name) = term.name.clone() {
        let ilike_expr = sql("name::text ILIKE concat('%', $1, '%')").bind::<VarChar, _>(term_name);
        expr = Box::new(expr.and(ilike_expr));
    }

    if let Some(term_store_id) = term.store_id.clone() {
        expr = Box::new(expr.and(store_id.eq(term_store_id)));
    }

    if let Some(term_state) = term.state.clone() {
        expr = Box::new(expr.and(status.eq(term_state)));
    }

    expr
}
