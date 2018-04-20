//! Base product service
use std::collections::{HashMap, HashSet};

use futures::future::*;
use futures::future;
use futures_cpupool::CpuPool;
use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use serde_json;
use r2d2::{ManageConnection, Pool};

use models::*;
use elastic::{ProductsElastic, ProductsElasticImpl};
use super::types::ServiceFuture;
use repos::ReposFactory;
use repos::remove_unused_categories;
use repos::get_parent_category;
use repos::clear_child_categories;
use repos::get_all_children_till_the_end;
use super::error::ServiceError;
use repos::error::RepoError;

use stq_http::client::ClientHandle;

const MAX_PRODUCTS_SEARCH_COUNT: i32 = 1000;

pub trait BaseProductsService {
    /// Find product by name limited by `count` and `offset` parameters
    fn search_by_name(&self, prod: SearchProductsByName, count: i32, offset: i32) -> ServiceFuture<Vec<BaseProduct>>;
    /// Find product by views limited by `count` and `offset` parameters
    fn search_most_viewed(&self, prod: MostViewedProducts, count: i32, offset: i32) -> ServiceFuture<Vec<BaseProduct>>;
    /// Find product by dicount pattern limited by `count` and `offset` parameters
    fn search_most_discount(&self, prod: MostDiscountProducts, count: i32, offset: i32) -> ServiceFuture<Vec<BaseProduct>>;
    /// auto complete limited by `count` and `offset` parameters
    fn auto_complete(&self, name: String, count: i32, offset: i32) -> ServiceFuture<Vec<String>>;
    /// search filters
    fn search_filters_price(&self, search_prod: SearchProductsByName) -> ServiceFuture<RangeFilter>;
    /// search filters
    fn search_filters_category(&self, search_prod: SearchProductsByName) -> ServiceFuture<Category>;
    /// search filters
    fn search_filters_attributes(&self, search_prod: SearchProductsByName) -> ServiceFuture<Option<Vec<AttributeFilter>>>;
    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<BaseProduct>;
    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<BaseProduct>;
    /// Creates base product
    fn create(&self, payload: NewBaseProduct) -> ServiceFuture<BaseProduct>;
    /// Lists base products limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> ServiceFuture<Vec<BaseProduct>>;
    /// Returns list of base_products by store id and exclude base_product_id_arg, limited by 10
    fn get_products_of_the_store(
        &self,
        store_id: i32,
        skip_base_product_id: Option<i32>,
        from: i32,
        count: i32,
    ) -> ServiceFuture<Vec<BaseProduct>>;
    /// Updates base product
    fn update(&self, product_id: i32, payload: UpdateBaseProduct) -> ServiceFuture<BaseProduct>;
}

/// Products services, responsible for Product-related CRUD operations
pub struct BaseProductsServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub user_id: Option<i32>,
    pub client_handle: ClientHandle,
    pub elastic_address: String,
    pub repo_factory: F,
}

impl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> BaseProductsServiceImpl<T, M, F>
{
    pub fn new(
        db_pool: Pool<M>,
        cpu_pool: CpuPool,
        user_id: Option<i32>,
        client_handle: ClientHandle,
        elastic_address: String,
        repo_factory: F,
    ) -> Self {
        Self {
            db_pool,
            cpu_pool,
            user_id,
            client_handle,
            elastic_address,
            repo_factory,
        }
    }

    fn linearize_categories(&self, options: Option<ProductsSearchOptions>) -> ServiceFuture<Option<ProductsSearchOptions>> {
        let cpu_pool = self.cpu_pool.clone();
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        let category_id = options
            .clone()
            .map(|options| options.category_id)
            .and_then(|c| c);

        Box::new(cpu_pool.spawn_fn({
            let db_pool = db_pool.clone();
            let repo_factory = repo_factory.clone();
            move || {
                db_pool
                    .get()
                    .map_err(|e| {
                        error!(
                            "Could not get connection to db from pool! {}",
                            e.to_string()
                        );
                        ServiceError::Connection(e.into())
                    })
                    .and_then(move |conn| {
                        if let Some(category_id) = category_id {
                            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                            categories_repo.find(category_id).and_then(|cat| {
                                let cats_ids = if cat.children.is_empty() {
                                    vec![category_id]
                                } else {
                                    get_all_children_till_the_end(cat)
                                        .into_iter()
                                        .map(|c| c.id)
                                        .collect()
                                };
                                let options = options.map(|mut options| {
                                    options.categories_ids = Some(cats_ids);
                                    options
                                });
                                Ok(options)
                            })
                        } else {
                            Ok(options)
                        }.map_err(ServiceError::from)
                    })
            }
        }))
    }

