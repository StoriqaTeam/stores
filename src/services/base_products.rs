//! Base product service
use std::collections::{BTreeMap, HashMap, HashSet};

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use futures::future;
use futures::future::*;
use r2d2::ManageConnection;

use stq_static_resources::{Currency, ModerationStatus};
use stq_types::{BaseProductId, BaseProductSlug, CategoryId, ExchangeRate, ProductId, StoreId, StoreIdentifier};

use super::types::ServiceFuture;
use elastic::{ProductsElastic, ProductsElasticImpl};
use errors::Error;
use models::*;
use repos::clear_child_categories;
use repos::get_all_children_till_the_end;
use repos::get_parent_category;
use repos::remove_unused_categories;
use repos::{
    BaseProductsRepo, BaseProductsSearchTerms, CategoriesRepo, ProductAttrsRepo, ProductsRepo, RepoResult, ReposFactory, StoresRepo,
};
use services::create_product_attributes_values;
use services::products::calculate_customer_price;
use services::Service;
use services::{check_can_update_by_status, check_change_status, check_vendor_code};

const MAX_PRODUCTS_SEARCH_COUNT: i32 = 1000;

pub trait BaseProductsService {
    /// Returns base product count
    fn base_product_count(&self, visibility: Option<Visibility>) -> ServiceFuture<i64>;

    /// Find product by name limited by `count` and `offset` parameters
    fn search_base_products_by_name(
        self,
        prod: SearchProductsByName,
        count: i32,
        offset: i32,
    ) -> ServiceFuture<Vec<BaseProductWithVariants>>;

    /// Find product by views limited by `count` and `offset` parameters
    fn search_base_products_most_viewed(
        &self,
        prod: MostViewedProducts,
        count: i32,
        offset: i32,
    ) -> ServiceFuture<Vec<BaseProductWithVariants>>;

    /// Find product by discount pattern limited by `count` and `offset` parameters
    fn search_base_products_most_discount(
        self,
        prod: MostDiscountProducts,
        count: i32,
        offset: i32,
    ) -> ServiceFuture<Vec<BaseProductWithVariants>>;

    /// auto complete limited by `count` and `offset` parameters
    fn base_products_auto_complete(&self, name: AutoCompleteProductName, count: i32, offset: i32) -> ServiceFuture<Vec<String>>;

    /// search filters
    fn search_base_products_filters_price(self, search_prod: SearchProductsByName) -> ServiceFuture<RangeFilter>;

    /// search filters
    fn search_base_products_filters_category(self, search_prod: SearchProductsByName) -> ServiceFuture<Category>;

    /// search filters
    fn search_base_products_attributes(&self, search_prod: SearchProductsByName) -> ServiceFuture<Option<Vec<AttributeFilter>>>;

    /// search filters
    fn search_base_products_filters_count(&self, search_prod: SearchProductsByName) -> ServiceFuture<i32>;

    /// Returns product by ID
    fn get_base_product(&self, base_product_id: BaseProductId, visibility: Option<Visibility>) -> ServiceFuture<Option<BaseProduct>>;

    /// Returns products by IDs
    fn get_base_products(&self, base_product_ids: Vec<BaseProductId>) -> ServiceFuture<Vec<BaseProduct>>;

    /// Returns product by ID
    fn get_base_product_without_filters(&self, base_product_id: BaseProductId) -> ServiceFuture<Option<BaseProduct>>;

    /// Returns product by Slug
    fn get_base_product_by_slug(
        &self,
        store_identifier: StoreIdentifier,
        base_product_slug: BaseProductSlug,
        visibility: Option<Visibility>,
    ) -> ServiceFuture<Option<BaseProduct>>;

    /// Returns base product by ID with update views
    fn get_base_product_with_views_update(&self, base_product_id: BaseProductId) -> ServiceFuture<Option<BaseProduct>>;

    /// Returns base product by Slug with update views
    fn get_base_product_by_slug_with_views_update(
        &self,
        store_identifier: StoreIdentifier,
        base_product_slug: BaseProductSlug,
    ) -> ServiceFuture<Option<BaseProduct>>;

    /// Returns base_product by product ID
    fn get_base_product_by_product(
        &self,
        product_id: ProductId,
        visibility: Option<Visibility>,
    ) -> ServiceFuture<Option<BaseProductWithVariants>>;

    /// Deactivates specific product
    fn deactivate_base_product(&self, base_product_id: BaseProductId) -> ServiceFuture<BaseProduct>;

    /// Creates base product
    fn create_base_product(&self, payload: NewBaseProduct) -> ServiceFuture<BaseProduct>;

    /// Creates base product with variants
    fn create_base_product_with_variants(&self, payload: NewBaseProductWithVariants) -> ServiceFuture<BaseProduct>;

    /// Lists base products limited by `from` and `count` parameters
    fn list_base_products(&self, from: BaseProductId, count: i32, visibility: Option<Visibility>) -> ServiceFuture<Vec<BaseProduct>>;

    /// Returns list of base_products by store id and exclude base_product_id_arg, limited by 10
    fn get_base_products_of_the_store(
        &self,
        store_id: StoreId,
        skip_base_product_id: Option<BaseProductId>,
        from: BaseProductId,
        count: i32,
        visibility: Option<Visibility>,
    ) -> ServiceFuture<Vec<BaseProduct>>;

    /// Updates base product
    fn update_base_product(&self, base_product_id: BaseProductId, payload: UpdateBaseProduct) -> ServiceFuture<BaseProduct>;

    /// Cart
    fn find_by_cart(&self, cart: Vec<CartProduct>) -> ServiceFuture<Vec<StoreWithBaseProducts>>;

    /// Search base products limited by `from`, `skip` and `count` parameters
    fn moderator_search_base_product(
        &self,
        from: Option<BaseProductId>,
        skip: i64,
        count: i64,
        term: ModeratorBaseProductSearchTerms,
    ) -> ServiceFuture<ModeratorBaseProductSearchResults>;

