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
use stq_types::{BaseProductId, ExchangeRate, ProductId, StoreId};

use super::types::ServiceFuture;
use elastic::{ProductsElastic, ProductsElasticImpl};
use errors::Error;
use models::*;
use repos::clear_child_categories;
use repos::get_all_children_till_the_end;
use repos::get_parent_category;
use repos::remove_unused_categories;
use repos::{RepoResult, ReposFactory};
use services::create_product_attributes_values;
use services::Service;

const MAX_PRODUCTS_SEARCH_COUNT: i32 = 1000;

pub trait BaseProductsService {
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
    /// Find product by dicount pattern limited by `count` and `offset` parameters
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
    fn get_base_product(&self, base_product_id: BaseProductId) -> ServiceFuture<Option<BaseProduct>>;
    /// Returns base product by ID with update views
    fn get_base_product_with_views_update(&self, base_product_id: BaseProductId) -> ServiceFuture<Option<BaseProduct>>;
    /// Returns base_product by product ID
    fn get_base_product_by_product(&self, product_id: ProductId) -> ServiceFuture<Option<BaseProductWithVariants>>;
    /// Deactivates specific product
    fn deactivate_base_product(&self, base_product_id: BaseProductId) -> ServiceFuture<BaseProduct>;
    /// Creates base product
    fn create_base_product(&self, payload: NewBaseProduct) -> ServiceFuture<BaseProduct>;
    /// Creates base product with variants
    fn create_base_product_with_variant(&self, payload: NewBaseProductWithVariant) -> ServiceFuture<BaseProduct>;
    /// Lists base products limited by `from` and `count` parameters
    fn list_base_products(&self, from: BaseProductId, count: i32) -> ServiceFuture<Vec<BaseProduct>>;
    /// Returns list of base_products by store id and exclude base_product_id_arg, limited by 10
    fn get_base_products_of_the_store(
        &self,
        store_id: StoreId,
        skip_base_product_id: Option<BaseProductId>,
        from: BaseProductId,
        count: i32,
    ) -> ServiceFuture<Vec<BaseProduct>>;
    /// Updates base product
    fn update_base_product(&self, base_product_id: BaseProductId, payload: UpdateBaseProduct) -> ServiceFuture<BaseProduct>;
    /// Cart
    fn find_by_cart(&self, cart: Vec<CartProduct>) -> ServiceFuture<Vec<StoreWithBaseProducts>>;
    /// Search base products limited by `from` and `count` parameters
    fn moderator_search_base_product(
        &self,
        from: BaseProductId,
        count: i64,
        term: ModeratorBaseProductSearchTerms,
    ) -> ServiceFuture<Vec<BaseProduct>>;
    /// Set moderation status for base_product_ids
    fn set_moderation_status_base_product(
        &self,
        base_product_ids: Vec<BaseProductId>,
        status: ModerationStatus,
    ) -> ServiceFuture<Vec<BaseProduct>>;
    /// Flattens categories
    fn flatten_categories(&self, options: Option<ProductsSearchOptions>) -> ServiceFuture<Option<ProductsSearchOptions>>;
    /// Remove categories not 3rd level
    fn remove_non_third_level_categories(&self, options: Option<ProductsSearchOptions>) -> ServiceFuture<Option<ProductsSearchOptions>>;
    /// Create currency map
    fn create_currency_map(&self, options: Option<ProductsSearchOptions>) -> ServiceFuture<Option<ProductsSearchOptions>>;
}
impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > BaseProductsService for Service<T, M, F>
{
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
        let address = self.static_context.config.server.elastic.clone();
        let products_el = ProductsElasticImpl::new(client_handle, address);
        let service = self.clone();
        Box::new(
            self.flatten_categories(search_product.options.clone())
                .and_then(move |options| self.create_currency_map(options))
                .and_then(move |options| {
                    let currency_map = options.clone().and_then(|o| o.currency_map);
                    search_product.options = options;
                    products_el
                        .search_by_name(search_product, count, offset)
                        .map(|el_products| (el_products, currency_map))
                }).and_then({
                    move |(el_products, currency_map)| {
                        service.spawn_on_pool(move |conn| {
                            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                            let mut base_products = base_products_repo.convert_from_elastic(el_products)?;
                            recalc_currencies(&mut base_products, currency_map, currency);
                            Ok(base_products)
                        })
                    }
                }).map_err(|e| {
                    e.context("Service BaseProduct, search_base_products_by_name endpoint error occured.")
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
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            {
                let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let mut base_products = base_products_repo.most_viewed(search_product, count, offset)?;
                let currencies_map = currency_exchange.get_exchange_for_currency(currency)?;
                recalc_currencies(&mut base_products, currencies_map, currency);
                Ok(base_products)
            }.map_err(|e: FailureError| {
                e.context("Service BaseProduct, search_base_products_most_viewed endpoint error occured.")
                    .into()
            })
        })
    }

    /// Find product by dicount pattern limited by `count` and `offset` parameters
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
        let repo_factory = self.static_context.repo_factory.clone();
        Box::new(
            self.flatten_categories(search_product.options.clone())
                .and_then(move |options| {
                    search_product.options = options;
                    products_el.search_most_discount(search_product, count, offset)
                }).and_then({
                    move |el_products| {
                        self.spawn_on_pool(move |conn| {
                            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                            let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                            let mut base_products = base_products_repo.convert_from_elastic(el_products)?;
                            let currencies_map = currency_exchange.get_exchange_for_currency(currency)?;
                            recalc_currencies(&mut base_products, currencies_map, currency);
                            Ok(base_products)
                        })
                    }
                }).map_err(|e| {
                    e.context("Service BaseProduct, search_base_products_most_discount endpoint error occured.")
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
            e.context("Service BaseProduct, base_products_auto_complete endpoint error occured.")
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
                }).map_err(|e| {
                    e.context("Service BaseProduct, search_base_products_filters_price endpoint error occured.")
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
                }).map_err(|e| {
                    e.context("Service BaseProduct, search_base_products_filters_count endpoint error occured.")
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
                    let root = categories_repo.get_all_categories()?;
                    if let Some(category_id) = category_id {
                        let cat = categories_repo.find(category_id)?;
                        Ok(get_path_to_searched_category(cat, root))
                    } else {
                        Ok(root)
                    }
                }.map_err(|e: FailureError| {
                    e.context("Service BaseProduct, search_base_products_filters_category endpoint with empty name option error occured.")
                        .into()
                })
            })
        } else {
            Box::new(products_el.aggregate_categories(search_prod.name.clone()).and_then(move |cats| {
                self.spawn_on_pool(move |conn| {
                    {
                        let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                        let category = categories_repo.get_all_categories()?;
                        let new_cat = remove_unused_categories(category, &cats, 2);
                        Ok(new_cat)
                    }.map_err(|e: FailureError| {
                        e.context("Service BaseProduct, search_base_products_filters_category endpoint with name aggregation in elastic error occured.")
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
                }).map_err(|e| {
                    e.context("Service BaseProduct, search_base_products_attributes endpoint error occured.")
                        .into()
                }),
        )
    }

    /// Returns product by ID
    fn get_base_product(&self, base_product_id: BaseProductId) -> ServiceFuture<Option<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            base_products_repo
                .find(base_product_id)
                .map_err(|e| e.context("Service BaseProduct, get_base_product endpoint error occured.").into())
        })
    }

    /// Returns base product by ID with update views
    fn get_base_product_with_views_update(&self, base_product_id: BaseProductId) -> ServiceFuture<Option<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            base_products_repo.update_views(base_product_id).map_err(|e| {
                e.context("Service BaseProduct, get_base_product_with_views_update endpoint error occured.")
                    .into()
            })
        })
    }

    /// Returns base_product by product ID
    fn get_base_product_by_product(&self, product_id: ProductId) -> ServiceFuture<Option<BaseProductWithVariants>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        let currency = self.dynamic_context.currency;

        self.spawn_on_pool(move |conn| {
            {
                let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let product = products_repo.find(product_id)?;
                if let Some(product) = product {
                    let base_product = base_products_repo
                        .find(product.base_product_id)
                        .map(|base_product| base_product.map(|base_product| BaseProductWithVariants::new(base_product, vec![product])))?;
                    if let Some(base_product) = base_product {
                        let currencies_map = currency_exchange.get_exchange_for_currency(currency)?;
                        let mut base_products = vec![base_product];
                        recalc_currencies(&mut base_products, currencies_map, currency);
                        return Ok(base_products.pop());
                    };
                }
                Ok(None)
            }.map_err(|e: FailureError| {
                e.context("Service BaseProduct, get_base_product_by_product endpoint error occured.")
                    .into()
            })
        })
    }

    /// Deactivates specific base product
    fn deactivate_base_product(&self, base_product_id: BaseProductId) -> ServiceFuture<BaseProduct> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            {
                let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                let prod = base_products_repo.deactivate(base_product_id)?;
                // update product categories of the store
                let store = stores_repo.find(prod.store_id)?;
                if let Some(store) = store {
                    let category_root = categories_repo.get_all_categories()?;
                    let cat = get_first_level_category(prod.category_id, category_root)?;
                    let update_store = UpdateStore::delete_category_from_product_categories(store.product_categories.clone(), cat.id);
                    stores_repo.update(store.id, update_store)?;
                };
                Ok(prod)
            }.map_err(|e: FailureError| {
                e.context("Service BaseProduct, deactivate_base_product endpoint error occured.")
                    .into()
            })
        })
    }

    /// Lists base products limited by `from` and `count` parameters
    fn list_base_products(&self, from: BaseProductId, count: i32) -> ServiceFuture<Vec<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            base_products_repo
                .list(from, count)
                .map_err(|e| e.context("Service BaseProduct, list endpoint error occured.").into())
        })
    }

    /// Returns list of base_products by store id and exclude skip_base_product_id, limited by from and count
    fn get_base_products_of_the_store(
        &self,
        store_id: StoreId,
        skip_base_product_id: Option<BaseProductId>,
        from: BaseProductId,
        count: i32,
    ) -> ServiceFuture<Vec<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            base_products_repo
                .get_products_of_the_store(store_id, skip_base_product_id, from, count)
                .map_err(|e| {
                    e.context("Service BaseProduct, get_products_of_the_store endpoint error occured.")
                        .into()
                })
        })
    }

    /// Creates new base product
    fn create_base_product(&self, payload: NewBaseProduct) -> ServiceFuture<BaseProduct> {
        let user_id = self.dynamic_context.user_id;

        let repo_factory = self.static_context.repo_factory.clone();
        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            conn.transaction::<(BaseProduct), FailureError, _>(move || {
                // create base_product
                let prod = base_products_repo.create(payload)?;
                // update product categories of the store
                let store = stores_repo.find(prod.store_id)?;
                if let Some(store) = store {
                    let category_root = categories_repo.get_all_categories()?;
                    let cat = get_first_level_category(prod.category_id, category_root)?;
                    let update_store = UpdateStore::add_category_to_product_categories(store.product_categories.clone(), cat.id);
                    stores_repo.update(store.id, update_store)?;
                }
                Ok(prod)
            }).map_err(|e| e.context("Service BaseProduct, create endpoint error occured.").into())
        })
    }

    /// Creates base product with variants
    fn create_base_product_with_variant(&self, payload: NewBaseProductWithVariant) -> ServiceFuture<BaseProduct> {
        let user_id = self.dynamic_context.user_id;

        let repo_factory = self.static_context.repo_factory.clone();
        let NewBaseProductWithVariant {
            new_base_product,
            variant,
            selected_attributes,
        } = payload;

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
            let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
            let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
            let products_repo = repo_factory.create_product_repo(&*conn, user_id);
            let prod_attr_repo = repo_factory.create_product_attrs_repo(&*conn, user_id);
            let attr_repo = repo_factory.create_attributes_repo(&*conn, user_id);
            let custom_attributes_repo = repo_factory.create_custom_attributes_repo(&*conn, user_id);

            conn.transaction::<(BaseProduct), FailureError, _>(move || {
                // create base_product
                let base_prod = base_products_repo.create(new_base_product)?;

                // update product categories of the store
                let store = stores_repo.find(base_prod.store_id)?;
                if let Some(store) = store {
                    let category_root = categories_repo.get_all_categories()?;
                    let cat = get_first_level_category(base_prod.category_id, category_root)?;
                    let update_store = UpdateStore::add_category_to_product_categories(store.product_categories.clone(), cat.id);
                    stores_repo.update(store.id, update_store)?;
                }

                // Create variant
                let product = products_repo.create((variant.product, base_prod.currency).into())?;
                // Create attributes values for variant
                create_product_attributes_values(
                    &*prod_attr_repo,
                    &*attr_repo,
                    &*custom_attributes_repo,
                    &product,
                    base_prod.id,
                    variant.attributes,
                )?;

                // Save selected_attributes
                let _ = selected_attributes
                    .into_iter()
                    .map(|attribute_id| {
                        let new_custom_attribute = NewCustomAttribute::new(attribute_id, base_prod.id);
                        custom_attributes_repo.create(new_custom_attribute)
                    }).collect::<RepoResult<Vec<_>>>()?;

                Ok(base_prod)
            }).map_err(|e| {
                e.context("Service BaseProduct, create with variant and attributes endpoint error occured.")
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
            conn.transaction::<(BaseProduct), FailureError, _>(move || {
                let old_prod = base_products_repo.find(base_product_id)?;
                if let Some(old_prod) = old_prod {
                    let payload = payload.reset_moderation_status();
                    let updated_prod = base_products_repo.update(base_product_id, payload.clone())?;
                    if let Some(new_cat_id) = payload.category_id {
                        // updating product categories of the store
                        let old_cat_id = old_prod.category_id;
                        let old_prod_store_id = old_prod.store_id;
                        let store = stores_repo.find(old_prod_store_id)?;
                        if let Some(store) = store {
                            let update_store =
                                UpdateStore::update_product_categories(store.product_categories.clone(), old_cat_id, new_cat_id);
                            stores_repo.update(store.id, update_store)?;
                        }
                    } else if let Some(currency) = payload.currency {
                        // updating currency of base_products variants
                        products_repo.update_currency(currency, updated_prod.id)?;
                    }
                    Ok(updated_prod)
                } else {
                    Err(Error::NotFound.into())
                }
            }).map_err(|e| e.context("Service BaseProduct, update endpoint error occured.").into())
        })
    }

    /// Find by cart
    fn find_by_cart(&self, cart: Vec<CartProduct>) -> ServiceFuture<Vec<StoreWithBaseProducts>> {
        let user_id = self.dynamic_context.user_id;
        let currency = self.dynamic_context.currency;
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
                let mut group_by_base_product_id = BTreeMap::<BaseProductId, Vec<Product>>::default();
                for product in products {
                    let p = group_by_base_product_id.entry(product.base_product_id).or_insert_with(Vec::new);
                    p.push(product);
                }
                //find base_products with products
                let mut base_products = group_by_base_product_id
                    .into_iter()
                    .map(|(base_product_id, products)| {
                        let base_product = base_products_repo.find(base_product_id)?;
                        if let Some(base_product) = base_product {
                            Ok(BaseProductWithVariants::new(base_product, products))
                        } else {
                            Err(format_err!("Not found such base product id : {}", base_product_id)
                                .context(Error::NotFound)
                                .into())
                        }
                    }).collect::<RepoResult<Vec<BaseProductWithVariants>>>()?;
                let currencies_map = currency_exchange.get_exchange_for_currency(currency)?;
                recalc_currencies(&mut base_products, currencies_map, currency);
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
                        let store = stores_repo.find(store_id)?;
                        if let Some(store) = store {
                            Ok(StoreWithBaseProducts::new(store, base_products))
                        } else {
                            Err(format_err!("Not found such store id : {}", store_id)
                                .context(Error::NotFound)
                                .into())
                        }
                    }).collect::<RepoResult<Vec<StoreWithBaseProducts>>>()
            }.map_err(|e| e.context("Service BaseProduct, find_by_cart endpoint error occured.").into())
        })
    }

    /// Search base products limited by `from` and `count` parameters
    fn moderator_search_base_product(
        &self,
        from: BaseProductId,
        count: i64,
        term: ModeratorBaseProductSearchTerms,
    ) -> ServiceFuture<Vec<BaseProduct>> {
        let user_id = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!(
            "Searching for {} base_products starting from {} with payload: {:?}",
            count, from, term
        );

        self.spawn_on_pool(move |conn| {
            let base_products_repo = repo_factory.create_base_product_repo(&conn, user_id);
            base_products_repo
                .moderator_search(from, count, term)
                .map_err(|e: FailureError| e.context("Service base_products, moderator_search endpoint error occured.").into())
        })
    }

    /// Set moderation status for base_product_ids
    fn set_moderation_status_base_product(
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
                .set_moderation_status(base_product_ids, status)
                .map_err(|e: FailureError| {
                    e.context("Service base_products, set_moderation_status endpoint error occured.")
                        .into()
                })
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
        let currency = self.dynamic_context.currency;
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        if let Some(mut options) = options {
            self.spawn_on_pool(move |conn| {
                let currency_exchange = repo_factory.create_currency_exchange_repo(&*conn, user_id);
                let currencies_map = currency_exchange.get_exchange_for_currency(currency)?;
                options.currency_map = currencies_map;
                Ok(Some(options))
            })
        } else {
            Box::new(future::ok(None))
        }
    }
}