    fn accept_only_categories_without_children(
        &self,
        options: Option<ProductsSearchOptions>,
    ) -> ServiceFuture<Option<ProductsSearchOptions>> {
        let cpu_pool = self.cpu_pool.clone();
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        let category_id = options
            .clone()
            .map(|options| options.category_id)
            .and_then(|c| c);

        Box::new(cpu_pool.spawn_fn({
            let db_pool = db_pool.clone();
            let repo_factory = repo_factory.clone();
            move || {
                db_pool
                    .get()
                    .map_err(|e| {
                        error!(
                            "Could not get connection to db from pool! {}",
                            e.to_string()
                        );
                        ServiceError::Connection(e.into())
                    })
                    .and_then(move |conn| {
                        if let Some(category_id) = category_id {
                            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                            categories_repo.find(category_id).and_then(|cat| {
                                let cats_ids = if cat.children.is_empty() {
                                    Some(vec![category_id])
                                } else {
                                    None
                                };
                                let options = options.map(|mut options| {
                                    options.categories_ids = cats_ids;
                                    options
                                });
                                Ok(options)
                            })
                        } else {
                            Ok(options)
                        }.map_err(ServiceError::from)
                    })
            }
        }))
    }
}

impl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> BaseProductsService for BaseProductsServiceImpl<T, M, F>
{
    fn search_by_name(&self, mut search_product: SearchProductsByName, count: i32, offset: i32) -> ServiceFuture<Vec<BaseProduct>> {
        let cpu_pool = self.cpu_pool.clone();
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let products_el = ProductsElasticImpl::new(client_handle, address);

        Box::new(
            self.linearize_categories(search_product.options.clone())
                .and_then(move |options| {
                    search_product.options = options;
                    products_el
                        .search_by_name(search_product, count, offset)
                        .map_err(ServiceError::from)
                        .and_then({
                            let db_pool = db_pool.clone();
                            let repo_factory = repo_factory.clone();
                            move |el_products| {
                                cpu_pool.spawn_fn(move || {
                                    db_pool
                                        .get()
                                        .map_err(|e| {
                                            error!(
                                                "Could not get connection to db from pool! {}",
                                                e.to_string()
                                            );
                                            ServiceError::Connection(e.into())
                                        })
                                        .and_then(move |conn| {
                                            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                                            el_products
                                                .into_iter()
                                                .map(|el_product| {
                                                    base_products_repo
                                                        .find(el_product.id)
                                                        .map_err(ServiceError::from)
                                                })
                                                .collect()
                                        })
                                })
                            }
                        })
                }),
        )
    }