    /// Set moderation status for base_product_ids. For moderator
    fn set_moderation_status_base_products(
        &self,
        base_product_ids: Vec<BaseProductId>,
        status: ModerationStatus,
    ) -> ServiceFuture<Vec<BaseProduct>>;

    /// Set moderation status for base_product_id
    fn set_moderation_status_base_product(&self, base_product_id: BaseProductId, status: ModerationStatus) -> ServiceFuture<BaseProduct>;

    /// send base product to moderation from store manager
    fn send_base_product_to_moderation(&self, base_product_id: BaseProductId) -> ServiceFuture<BaseProduct>;

    /// Hide base product from search. For store manager
    fn set_base_product_moderation_status_draft(&self, base_product_id: BaseProductId) -> ServiceFuture<BaseProduct>;

    // Check that you can change the moderation status
    fn validate_change_moderation_status_base_product(
        &self,
        base_product_id: BaseProductId,
        status: ModerationStatus,
    ) -> ServiceFuture<bool>;

    // Flattens categories
    fn flatten_categories(&self, options: Option<ProductsSearchOptions>) -> ServiceFuture<Option<ProductsSearchOptions>>;

    /// Remove categories not 3rd level
    fn remove_non_third_level_categories(&self, options: Option<ProductsSearchOptions>) -> ServiceFuture<Option<ProductsSearchOptions>>;

    /// Create currency map
    fn create_currency_map(&self, options: Option<ProductsSearchOptions>) -> ServiceFuture<Option<ProductsSearchOptions>>;

    /// Replace category in all base products
    fn replace_category(&self, payload: CategoryReplacePayload) -> ServiceFuture<Vec<BaseProduct>>;

    /// Check that you can update base product
    fn validate_update_base_product(&self, base_product_id: BaseProductId) -> ServiceFuture<bool>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > BaseProductsService for Service<T, M, F>
{
    /// Returns base product count
    fn base_product_count(&self, visibility: Option<Visibility>) -> ServiceFuture<i64> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!("Getting base product count with visibility = {:?}", visibility);

        self.spawn_on_pool(move |conn| {
            let base_product_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            base_product_repo
                .count(visibility.unwrap_or(Visibility::Active))
                .map_err(|e: FailureError| e.context("Service `base_products`, `count` endpoint error occurred.").into())
        })
    }