fn recalc_currencies(
    base_products: &mut [BaseProductWithVariants],
    currencies_map: Option<HashMap<Currency, ExchangeRate>>,
    currency: Currency,
) {
    if let Some(currency_map) = currencies_map {
        for base_product in base_products {
            for mut variant in &mut base_product.variants {
                variant.price.0 *= currency_map[&variant.currency].0;
                variant.currency = currency;
            }
        }
    }
}

fn get_path_to_searched_category(searched_category: Option<Category>, root: Category) -> Category {
    if let Some(searched_category) = searched_category {
        if searched_category.children.is_empty() {
            let new_cat = remove_unused_categories(
                root,
                &[searched_category.parent_id.unwrap_or_default()],
                searched_category.level - 2,
            );
            new_cat
        } else {
            let new_cat = remove_unused_categories(root, &[searched_category.id], searched_category.level - 1);
            let removed_cat = clear_child_categories(new_cat, searched_category.level + 1);
            removed_cat
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

fn get_first_level_category(third_level_category_id: i32, root: Category) -> RepoResult<Category> {
    root.children
        .into_iter()
        .find(|cat_child| get_parent_category(&cat_child, third_level_category_id, 2).is_some())
        .ok_or_else(|| {
            format_err!("There is no such 3rd level category in db - {}", third_level_category_id)
                .context(Error::NotFound)
                .into()
        })
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use serde_json;
    use tokio_core::reactor::Core;

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
            category_id: 3,
            slug: Some("slug".to_string()),
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
            rating: None,
            slug: None,
            status: None,
        }
    }

    #[test]
    fn test_get_base_product() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.get_base_product(BaseProductId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, BaseProductId(1));
    }

    #[test]
    fn test_list() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(MOCK_USER_ID), handle);
        let work = service.list_base_products(BaseProductId(1), 5);
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