    /// Find product by views limited by `count` and `offset` parameters
    fn search_most_viewed(&self, mut search_product: MostViewedProducts, count: i32, offset: i32) -> ServiceFuture<Vec<BaseProduct>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let products_el = ProductsElasticImpl::new(client_handle, address);
        let cpu_pool = self.cpu_pool.clone();
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.linearize_categories(search_product.options.clone())
                .and_then(move |options| {
                    search_product.options = options;
                    products_el
                        .search_most_viewed(search_product, count, offset)
                        .map_err(ServiceError::from)
                        .and_then({
                            move |el_products| {
                                cpu_pool.spawn_fn(move || {
                                    db_pool
                                        .get()
                                        .map_err(|e| {
                                            error!(
                                                "Could not get connection to db from pool! {}",
                                                e.to_string()
                                            );
                                            ServiceError::Connection(e.into())
                                        })
                                        .and_then(move |conn| {
                                            el_products
                                                .into_iter()
                                                .map(|el_product| {
                                                    let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                                                    base_products_repo
                                                        .find(el_product.id)
                                                        .map_err(ServiceError::from)
                                                })
                                                .collect()
                                        })
                                })
                            }
                        })
                }),
        )
    }

    /// Find product by dicount pattern limited by `count` and `offset` parameters
    fn search_most_discount(&self, mut search_product: MostDiscountProducts, count: i32, offset: i32) -> ServiceFuture<Vec<BaseProduct>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let products_el = ProductsElasticImpl::new(client_handle, address);
        let cpu_pool = self.cpu_pool.clone();
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.linearize_categories(search_product.options.clone())
                .and_then(move |options| {
                    search_product.options = options;
                    products_el
                        .search_most_discount(search_product, count, offset)
                        .map_err(ServiceError::from)
                        .and_then({
                            move |el_products| {
                                cpu_pool.spawn_fn(move || {
                                    db_pool
                                        .get()
                                        .map_err(|e| {
                                            error!(
                                                "Could not get connection to db from pool! {}",
                                                e.to_string()
                                            );
                                            ServiceError::Connection(e.into())
                                        })
                                        .and_then(move |conn| {
                                            el_products
                                                .into_iter()
                                                .map(|el_product| {
                                                    let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                                                    base_products_repo
                                                        .find(el_product.id)
                                                        .map_err(ServiceError::from)
                                                })
                                                .collect()
                                        })
                                })
                            }
                        })
                }),
        )
    }

    fn auto_complete(&self, name: String, count: i32, offset: i32) -> ServiceFuture<Vec<String>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let products_names = {
            let products_el = ProductsElasticImpl::new(client_handle, address);
            products_el
                .auto_complete(name, count, offset)
                .map_err(ServiceError::from)
        };

        Box::new(products_names)
    }

    fn search_filters_price(&self, mut search_product: SearchProductsByName) -> ServiceFuture<RangeFilter> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let products_el = ProductsElasticImpl::new(client_handle, address);

        Box::new(
            self.linearize_categories(search_product.options.clone())
                .and_then(move |options| {
                    search_product.options = options;
                    products_el
                        .aggregate_price(search_product)
                        .map_err(ServiceError::from)
                }),
        )
    }

    /// search filters
    fn search_filters_category(&self, search_prod: SearchProductsByName) -> ServiceFuture<Category> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let cpu_pool = self.cpu_pool.clone();
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();
        let products_el = ProductsElasticImpl::new(client_handle, address);

        if search_prod.name.is_empty() {
            let category_id = search_prod
                .options
                .map(|options| options.category_id)
                .and_then(|c| c);
            Box::new(cpu_pool.spawn_fn(move || {
                db_pool
                    .get()
                    .map_err(|e| {
                        error!(
                            "Could not get connection to db from pool! {}",
                            e.to_string()
                        );
                        ServiceError::Connection(e.into())
                    })
                    .and_then(move |conn| {
                        let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                        categories_repo
                            .get_all()
                            .and_then(|category| {
                                if let Some(category_id) = category_id {
                                    let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                                    categories_repo.find(category_id).and_then(|cat| {
                                        if cat.children.is_empty() {
                                            let new_cat = remove_unused_categories(
                                                category,
                                                &[cat.parent_id.unwrap_or_default()],
                                                cat.level - 2,
                                            );
                                            Ok(new_cat)
                                        } else {
                                            let new_cat = remove_unused_categories(category, &[cat.id], cat.level - 1);
                                            let removed_cat = clear_child_categories(new_cat, cat.level + 1);
                                            Ok(removed_cat)
                                        }
                                    })
                                } else {
                                    Ok(category)
                                }
                            })
                            .map_err(ServiceError::from)
                    })
            }))
        } else {
            Box::new(
                products_el
                    .aggregate_categories(search_prod.name.clone())
                    .map_err(ServiceError::from)
                    .and_then(move |cats| {
                        cpu_pool.spawn_fn(move || {
                            db_pool
                                .get()
                                .map_err(|e| {
                                    error!(
                                        "Could not get connection to db from pool! {}",
                                        e.to_string()
                                    );
                                    ServiceError::Connection(e.into())
                                })
                                .and_then(move |conn| {
                                    let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                                    categories_repo.get_all().map_err(ServiceError::from)
                                })
                                .and_then(|category| {
                                    let new_cat = remove_unused_categories(category, &cats, 2);
                                    Ok(new_cat)
                                })
                        })
                    }),
            )
        }
    }

    /// search filters
    fn search_filters_attributes(&self, mut search_product: SearchProductsByName) -> ServiceFuture<Option<Vec<AttributeFilter>>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let products_el = ProductsElasticImpl::new(client_handle, address);
        Box::new(
            self.accept_only_categories_without_children(search_product.options.clone())
                .and_then(
                    move |options| -> ServiceFuture<Option<Vec<AttributeFilter>>> {
                        search_product.options = options;
                        if let Some(options) = search_product.options.clone() {
                            if options.categories_ids.is_some() {
                                return Box::new(
                                    products_el
                                        .search_by_name(search_product, MAX_PRODUCTS_SEARCH_COUNT, 0)
                                        .map_err(ServiceError::from)
                                        .and_then(|el_products| {
                                            let mut equal_attrs = HashMap::<i32, HashSet<String>>::default();
                                            let mut range_attrs = HashMap::<i32, RangeFilter>::default();

                                            for product in el_products {
                                                for variant in product.variants {
                                                    for attr_value in variant.attrs {
                                                        if let Some(value) = attr_value.str_val {
                                                            let equal = equal_attrs
                                                                .entry(attr_value.attr_id)
                                                                .or_insert_with(HashSet::<String>::default);
                                                            equal.insert(value);
                                                        }
                                                        if let Some(value) = attr_value.float_val {
                                                            let range = range_attrs
                                                                .entry(attr_value.attr_id)
                                                                .or_insert_with(RangeFilter::default);
                                                            range.add_value(value);
                                                        }
                                                    }
                                                }
                                            }

                                            let eq_filters = equal_attrs.into_iter().map(|(k, v)| AttributeFilter {
                                                id: k,
                                                equal: Some(EqualFilter {
                                                    values: v.iter().cloned().collect(),
                                                }),
                                                range: None,
                                            });

                                            let range_filters = range_attrs.into_iter().map(|(k, v)| AttributeFilter {
                                                id: k,
                                                equal: None,
                                                range: Some(v),
                                            });

                                            future::ok(Some(eq_filters.chain(range_filters).collect()))
                                        }),
                                );
                            }
                        }
                        return Box::new(future::ok(None));
                    },
                ),
        )
    }

    /// Returns product by ID
    fn get(&self, product_id: i32) -> ServiceFuture<BaseProduct> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                    base_products_repo
                        .find(product_id)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Deactivates specific base product
    fn deactivate(&self, product_id: i32) -> ServiceFuture<BaseProduct> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                    let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                    let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                    base_products_repo
                        .deactivate(product_id)
                        .and_then(|prod| {
                            categories_repo
                                .get_all()
                                .and_then(|category_root| {
                                    category_root
                                        .children
                                        .into_iter()
                                        .find(|cat_child| get_parent_category(&cat_child, prod.category_id, 2).is_some())
                                        .ok_or_else(|| RepoError::NotFound)
                                })
                                .and_then(|cat| stores_repo.find(prod.store_id).map(|store| (store, cat)))
                                .and_then(|(store, cat)| {
                                    let prod_cats = if let Some(prod_cats) = store.product_categories.clone() {
                                        let mut product_categories =
                                            serde_json::from_value::<Vec<ProductCategories>>(prod_cats).unwrap_or_default();
                                        let mut new_prod_cats = vec![];
                                        for pc in product_categories.iter_mut() {
                                            if pc.category_id == cat.id {
                                                pc.count -= 1;
                                            }
                                            new_prod_cats.push(pc.clone());
                                        }
                                        new_prod_cats
                                    } else {
                                        vec![]
                                    };

                                    let product_categories = serde_json::to_value(prod_cats).ok();

                                    let update_store = UpdateStore {
                                        product_categories,
                                        ..Default::default()
                                    };
                                    stores_repo.update(store.id, update_store)
                                })
                                .and_then(|_| Ok(prod))
                        })
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Lists base products limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> ServiceFuture<Vec<BaseProduct>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                    base_products_repo
                        .list(from, count)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Returns list of base_products by store id and exclude skip_base_product_id, limited by from and count
    fn get_products_of_the_store(
        &self,
        store_id: i32,
        skip_base_product_id: Option<i32>,
        from: i32,
        count: i32,
    ) -> ServiceFuture<Vec<BaseProduct>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                    base_products_repo
                        .get_products_of_the_store(store_id, skip_base_product_id, from, count)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Creates new product
    fn create(&self, payload: NewBaseProduct) -> ServiceFuture<BaseProduct> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let cpu_pool = self.cpu_pool.clone();
        let repo_factory = self.repo_factory.clone();
        Box::new(cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                    let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                    let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                    conn.transaction::<(BaseProduct), ServiceError, _>(move || {
                        base_products_repo
                            .create(payload)
                            .and_then(|prod| {
                                categories_repo
                                    .get_all()
                                    .and_then(|category_root| {
                                        category_root
                                            .children
                                            .into_iter()
                                            .find(|cat_child| get_parent_category(&cat_child, prod.category_id, 2).is_some())
                                            .ok_or_else(|| {
                                                error!("There is no such 3rd level category in db");
                                                RepoError::NotFound
                                            })
                                    })
                                    .and_then(|cat| stores_repo.find(prod.store_id).map(|store| (store, cat)))
                                    .and_then(|(store, cat)| {
                                        let prod_cats = if let Some(prod_cats) = store.product_categories.clone() {
                                            let mut product_categories =
                                                serde_json::from_value::<Vec<ProductCategories>>(prod_cats).unwrap_or_default();
                                            let mut new_prod_cats = vec![];
                                            let mut cat_exists = false;
                                            for pc in product_categories.iter_mut() {
                                                if pc.category_id == cat.id {
                                                    pc.count += 1;
                                                    cat_exists = true;
                                                }
                                                new_prod_cats.push(pc.clone());
                                            }
                                            if !cat_exists {
                                                new_prod_cats.push(ProductCategories::new(cat.id));
                                            }
                                            new_prod_cats
                                        } else {
                                            let pc = ProductCategories::new(cat.id);
                                            vec![pc]
                                        };

                                        let product_categories = serde_json::to_value(prod_cats).ok();

                                        let update_store = UpdateStore {
                                            product_categories,
                                            ..Default::default()
                                        };
                                        stores_repo.update(store.id, update_store)
                                    })
                                    .and_then(|_| Ok(prod))
                            })
                            .map_err(ServiceError::from)
                    })
                })
        }))
    }

    /// Updates specific product
    fn update(&self, product_id: i32, payload: UpdateBaseProduct) -> ServiceFuture<BaseProduct> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let cpu_pool = self.cpu_pool.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!(
                        "Could not get connection to db from pool! {}",
                        e.to_string()
                    );
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                    let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                    let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                    conn.transaction::<(BaseProduct), ServiceError, _>(move || {
                        if let Some(new_cat_id) = payload.category_id {
                            base_products_repo
                                .find(product_id)
                                .and_then(|old_prod| {
                                    let old_cat_id = old_prod.category_id;
                                    categories_repo
                                        .get_all()
                                        .and_then(|category_root| {
                                            category_root
                                                .children
                                                .into_iter()
                                                .find(|cat_child| get_parent_category(&cat_child, old_cat_id, 2).is_some())
                                                .ok_or_else(|| RepoError::NotFound)
                                        })
                                        .map(|old_cat| (old_cat, old_prod))
                                })
                                .and_then(|(old_cat, old_prod)| {
                                    categories_repo
                                        .get_all()
                                        .and_then(|category_root| {
                                            category_root
                                                .children
                                                .into_iter()
                                                .find(|cat_child| get_parent_category(&cat_child, new_cat_id, 2).is_some())
                                                .ok_or_else(|| RepoError::NotFound)
                                        })
                                        .map(|new_cat| (new_cat, old_cat, old_prod))
                                })
                                .and_then(|(new_cat, old_cat, old_prod)| {
                                    stores_repo
                                        .find(old_prod.store_id)
                                        .map(|store| (store, new_cat, old_cat))
                                })
                                .and_then(|(store, new_cat, old_cat)| {
                                    if new_cat.id != old_cat.id {
                                        let prod_cats = if let Some(prod_cats) = store.product_categories.clone() {
                                            let mut product_categories =
                                                serde_json::from_value::<Vec<ProductCategories>>(prod_cats).unwrap_or_default();
                                            let mut new_prod_cats = vec![];
                                            let mut new_cat_exists = false;
                                            for pc in product_categories.iter_mut() {
                                                if pc.category_id == new_cat.id {
                                                    pc.count += 1;
                                                    new_cat_exists = true;
                                                }
                                                if pc.category_id == old_cat.id {
                                                    pc.count -= 1;
                                                }
                                                new_prod_cats.push(pc.clone());
                                            }
                                            if !new_cat_exists {
                                                new_prod_cats.push(ProductCategories::new(new_cat.id));
                                            }
                                            new_prod_cats
                                        } else {
                                            let pc = ProductCategories::new(new_cat.id);
                                            vec![pc]
                                        };

                                        let product_categories = serde_json::to_value(prod_cats).ok();

                                        let update_store = UpdateStore {
                                            product_categories,
                                            ..Default::default()
                                        };
                                        stores_repo.update(store.id, update_store).map(|_| ())
                                    } else {
                                        Ok(())
                                    }
                                })
                                .and_then(|_| base_products_repo.update(product_id, payload.clone()))
                                .map_err(ServiceError::from)
                        } else {
                            base_products_repo
                                .update(product_id, payload.clone())
                                .map_err(ServiceError::from)
                        }
                    })
                })
        }))
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use futures_cpupool::CpuPool;
    use tokio_core::reactor::Handle;
    use tokio_core::reactor::Core;
    use r2d2;
    use serde_json;

    use stq_http::client::Config as HttpConfig;
    use stq_http;

    use repos::repo_factory::tests::*;
    use services::*;
    use models::*;
    use config::Config;

    #[allow(unused)]
    fn create_base_product_service(
        user_id: Option<i32>,
        handle: Arc<Handle>,
    ) -> BaseProductsServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        let config = Config::new().unwrap();
        let http_config = HttpConfig {
            http_client_retries: config.client.http_client_retries,
            http_client_buffer_size: config.client.http_client_buffer_size,
        };
        let client = stq_http::client::Client::new(&http_config, &handle);
        let client_handle = client.handle();

        BaseProductsServiceImpl {
            db_pool: db_pool,
            cpu_pool: cpu_pool,
            user_id: user_id,
            client_handle: client_handle,
            elastic_address: "".to_string(),
            repo_factory: MOCK_REPO_FACTORY,
        }
    }

    pub fn create_new_base_product(name: &str) -> NewBaseProduct {
        NewBaseProduct {
            name: serde_json::from_str(name).unwrap(),
            store_id: 1,
            short_description: serde_json::from_str("{}").unwrap(),
            long_description: None,
            seo_title: None,
            seo_description: None,
            currency_id: 1,
            category_id: 1,
        }
    }

    pub fn create_update_base_product(name: &str) -> UpdateBaseProduct {
        UpdateBaseProduct {
            name: Some(serde_json::from_str(name).unwrap()),
            short_description: Some(serde_json::from_str("{}").unwrap()),
            long_description: None,
            seo_title: None,
            seo_description: None,
            currency_id: Some(1),
            category_id: Some(1),
        }
    }

    #[test]
    fn test_get_base_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_base_product_service(Some(MOCK_USER_ID), handle);
        let work = service.get(1);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
    }

    #[test]
    fn test_list() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_base_product_service(Some(MOCK_USER_ID), handle);
        let work = service.list(1, 5);
        let result = core.run(work).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_create_base_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_base_product_service(Some(MOCK_USER_ID), handle);
        let new_base_product = create_new_base_product(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.create(new_base_product);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, MOCK_BASE_PRODUCT_ID);
    }

    #[test]
    fn test_update() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_base_product_service(Some(MOCK_USER_ID), handle);
        let new_base_product = create_update_base_product(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.update(1, new_base_product);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.id, MOCK_BASE_PRODUCT_ID);
    }

    #[test]
    fn test_deactivate() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_base_product_service(Some(MOCK_USER_ID), handle);
        let work = service.deactivate(1);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.is_active, false);
    }

}