    fn search_base_products_by_name(
        self,
        mut search_product: SearchProductsByName,
        count: i32,
        offset: i32,
    ) -> ServiceFuture<Vec<BaseProductWithVariants>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;
        let client_handle = self.static_context.client_handle.clone();
        let currency = self.dynamic_context.currency;
        let fiat_currency = self.dynamic_context.fiat_currency;
        let address = self.static_context.config.server.elastic.clone();
        let products_el = ProductsElasticImpl::new(client_handle, address);
        let service = self.clone();
        Box::new(
            self.flatten_categories(search_product.options.clone())
                .and_then(move |options| self.create_currency_map(options))
                .and_then(move |options| {
                    search_product.options = options;
                    products_el.search_by_name(search_product, count, offset)
                })
                .and_then({
                    move |el_products| {
                        service.spawn_on_pool(move |conn| {
                            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                            let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                            let mut base_products = base_products_repo.convert_from_elastic(el_products)?;
                            let latest_currencies = currency_exchange.get_latest()?;
                            calculate_base_products_customer_price(&mut base_products, latest_currencies, currency, fiat_currency);
                            Ok(base_products)
                        })
                    }
                })
                .map_err(|e| {
                    e.context("Service BaseProduct, search_base_products_by_name endpoint error occurred.")
                        .into()
                }),
        )
    }

    /// Find product by views limited by `count` and `offset` parameters
    fn search_base_products_most_viewed(
        &self,
        search_product: MostViewedProducts,
        count: i32,
        offset: i32,
    ) -> ServiceFuture<Vec<BaseProductWithVariants>> {
        let user_id = self.dynamic_context.user_id;
        let currency = self.dynamic_context.currency;
        let fiat_currency = self.dynamic_context.fiat_currency;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            {
                let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let mut base_products = base_products_repo.most_viewed(search_product, count, offset)?;
                let latest_currencies = currency_exchange.get_latest()?;
                calculate_base_products_customer_price(&mut base_products, latest_currencies, currency, fiat_currency);
                Ok(base_products)
            }
            .map_err(|e: FailureError| {
                e.context("Service BaseProduct, search_base_products_most_viewed endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Find product by discount pattern limited by `count` and `offset` parameters
    fn search_base_products_most_discount(
        self,
        mut search_product: MostDiscountProducts,
        count: i32,
        offset: i32,
    ) -> ServiceFuture<Vec<BaseProductWithVariants>> {
        let client_handle = self.static_context.client_handle.clone();
        let address = self.static_context.config.server.elastic.clone();
        let products_el = ProductsElasticImpl::new(client_handle, address);

        let user_id = self.dynamic_context.user_id;
        let currency = self.dynamic_context.currency;
        let fiat_currency = self.dynamic_context.fiat_currency;
        let repo_factory = self.static_context.repo_factory.clone();
        Box::new(
            self.flatten_categories(search_product.options.clone())
                .and_then(move |options| {
                    search_product.options = options;
                    products_el.search_most_discount(search_product, count, offset)
                })
                .and_then({
                    move |el_products| {
                        self.spawn_on_pool(move |conn| {
                            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                            let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                            let mut base_products = base_products_repo.convert_from_elastic(el_products)?;
                            let latest_currencies = currency_exchange.get_latest()?;
                            calculate_base_products_customer_price(&mut base_products, latest_currencies, currency, fiat_currency);
                            Ok(base_products)
                        })
                    }
                })
                .map_err(|e| {
                    e.context("Service BaseProduct, search_base_products_most_discount endpoint error occurred.")
                        .into()
                }),
        )
    }

    fn base_products_auto_complete(&self, name: AutoCompleteProductName, count: i32, offset: i32) -> ServiceFuture<Vec<String>> {
        let client_handle = self.static_context.client_handle.clone();
        let address = self.static_context.config.server.elastic.clone();
        let products_names = {
            let products_el = ProductsElasticImpl::new(client_handle, address);
            products_el.auto_complete(name, count, offset)
        };

        Box::new(products_names.map_err(|e| {
            e.context("Service BaseProduct, base_products_auto_complete endpoint error occurred.")
                .into()
        }))
    }

    fn search_base_products_filters_price(self, mut search_product: SearchProductsByName) -> ServiceFuture<RangeFilter> {
        let client_handle = self.static_context.client_handle.clone();
        let address = self.static_context.config.server.elastic.clone();
        let products_el = ProductsElasticImpl::new(client_handle, address);
        Box::new(
            self.flatten_categories(search_product.options.clone())
                .and_then(move |options| self.create_currency_map(options))
                .and_then(move |options| {
                    search_product.options = options;
                    products_el.aggregate_price(search_product)
                })
                .map_err(|e| {
                    e.context("Service BaseProduct, search_base_products_filters_price endpoint error occurred.")
                        .into()
                }),
        )
    }

    /// search filters
    fn search_base_products_filters_count(&self, mut search_prod: SearchProductsByName) -> ServiceFuture<i32> {
        let client_handle = self.static_context.client_handle.clone();
        let address = self.static_context.config.server.elastic.clone();
        let products_el = ProductsElasticImpl::new(client_handle, address);
        Box::new(
            self.flatten_categories(search_prod.options.clone())
                .and_then(move |options| {
                    search_prod.options = options;
                    products_el.count(search_prod)
                })
                .map_err(|e| {
                    e.context("Service BaseProduct, search_base_products_filters_count endpoint error occurred.")
                        .into()
                }),
        )
    }

    /// search filters
    fn search_base_products_filters_category(self, search_prod: SearchProductsByName) -> ServiceFuture<Category> {
        let client_handle = self.static_context.client_handle.clone();
        let address = self.static_context.config.server.elastic.clone();

        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let products_el = ProductsElasticImpl::new(client_handle, address);

        if search_prod.name.is_empty() {
            let category_id = search_prod.options.map(|options| options.category_id).and_then(|c| c);
            self.spawn_on_pool(move |conn| {
                {
                    let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                    let root = categories_repo.get_all_categories_with_products()?;
                    if let Some(category_id) = category_id {
                        let cat = categories_repo.find(category_id)?;
                        Ok(get_path_to_searched_category(cat, root))
                    } else {
                        Ok(root)
                    }
                }
                .map_err(|e: FailureError| {
                    e.context("Service BaseProduct, search_base_products_filters_category endpoint with empty name option error occurred.")
                        .into()
                })
            })
        } else {
            Box::new(products_el.aggregate_categories(search_prod.name.clone()).and_then(move |cats| {
                self.spawn_on_pool(move |conn| {
                    {
                        let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                        let category = categories_repo.get_all_categories()?;
                        let new_cat = remove_unused_categories(category, &cats);
                        Ok(new_cat)
                    }.map_err(|e: FailureError| {
                        e.context("Service BaseProduct, search_base_products_filters_category endpoint with name aggregation in elastic error occurred.")
                            .into()
                    })
                })
            }))
        }
    }

    /// search filters
    fn search_base_products_attributes(&self, mut search_product: SearchProductsByName) -> ServiceFuture<Option<Vec<AttributeFilter>>> {
        let client_handle = self.static_context.client_handle.clone();
        let address = self.static_context.config.server.elastic.clone();
        let products_el = ProductsElasticImpl::new(client_handle, address);
        Box::new(
            self.remove_non_third_level_categories(search_product.options.clone())
                .and_then(move |options| -> ServiceFuture<Option<Vec<AttributeFilter>>> {
                    search_product.options = options;
                    if let Some(options) = search_product.options.clone() {
                        if options.categories_ids.is_some() {
                            return Box::new(
                                products_el
                                    .search_by_name(search_product, MAX_PRODUCTS_SEARCH_COUNT, 0)
                                    .map(get_attribute_filters),
                            );
                        }
                    }
                    Box::new(future::ok(None))
                })
                .map_err(|e| {
                    e.context("Service BaseProduct, search_base_products_attributes endpoint error occurred.")
                        .into()
                }),
        )
    }

    /// Returns product by ID
    fn get_base_product(&self, base_product_id: BaseProductId, visibility: Option<Visibility>) -> ServiceFuture<Option<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let visibility = visibility.unwrap_or(Visibility::Published);

        debug!("Get base product by id = {:?} with visibility = {:?}", base_product_id, visibility);

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            base_products_repo
                .find(base_product_id, visibility)
                .map_err(|e| e.context("Service BaseProduct, get_base_product endpoint error occurred.").into())
        })
    }

    /// Returns products by IDs
    fn get_base_products(&self, base_product_ids: Vec<BaseProductId>) -> ServiceFuture<Vec<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!("Get base products by ids ({})", base_product_ids.len());
        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            base_products_repo
                .find_many(base_product_ids)
                .map_err(|e| e.context("Service BaseProduct, get_base_products endpoint error occurred.").into())
        })
    }

    /// Returns product by ID
    fn get_base_product_without_filters(&self, base_product_id: BaseProductId) -> ServiceFuture<Option<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!("Get base product by id = {:?}", base_product_id);

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let base_product_filters = BaseProductsSearchTerms::default();

            base_products_repo
                .find_by_filters(base_product_id, base_product_filters)
                .map_err(|e| {
                    e.context("Service BaseProduct, get_product_without_filters endpoint error occurred.")
                        .into()
                })
        })
    }

    /// Returns base product by ID with update views
    fn get_base_product_with_views_update(&self, base_product_id: BaseProductId) -> ServiceFuture<Option<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            base_products_repo.update_views(base_product_id).map_err(|e| {
                e.context("Service BaseProduct, get_base_product_with_views_update endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Returns base_product by product ID
    fn get_base_product_by_product(
        &self,
        product_id: ProductId,
        visibility: Option<Visibility>,
    ) -> ServiceFuture<Option<BaseProductWithVariants>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let currency = self.dynamic_context.currency;
        let fiat_currency = self.dynamic_context.fiat_currency;
        let visibility = visibility.unwrap_or(Visibility::Published);

        debug!(
            "Get base product by variant id = {:?} with visibility = {:?}",
            product_id, visibility
        );

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let product = products_repo.find(product_id)?;
                if let Some(product) = product {
                    let base_product = base_products_repo.find(product.base_product_id, visibility).map(|base_product| {
                        base_product.map(|base_product| BaseProductWithVariants::new(base_product, vec![Product::from(product)]))
                    })?;
                    if let Some(base_product) = base_product {
                        let mut base_products = vec![base_product];
                        let latest_currencies = currency_exchange.get_latest()?;
                        calculate_base_products_customer_price(&mut base_products, latest_currencies, currency, fiat_currency);
                        return Ok(base_products.pop());
                    };
                }
                Ok(None)
            }
            .map_err(|e: FailureError| {
                e.context("Service BaseProduct, get_base_product_by_product endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Deactivates specific base product
    fn deactivate_base_product(&self, base_product_id: BaseProductId) -> ServiceFuture<BaseProduct> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
            conn.transaction::<BaseProduct, FailureError, _>(move || {
                let prod = base_products_repo.deactivate(base_product_id)?;
                let _ = products_repo.deactivate_by_base_product(base_product_id)?;
                // update product categories of the store
                let store = stores_repo.find(prod.store_id, Visibility::Active)?;
                if let Some(store) = store {
                    let category_root = categories_repo.get_all_categories()?;
                    let cat = get_first_level_category(prod.category_id, category_root)?;
                    let service_update_store =
                        ServiceUpdateStore::delete_category_from_product_categories(store.product_categories.clone(), cat.id);
                    let _ = stores_repo.update_service_fields(store.id, service_update_store)?;
                };
                Ok(prod)
            })
            .map_err(|e: FailureError| {
                e.context("Service BaseProduct, deactivate_base_product endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Lists base products limited by `from` and `count` parameters
    fn list_base_products(&self, from: BaseProductId, count: i32, visibility: Option<Visibility>) -> ServiceFuture<Vec<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let visibility = visibility.unwrap_or(Visibility::Published);

        debug!(
            "List base products from id = {:?} with count = {}, visibility = {:?}",
            from, count, visibility
        );

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            base_products_repo
                .list(from, count, visibility)
                .map_err(|e| e.context("Service BaseProduct, list endpoint error occurred.").into())
        })
    }

    /// Returns list of base_products by store id and exclude skip_base_product_id, limited by from and count
    fn get_base_products_of_the_store(
        &self,
        store_id: StoreId,
        skip_base_product_id: Option<BaseProductId>,
        from: BaseProductId,
        count: i32,
        visibility: Option<Visibility>,
    ) -> ServiceFuture<Vec<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let visibility = visibility.unwrap_or(Visibility::Published);

        debug!("Get base products of the store with id = {:?} skipping base product with id = {:?}, from id = {:?}, count = {}, visibility = {:?}",
               store_id, skip_base_product_id, from, count, visibility);

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            base_products_repo
                .get_products_of_the_store(store_id, skip_base_product_id, from, count, visibility)
                .map_err(|e| {
                    e.context("Service BaseProduct, get_products_of_the_store endpoint error occurred.")
                        .into()
                })
        })
    }

    /// Creates new base product
    fn create_base_product(&self, mut payload: NewBaseProduct) -> ServiceFuture<BaseProduct> {
        let user_id = self.dynamic_context.user_id;

        let repo_factory = self.static_context.repo_factory.clone();
        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            conn.transaction::<(BaseProduct), FailureError, _>(move || {
                //validate
                validate_base_product(&*base_products_repo, &payload)?;
                //enrich
                enrich_new_base_product(&*stores_repo, &mut payload)?;
                // create base_product
                let base_prod = base_products_repo.create(payload)?;

                // update product categories of the store
                add_product_categories(&*stores_repo, &*categories_repo, base_prod.store_id, base_prod.category_id)?;

                Ok(base_prod)
            })
            .map_err(|e| e.context("Service BaseProduct, create endpoint error occurred.").into())
        })
    }

    /// Creates base product with variants
    fn create_base_product_with_variants(&self, payload: NewBaseProductWithVariants) -> ServiceFuture<BaseProduct> {
        let user_id = self.dynamic_context.user_id;

        let repo_factory = self.static_context.repo_factory.clone();
        let NewBaseProductWithVariants {
            mut new_base_product,
            variants,
            selected_attributes,
        } = payload;

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
            let attr_repo = repo_factory.create_attributes_repo(&*conn, user_id);
            let attribute_values_repo = repo_factory.create_attribute_values_repo(&*conn, user_id);
            let custom_attributes_repo = repo_factory.create_custom_attributes_repo(&*conn, user_id);

            conn.transaction::<BaseProduct, FailureError, _>(move || {
                //validate base_product
                validate_base_product(&*base_products_repo, &new_base_product)?;
                //enrich base_product
                enrich_new_base_product(&*stores_repo, &mut new_base_product)?;
                // create base_product
                let base_prod = base_products_repo.create(new_base_product)?;
                let base_prod_id = base_prod.id;
                let store_id = base_prod.store_id;

                // update product categories of the store
                add_product_categories(&*stores_repo, &*categories_repo, base_prod.store_id, base_prod.category_id)?;

                let variants = variants.into_iter().map(move |mut variant| {
                    variant.product.base_product_id = Some(base_prod_id);

                    variant
                });

                for variant in variants {
                    check_vendor_code(&*stores_repo, store_id, &variant.product.vendor_code)?;
                    // create variant
                    let product = products_repo.create((variant.product, base_prod.currency).into())?;
                    // create attributes values for variant
                    create_product_attributes_values(
                        &*products_repo,
                        &*prod_attr_repo,
                        &*attr_repo,
                        &*custom_attributes_repo,
                        &*attribute_values_repo,
                        &product,
                        base_prod.id,
                        variant.attributes,
                    )?;
                }

                // Save selected_attributes
                selected_attributes
                    .into_iter()
                    .map(|attribute_id| {
                        let new_custom_attribute = NewCustomAttribute::new(attribute_id, base_prod.id);
                        custom_attributes_repo.create(new_custom_attribute)
                    })
                    .collect::<RepoResult<Vec<_>>>()?;

                Ok(base_prod)
            })
            .map_err(|e| {
                e.context("Service BaseProduct, create with variants and attributes endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Updates specific product
    fn update_base_product(&self, base_product_id: BaseProductId, payload: UpdateBaseProduct) -> ServiceFuture<BaseProduct> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
            let product_attrs_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
            conn.transaction::<BaseProduct, FailureError, _>(move || {
                let old_prod = base_products_repo.find(base_product_id, Visibility::Active)?;
                if let Some(old_prod) = old_prod {
                    // validate
                    validate_base_product_update(&*base_products_repo, old_prod.store_id.clone(), old_prod.id, &payload)?;
                    let updated_prod = base_products_repo.update(base_product_id, payload.clone())?;
                    if let Some(new_cat_id) = payload.category_id {
                        // updating product categories of the store
                        if old_prod.category_id != new_cat_id {
                            let _ = after_base_product_category_update(&*products_repo, &*product_attrs_repo, base_product_id);
                        }
                        let _ = update_product_categories(&*stores_repo, old_prod.store_id, old_prod.category_id, new_cat_id)?;
                    }

                    if let Some(currency) = payload.currency {
                        // updating currency of base_products variants
                        products_repo.update_currency(currency, updated_prod.id)?;
                    }

                    match updated_prod.status {
                        ModerationStatus::Decline => base_products_repo.set_moderation_status(updated_prod.id, ModerationStatus::Draft),
                        _ => Ok(updated_prod),
                    }
                } else {
                    Err(Error::NotFound.into())
                }
            })
            .map_err(|e| e.context("Service BaseProduct, update endpoint error occurred.").into())
        })
    }

    /// Find by cart
    fn find_by_cart(&self, cart: Vec<CartProduct>) -> ServiceFuture<Vec<StoreWithBaseProducts>> {
        let user_id = self.dynamic_context.user_id;
        let currency = self.dynamic_context.currency;
        let fiat_currency = self.dynamic_context.fiat_currency;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            {
                let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let products_ids = cart.into_iter().map(|cart_product| cart_product.product_id).collect();
                //find products
                let products = products_repo.find_many(products_ids)?;
                let mut group_by_base_product_id = BTreeMap::<BaseProductId, Vec<RawProduct>>::default();
                for product in products {
                    let p = group_by_base_product_id.entry(product.base_product_id).or_insert_with(Vec::new);
                    p.push(product);
                }
                //find base_products with products
                let mut base_products = group_by_base_product_id
                    .into_iter()
                    .map(|(base_product_id, products)| {
                        let base_product = base_products_repo.find(base_product_id, Visibility::Published)?;
                        let products = products.into_iter().map(Product::from).collect();

                        if let Some(base_product) = base_product {
                            Ok(BaseProductWithVariants::new(base_product, products))
                        } else {
                            Err(format_err!("Not found such base product id : {}", base_product_id)
                                .context(Error::NotFound)
                                .into())
                        }
                    })
                    .collect::<RepoResult<Vec<BaseProductWithVariants>>>()?;

                let latest_currencies = currency_exchange.get_latest()?;
                calculate_base_products_customer_price(&mut base_products, latest_currencies, currency, fiat_currency);

                let mut group_by_store_id = BTreeMap::<StoreId, Vec<BaseProductWithVariants>>::default();
                for base_product_with_variants in base_products {
                    let bp = group_by_store_id
                        .entry(base_product_with_variants.base_product.store_id)
                        .or_insert_with(Vec::new);
                    bp.push(base_product_with_variants);
                }
                //find stores with base_products with products
                group_by_store_id
                    .into_iter()
                    .map(|(store_id, base_products)| {
                        let store = stores_repo.find(store_id, Visibility::Published)?;
                        if let Some(store) = store {
                            Ok(StoreWithBaseProducts::new(store, base_products))
                        } else {
                            Err(format_err!("Not found such store id : {}", store_id)
                                .context(Error::NotFound)
                                .into())
                        }
                    })
                    .collect::<RepoResult<Vec<StoreWithBaseProducts>>>()
            }
            .map_err(|e| e.context("Service BaseProduct, find_by_cart endpoint error occurred.").into())
        })
    }

    /// Search base products limited by `from`, `skip` and `count` parameters
    fn moderator_search_base_product(
        &self,
        from: Option<BaseProductId>,
        skip: i64,
        count: i64,
        term: ModeratorBaseProductSearchTerms,
    ) -> ServiceFuture<ModeratorBaseProductSearchResults> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!(
            "Searching for base_products (from id: {:?}, skip: {}, count: {}) with payload: {:?}",
            from, skip, count, term
        );

        let pagination_params = PaginationParams {
            direction: Direction::Reverse,
            limit: count,
            ordering: Ordering::Descending,
            skip,
            start: from.filter(|id| id.0 > 0),
        };

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&conn, user_id);
            base_products_repo
                .moderator_search(pagination_params, term)
                .map_err(|e: FailureError| {
                    e.context("Service `base_products`, `moderator_search` endpoint error occurred.")
                        .into()
                })
        })
    }

    /// Set moderation status for base_product_ids
    fn set_moderation_status_base_products(
        &self,
        base_product_ids: Vec<BaseProductId>,
        status: ModerationStatus,
    ) -> ServiceFuture<Vec<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        debug!("Set moderation status {} for base_products {:?}", status, &base_product_ids);

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&conn, user_id);
            base_products_repo
                .set_moderation_statuses(base_product_ids, status)
                .map_err(|e: FailureError| {
                    e.context("Service base_products, set_moderation_status_base_products endpoint error occurred.")
                        .into()
                })
        })
    }

    /// Set moderation status for base_product_id
    fn set_moderation_status_base_product(&self, base_product_id: BaseProductId, status: ModerationStatus) -> ServiceFuture<BaseProduct> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        info!("Set moderation status {} for base_product {}", status, base_product_id);

        self.spawn_on_pool(move |conn| {
            {
                let base_products_repo = repo_factory.create_base_product_repo(&conn, user_id);
                let base_product = base_products_repo.find(base_product_id, Visibility::Active)?;

                let current_status = match base_product {
                    Some(value) => value.status,
                    None => return Err(Error::NotFound.into()),
                };

                if check_change_status(current_status, status) {
                    base_products_repo.set_moderation_status(base_product_id, status)
                } else {
                    Err(format_err!("Base product status: {} not valid for set", status)
                        .context(Error::Validate(
                            validation_errors!({"base_products": ["base_products" => "Base product new status is not valid"]}),
                        ))
                        .into())
                }
            }
            .map_err(|e: FailureError| {
                e.context("Service base_products, set_moderation_status_base_product endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Send base product to moderation from store manager
    fn send_base_product_to_moderation(&self, base_product_id: BaseProductId) -> ServiceFuture<BaseProduct> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        info!("Send base product: {} to moderation", base_product_id);

        self.spawn_on_pool(move |conn| {
            {
                let base_products_repo = repo_factory.create_base_product_repo(&conn, user_id);
                let base_product = base_products_repo.find(base_product_id, Visibility::Active)?;

                let status = match base_product {
                    Some(value) => value.status,
                    None => return Err(Error::NotFound.into()),
                };

                if check_change_status(status, ModerationStatus::Moderation) {
                    base_products_repo.set_moderation_status(base_product_id, ModerationStatus::Moderation)
                } else {
                    Err(
                        format_err!("Base product with id: {}, cannot be sent to moderation", base_product_id)
                            .context(Error::Validate(
                                validation_errors!({"base_products": ["base_products" => "Base product can not be sent to moderation"]}),
                            ))
                            .into(),
                    )
                }
            }
            .map_err(|e: FailureError| {
                e.context("Service base_products, send_base_product_to_moderation endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Hide base product from search. For store manager
    fn set_base_product_moderation_status_draft(&self, base_product_id: BaseProductId) -> ServiceFuture<BaseProduct> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        info!("Hide base product: {}", base_product_id);

        self.spawn_on_pool(move |conn| {
            {
                let base_products_repo = repo_factory.create_base_product_repo(&conn, user_id);

                set_base_product_moderation_status_draft(&*base_products_repo, base_product_id)
            }
            .map_err(|e: FailureError| {
                e.context("Service base_products, set_base_product_moderation_status_draft endpoint error occurred.")
                    .into()
            })
        })
    }

    // Check that you can change the moderation status
    fn validate_change_moderation_status_base_product(
        &self,
        base_product_id: BaseProductId,
        status: ModerationStatus,
    ) -> ServiceFuture<bool> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        info!("Check change moderation status base product: {}", base_product_id);

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&conn, user_id);
            let base_product = base_products_repo.find(base_product_id, Visibility::Active)?;

            let current_status = match base_product {
                Some(value) => value.status,
                None => return Err(Error::NotFound.into()),
            };

            Ok(check_change_status(current_status, status))
        })
    }

    fn flatten_categories(&self, options: Option<ProductsSearchOptions>) -> ServiceFuture<Option<ProductsSearchOptions>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        if let Some(mut options) = options {
            let category_id = options.category_id;
            if let Some(category_id) = category_id {
                self.spawn_on_pool(move |conn| {
                    let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                    let cat = categories_repo.find(category_id)?;
                    if let Some(cat) = cat {
                        let cats_ids = if cat.children.is_empty() {
                            vec![category_id]
                        } else {
                            get_all_children_till_the_end(cat).into_iter().map(|c| c.id).collect()
                        };
                        options.categories_ids = Some(cats_ids);
                    }
                    Ok(Some(options))
                })
            } else {
                Box::new(future::ok(Some(options)))
            }
        } else {
            Box::new(future::ok(None))
        }
    }

    fn remove_non_third_level_categories(&self, options: Option<ProductsSearchOptions>) -> ServiceFuture<Option<ProductsSearchOptions>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        if let Some(mut options) = options {
            let category_id = options.category_id;
            if let Some(category_id) = category_id {
                self.spawn_on_pool(move |conn| {
                    let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                    let cat = categories_repo.find(category_id)?;
                    if let Some(cat) = cat {
                        let cats_ids = if cat.children.is_empty() { Some(vec![category_id]) } else { None };
                        options.categories_ids = cats_ids;
                    }
                    Ok(Some(options))
                })
            } else {
                Box::new(future::ok(Some(options)))
            }
        } else {
            Box::new(future::ok(None))
        }
    }

    fn create_currency_map(&self, options: Option<ProductsSearchOptions>) -> ServiceFuture<Option<ProductsSearchOptions>> {
        let repo_factory = self.static_context.repo_factory.clone();
        let fiat_currency = self.dynamic_context.fiat_currency;
        let user_id = self.dynamic_context.user_id;

        if let Some(mut options) = options {
            self.spawn_on_pool(move |conn| {
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                // We need to reconvert each currency into fiat currency to search in elastic
                let currencies_map = currency_exchange.get_latest()?.map(|currencies| {
                    let mut currencies_map = HashMap::new();
                    if let Some(fiat) = currencies.data.get(&fiat_currency) {
                        for cur in fiat.keys() {
                            let value = if let Some(cur_hash) = currencies.data.get(&cur) {
                                cur_hash.get(&fiat_currency).map(|c| c.0).unwrap_or(1.0)
                            } else {
                                1.0
                            };
                            currencies_map.insert(*cur, ExchangeRate(value));
                        }
                    }
                    currencies_map
                });
                options.currency_map = currencies_map;
                Ok(Some(options))
            })
        } else {
            Box::new(future::ok(None))
        }
    }

    fn get_base_product_by_slug(
        &self,
        store_identifier: StoreIdentifier,
        base_product_slug: BaseProductSlug,
        visibility: Option<Visibility>,
    ) -> ServiceFuture<Option<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let visibility = visibility.unwrap_or(Visibility::Published);

        debug!(
            "Get base product by slug = {:?} with visibility = {:?}",
            base_product_slug, visibility
        );

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            let store_id = match store_identifier {
                StoreIdentifier::Id(store_id) => store_id,
                StoreIdentifier::Slug(store_slug) => stores_repo
                    .find_by_slug(store_slug.clone(), visibility)?
                    .map(|store| store.id)
                    .ok_or(format_err!("Store with slug \"{}\" not found", store_slug).context(Error::NotFound))?,
            };
            base_products_repo
                .find_by_slug(store_id, base_product_slug, visibility)
                .map_err(|e| {
                    e.context("Service BaseProduct, get_base_product_by_slug endpoint error occurred.")
                        .into()
                })
        })
    }

    fn get_base_product_by_slug_with_views_update(
        &self,
        store_identifier: StoreIdentifier,
        base_product_slug: BaseProductSlug,
    ) -> ServiceFuture<Option<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            let store_id = match store_identifier {
                StoreIdentifier::Id(store_id) => store_id,
                StoreIdentifier::Slug(store_slug) => stores_repo
                    .find_by_slug(store_slug.clone(), Visibility::Published)?
                    .map(|store| store.id)
                    .ok_or(format_err!("Store with slug {} not found", store_slug))?,
            };
            base_products_repo.update_views_by_slug(store_id, base_product_slug).map_err(|e| {
                e.context("Service BaseProduct, get_base_product_by_slug_with_views_update endpoint error occurred.")
                    .into()
            })
        })
    }

    /// Replace category in all base products
    fn replace_category(&self, payload: CategoryReplacePayload) -> ServiceFuture<Vec<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        info!("Replace category in base products");

        self.spawn_on_pool(move |conn| {
            {
                let base_products_repo = repo_factory.create_base_product_repo(&conn, user_id);
                let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);

                conn.transaction::<Vec<BaseProduct>, FailureError, _>(move || {
                    let update_products = base_products_repo.replace_category(payload.clone())?;

                    for base_product in update_products.iter() {
                        let _ = update_product_categories(
                            &*stores_repo,
                            base_product.store_id,
                            payload.current_category,
                            payload.new_category,
                        )?;
                    }

                    Ok(update_products)
                })
            }
            .map_err(|e: FailureError| e.context("Service base_products, replace_category endpoint error occurred.").into())
        })
    }

    /// Check that you can update base product
    fn validate_update_base_product(&self, base_product_id: BaseProductId) -> ServiceFuture<bool> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        info!("Check update base product: {}", base_product_id);

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&conn, user_id);
            let base_product = base_products_repo.find(base_product_id, Visibility::Active)?;

            let current_status = match base_product {
                Some(value) => value.status,
                None => return Err(Error::NotFound.into()),
            };

            Ok(check_can_update_by_status(current_status))
        })
    }
}

fn after_base_product_category_update(
    products_repo: &ProductsRepo,
    product_attrs_repo: &ProductAttrsRepo,
    base_prod_id: BaseProductId,
) -> Result<(), FailureError> {
    product_attrs_repo.delete_by_base_product_id(base_prod_id)?;
    let mut all_products = products_repo.find_with_base_id(base_prod_id)?;
    all_products.sort_by_key(|p| p.created_at);
    //delete all except the first one
    for product in all_products.iter().skip(1) {
        products_repo.deactivate(product.id)?;
    }
    Ok(())
}

fn validate_base_product(base_products_repo: &BaseProductsRepo, payload: &NewBaseProduct) -> Result<(), FailureError> {
    if let Some(base_product_slug) = payload.slug.clone() {
        let base_product_with_same_slug =
            base_products_repo.find_by_slug(payload.store_id, BaseProductSlug(base_product_slug.clone()), Visibility::Active)?;
        if base_product_with_same_slug.is_some() {
            return Err(format_err!(
                "Base product with slug {} in store with id {} already exists",
                base_product_slug,
                payload.store_id
            )
            .context(Error::Validate(
                validation_errors!({"base_products": ["base_products" => "Base product with such slug already exists"]}),
            ))
            .into());
        }
    }
    Ok(())
}

fn validate_base_product_update(
    base_products_repo: &BaseProductsRepo,
    store_id: StoreId,
    base_product_id: BaseProductId,
    payload: &UpdateBaseProduct,
) -> Result<(), FailureError> {
    if let Some(base_product_slug) = payload.slug.clone() {
        let base_product_with_same_slug =
            base_products_repo.find_by_slug(store_id, BaseProductSlug(base_product_slug.clone()), Visibility::Active)?;
        if let Some(base_product_with_same_slug) = base_product_with_same_slug {
            if base_product_with_same_slug.id != base_product_id {
                return Err(format_err!(
                    "Base product with slug {} in store with id {} already exists",
                    base_product_slug,
                    store_id
                )
                .context(Error::Validate(
                    validation_errors!({"base_products": ["base_products" => "Base product with such slug already exists"]}),
                ))
                .into());
            }
        }
    }
    Ok(())
}

fn enrich_new_base_product(stores_repo: &StoresRepo, new_base_product: &mut NewBaseProduct) -> Result<(), FailureError> {
    let store = stores_repo
        .find(new_base_product.store_id, Visibility::Active)?
        .ok_or_else(|| format_err!("There is no store with id {}", new_base_product.store_id).context(Error::NotFound))?;
    new_base_product.store_status = Some(store.status);
    Ok(())
}

fn calculate_base_products_customer_price(
    base_products: &mut [BaseProductWithVariants],
    latest_currencies: Option<CurrencyExchange>,
    crypto_currency: Currency,
    fiat_currency: Currency,
) {
    for base_product in base_products {
        let currency = base_product.base_product.currency;
        let currencies_map = latest_currencies
            .as_ref()
            .and_then(|all_rates| all_rates.data.get(&currency).cloned());
        for mut variant in &mut base_product.variants {
            variant.customer_price = calculate_customer_price(&variant.product, &currencies_map, crypto_currency, fiat_currency);
        }
    }
}

fn get_path_to_searched_category(searched_category: Option<Category>, root: Category) -> Category {
    if let Some(searched_category) = searched_category {
        if searched_category.children.is_empty() {
            remove_unused_categories(root, &[searched_category.parent_id.unwrap_or_default()])
        } else {
            let new_cat = remove_unused_categories(root, &[searched_category.id]);
            clear_child_categories(new_cat, searched_category.level + 1)
        }
    } else {
        root
    }
}

fn get_attribute_filters(el_products: Vec<ElasticProduct>) -> Option<Vec<AttributeFilter>> {
    let mut equal_attrs = HashMap::<i32, HashSet<String>>::default();
    let mut range_attrs = HashMap::<i32, RangeFilter>::default();

    for product in el_products {
        for variant in product.variants {
            for attr_value in variant.attrs {
                if let Some(value) = attr_value.str_val {
                    let equal = equal_attrs.entry(attr_value.attr_id).or_insert_with(HashSet::<String>::default);
                    equal.insert(value);
                }
                if let Some(value) = attr_value.float_val {
                    let range = range_attrs.entry(attr_value.attr_id).or_insert_with(RangeFilter::default);
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

    Some(eq_filters.chain(range_filters).collect())
}

fn get_first_level_category(third_level_category_id: CategoryId, root: Category) -> RepoResult<Category> {
    root.children
        .into_iter()
        .find(|cat_child| get_parent_category(&cat_child, third_level_category_id, 2).is_some())
        .ok_or_else(|| {
            format_err!("There is no such 3rd level category in db - {}", third_level_category_id)
                .context(Error::NotFound)
                .into()
        })
}

/// Add product categories of the store
fn add_product_categories(
    stores_repo: &StoresRepo,
    categories_repo: &CategoriesRepo,
    store_id_arg: StoreId,
    category_id_arg: CategoryId,
) -> RepoResult<()> {
    let store = stores_repo.find(store_id_arg, Visibility::Active)?;
    if let Some(store) = store {
        let category_root = categories_repo.get_all_categories()?;
        let cat = get_first_level_category(category_id_arg, category_root)?;
        let service_update_store = ServiceUpdateStore::add_category_to_product_categories(store.product_categories.clone(), cat.id);
        let _ = stores_repo.update_service_fields(store.id, service_update_store)?;
    }

    Ok(())
}

/// Update product categories of store
fn update_product_categories(
    stores_repo: &StoresRepo,
    store_id_arg: StoreId,
    old_category: CategoryId,
    new_category: CategoryId,
) -> RepoResult<()> {
    let store = stores_repo.find(store_id_arg, Visibility::Active)?;
    if let Some(store) = store {
        let service_update_store =
            ServiceUpdateStore::update_product_categories(store.product_categories.clone(), old_category, new_category);
        let _ = stores_repo.update_service_fields(store.id, service_update_store)?;
    }

    Ok(())
}

pub fn set_base_product_moderation_status_draft(
    base_products_repo: &BaseProductsRepo,
    base_product_id: BaseProductId,
) -> RepoResult<BaseProduct> {
    let base_product = base_products_repo.find(base_product_id, Visibility::Active)?;

    let status = match base_product {
        Some(value) => value.status,
        None => return Err(Error::NotFound.into()),
    };

    if check_change_status(status, ModerationStatus::Draft) {
        base_products_repo.set_moderation_status(base_product_id, ModerationStatus::Draft)
    } else {
        Err(format_err!(
            "Base product with id: {}, cannot be hided when the store in status: {}",
            base_product_id,
            status
        )
        .context(Error::Validate(
            validation_errors!({"base_products": ["base_products" => "Base product cannot be hided"]}),
        ))
        .into())
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use serde_json;
    use tokio_core::reactor::Core;
    use uuid::Uuid;

    use stq_static_resources::Currency;
    use stq_types::*;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    pub fn create_new_base_product(name: &str) -> NewBaseProduct {
        NewBaseProduct {
            name: serde_json::from_str(name).unwrap(),
            store_id: StoreId(1),
            short_description: serde_json::from_str("{}").unwrap(),
            long_description: None,
            seo_title: None,
            seo_description: None,
            currency: Currency::STQ,
            category_id: CategoryId(3),
            slug: Some("slug".to_string()),
            uuid: Uuid::new_v4(),
            length_cm: Some(60),
            width_cm: Some(40),
            height_cm: Some(20),
            weight_g: Some(150),
            store_status: None,
        }
    }

    pub fn create_update_base_product(name: &str) -> UpdateBaseProduct {
        UpdateBaseProduct {
            name: Some(serde_json::from_str(name).unwrap()),
            short_description: Some(serde_json::from_str("{}").unwrap()),
            long_description: None,
            seo_title: None,
            seo_description: None,
            currency: Some(Currency::STQ),
            category_id: None,
            slug: None,
            length_cm: None,
            width_cm: None,
            height_cm: None,
            weight_g: None,
        }
    }

    #[test]
    fn test_get_base_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_base_product(BaseProductId(1), Some(Visibility::Active));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, BaseProductId(1));
    }

    #[test]
    fn test_list() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.list_base_products(BaseProductId(1), 5, Some(Visibility::Active));
        let result = core.run(work).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_create_base_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_base_product = create_new_base_product(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.create_base_product(new_base_product);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, MOCK_BASE_PRODUCT_ID);
    }

    #[test]
    fn test_update() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let new_base_product = create_update_base_product(MOCK_BASE_PRODUCT_NAME_JSON);
        let work = service.update_base_product(BaseProductId(1), new_base_product);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, BaseProductId(1));
        assert_eq!(result.id, MOCK_BASE_PRODUCT_ID);
    }

    #[test]
    fn test_deactivate() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.deactivate_base_product(BaseProductId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.id, BaseProductId(1));
        assert_eq!(result.is_active, false);
    }
}
