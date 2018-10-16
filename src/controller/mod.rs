//! `Controller` is a top layer that handles all http-related
//! stuff like reading bodies, parsing params, forming a response.
//! Basically it provides inputs to `Service` layer and converts outputs
//! of `Service` layer to http responses

pub mod context;
pub mod routes;
pub mod utils;

use std::str::FromStr;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future;
use futures::Future;
use futures::IntoFuture;
use hyper::header::{Authorization, Cookie};
use hyper::server::Request;
use hyper::{Delete, Get, Post, Put};
use r2d2::ManageConnection;
use validator::Validate;

use stq_http::controller::Controller;
use stq_http::controller::ControllerFuture;
use stq_http::errors::ErrorMessageWrapper;
use stq_http::request_util::serialize_future;
use stq_http::request_util::Currency as CurrencyHeader;
use stq_http::request_util::{parse_body, read_body};
use stq_static_resources::{Currency, ModerationStatus};
use stq_types::*;

use self::routes::Route;
use controller::context::{DynamicContext, StaticContext};
use errors::Error;
use models::*;
use repos::repo_factory::*;
use sentry_integration::log_and_capture_error;
use services::attributes::AttributesService;
use services::base_products::BaseProductsService;
use services::categories::CategoriesService;
use services::currency_exchange::CurrencyExchangeService;
use services::custom_attributes::CustomAttributesService;
use services::moderator_comments::ModeratorCommentsService;
use services::products::ProductsService;
use services::stores::StoresService;
use services::user_roles::UserRolesService;
use services::wizard_stores::WizardStoresService;
use services::Service;

/// Controller handles route parsing and calling `Service` layer
pub struct ControllerImpl<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    pub static_context: StaticContext<T, M, F>,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ControllerImpl<T, M, F>
{
    /// Create a new controller based on services
    pub fn new(static_context: StaticContext<T, M, F>) -> Self {
        Self { static_context }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > Controller for ControllerImpl<T, M, F>
{
    /// Handle a request and get future response
    fn call(&self, req: Request) -> ControllerFuture {
        let headers = req.headers().clone();
        let auth_header = headers.get::<Authorization<String>>();
        let user_id = auth_header
            .map(|auth| auth.0.clone())
            .and_then(|id| i32::from_str(&id).ok())
            .map(UserId);

        let uuid_header = headers.get::<Cookie>();
        let uuid = uuid_header.and_then(|cookie| cookie.get("UUID"));
        debug!("User with id = '{:?}' and uuid = {:?} is requesting {}", user_id, uuid, req.path());

        let currency = match headers
            .get::<CurrencyHeader>()
            .ok_or(format_err!("Missing currency header"))
            .and_then(|sid| Currency::from_code(sid).ok_or(format_err!("Invalid currency: {}", sid)))
            .map_err(|e| e.context(Error::Parse).into())
        {
            Ok(v) => v,
            Err(e) => {
                return Box::new(future::err(e));
            }
        };

        let dynamic_context = DynamicContext::new(user_id, currency);

        let service = Service::new(self.static_context.clone(), dynamic_context);

        let path = req.path().to_string();

        let fut = match (&req.method().clone(), self.static_context.route_parser.test(req.path())) {
            // GET /stores/<store_id>
            (&Get, Some(Route::Store(store_id))) => {
                debug!("User with id = '{:?}' is requesting  // GET /stores/{}", user_id, store_id);
                serialize_future(service.get_store(store_id))
            }

            // GET /stores
            (&Get, Some(Route::Stores)) => {
                debug!("User with id = '{:?}' is requesting  // GET /stores", user_id);
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => StoreId, "count" => i32) {
                    serialize_future(service.list_stores(offset, count))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // GET /stores failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // GET /stores/:id/products route
            (&Get, Some(Route::StoreProducts(store_id))) => {
                debug!("User with id = '{:?}' is requesting  // GET /stores/:id/products route", user_id);
                if let (skip_base_product_id, Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "skip_base_product_id" => BaseProductId, "offset" => BaseProductId, "count" => i32)
                {
                    serialize_future(service.get_base_products_of_the_store(store_id, skip_base_product_id, offset, count))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // GET /stores/:id/product failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // GET /stores/:id/products/count route
            (&Get, Some(Route::StoreProductsCount(store_id))) => {
                debug!("User with id = '{:?}' is requesting  // GET /stores/{}", user_id, store_id);
                serialize_future(service.get_store_products_count(store_id))
            }

            // GET /stores/slug_exists route
            (&Get, Some(Route::StoresSlugExists)) => {
                if let Some(slug) = parse_query!(req.query().unwrap_or_default(), "slug" => String) {
                    serialize_future(service.store_slug_exists(slug))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // GET /stores/slug_exists failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /stores/cart
            (&Post, Some(Route::StoresCart)) => {
                debug!("User with id = '{:?}' is requesting  // POST /stores/cart", user_id);
                serialize_future(
                    parse_body::<Vec<CartProduct>>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /stores/cart in Vec<CartProduct> failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |cart_products| service.find_by_cart(cart_products)),
                )
            }

            // POST /stores/search
            (&Post, Some(Route::StoresSearch)) => {
                debug!("User with id = '{:?}' is requesting  // POST /stores/search", user_id);
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<SearchStore>(req.body())
                            .map_err(|e| {
                                e.context("Parsing body // POST /stores/search in SearchStore failed!")
                                    .context(Error::Parse)
                                    .into()
                            }).and_then(move |store_search| service.find_store_by_name(store_search, count, offset)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // POST /stores/search failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /stores/search/filters/count
            (&Post, Some(Route::StoresSearchFiltersCount)) => {
                debug!("User with id = '{:?}' is requesting  // POST /stores/search/filters/count", user_id);
                serialize_future(
                    parse_body::<SearchStore>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /stores/search/filters/count in SearchStore failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |search_store| service.search_store_filters_count(search_store)),
                )
            }

            // POST /stores/search/filters/country
            (&Post, Some(Route::StoresSearchFiltersCountry)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /stores/search/filters/country",
                    user_id
                );
                serialize_future(
                    parse_body::<SearchStore>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /stores/search/filters/country in SearchStore failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |search_store| service.search_store_filters_country(search_store)),
                )
            }

            // POST /stores/search/filters/category
            (&Post, Some(Route::StoresSearchFiltersCategory)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /stores/search/filters/category",
                    user_id
                );
                serialize_future(
                    parse_body::<SearchStore>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /stores/search/filters/category in SearchStore failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |search_store| service.search_store_filters_category(search_store)),
                )
            }

            // POST /stores/auto_complete
            (&Post, Some(Route::StoresAutoComplete)) => {
                debug!("User with id = '{:?}' is requesting  // POST /stores/auto_complete", user_id);
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        read_body(req.body())
                            .map_err(|e| {
                                e.context("Parsing body // POST /stores/auto_complete in String failed!")
                                    .context(Error::Parse)
                                    .into()
                            }).and_then(move |name| service.store_auto_complete(name, count, offset)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // POST /stores/auto_complete failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /stores
            (&Post, Some(Route::Stores)) => {
                debug!("User with id = '{:?}' is requesting  // POST /stores", user_id);
                serialize_future(
                    parse_body::<NewStore>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /stores in NewStore failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |new_store| {
                            new_store
                                .validate()
                                .map_err(|e| format_err!("Validation of NewStore failed!").context(Error::Validate(e)).into())
                                .into_future()
                                .and_then(move |_| service.create_store(new_store))
                        }),
                )
            }

            // PUT /stores/<store_id>
            (&Put, Some(Route::Store(store_id))) => {
                debug!("User with id = '{:?}' is requesting  // PUT /stores/{}", user_id, store_id);
                serialize_future(
                    parse_body::<UpdateStore>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /stores/<store_id> in UpdateStore failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |update_store| {
                            update_store
                                .validate()
                                .map_err(|e| format_err!("Validation of UpdateStore failed!").context(Error::Validate(e)).into())
                                .into_future()
                                .and_then(move |_| service.update_store(store_id, update_store))
                        }),
                )
            }

            // DELETE /stores/<store_id>
            (&Delete, Some(Route::Store(store_id))) => {
                debug!("User with id = '{:?}' is requesting  // DELETE /stores/{}", user_id, store_id);
                serialize_future(service.deactivate_store(store_id))
            }

            // Get /stores/by_user_id/<user_id>
            (&Get, Some(Route::StoreByUser(user_id_arg))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // Get /stores/by_user_id/{}",
                    user_id, user_id_arg
                );
                serialize_future(service.get_store_by_user(user_id_arg))
            }

            // DELETE /stores/by_user_id/<user_id>
            (&Delete, Some(Route::StoreByUser(user_id_arg))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /stores/by_user_id/{}",
                    user_id, user_id_arg
                );
                serialize_future(service.delete_store_by_user(user_id_arg))
            }

            // POST /stores/<store_id>/publish
            (&Post, Some(Route::StorePublish(store_id))) => {
                debug!("Received request to publish store {}", store_id);
                serialize_future(service.set_store_moderation_status(store_id, ModerationStatus::Published))
            }

            // POST /stores/<store_id>/draft
            (&Post, Some(Route::StoreDraft(store_id))) => {
                debug!("Received request to draft store {}", store_id);
                serialize_future(service.set_store_moderation_status(store_id, ModerationStatus::Draft))
            }

            // GET /products/<product_id>
            (&Get, Some(Route::Product(product_id))) => {
                debug!("User with id = '{:?}' is requesting  // GET /products/{}", user_id, product_id);
                serialize_future(service.get_product(product_id))
            }

            // GET products/by_base_product/<base_product_id> route
            (&Get, Some(Route::ProductsByBaseProduct(base_product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET products/by_base_product/{}",
                    user_id, base_product_id
                );
                serialize_future(service.find_products_with_base_id(base_product_id))
            }

            // GET products/<product_id>/attributes route
            (&Get, Some(Route::ProductAttributes(product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET attributes/{}/attributes",
                    user_id, product_id
                );
                serialize_future(service.find_products_attributes(product_id))
            }

            // GET /products
            (&Get, Some(Route::Products)) => {
                debug!("User with id = '{:?}' is requesting  // GET /products", user_id);
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(service.list_products(offset, count))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // GET /products failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // GET /products/store_id
            (&Get, Some(Route::ProductStoreId)) => {
                debug!("User with id = '{:?}' is requesting  // GET /products/store_id", user_id);
                if let Some(product_id) = parse_query!(req.query().unwrap_or_default(), "product_id" => ProductId) {
                    serialize_future(service.get_product_store_id(product_id))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // GET /products/store_id failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /products
            (&Post, Some(Route::Products)) => {
                debug!("User with id = '{:?}' is requesting  // POST /products", user_id);
                serialize_future(
                    parse_body::<NewProductWithAttributes>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /products in NewProductWithAttributes failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |new_product| {
                            new_product
                                .product
                                .validate()
                                .map_err(|e| {
                                    format_err!("Validation of NewProductWithAttributes failed!")
                                        .context(Error::Validate(e))
                                        .into()
                                }).into_future()
                                .and_then(move |_| service.create_product(new_product))
                        }),
                )
            }

            // PUT /products/<product_id>
            (&Put, Some(Route::Product(product_id))) => {
                debug!("User with id = '{:?}' is requesting  // PUT /products/{}", user_id, product_id);
                serialize_future(
                    parse_body::<UpdateProductWithAttributes>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /products/<product_id> in UpdateProductWithAttributes failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |update_product| {
                            let validation = if let Some(product) = update_product.product.clone() {
                                product
                                    .validate()
                                    .map_err(|e| {
                                        format_err!("Validation of UpdateProductWithAttributes failed!")
                                            .context(Error::Validate(e))
                                            .into()
                                    }).into_future()
                            } else {
                                future::ok(())
                            };
                            validation.and_then(move |_| service.update_product(product_id, update_product))
                        }),
                )
            }

            // DELETE /products/<product_id>
            (&Delete, Some(Route::Product(product_id))) => {
                debug!("User with id = '{:?}' is requesting  // DELETE /products/{}", user_id, product_id);
                serialize_future(service.deactivate_product(product_id))
            }

            // GET /base_products/<base_product_id>
            (&Get, Some(Route::BaseProduct(base_product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /base_products/{}",
                    user_id, base_product_id
                );
                serialize_future(service.get_base_product(base_product_id))
            }

            // GET /base_products/<base_product_id>/update_view
            (&Get, Some(Route::BaseProductWithViewsUpdate(base_product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /base_products/{}/update_view",
                    user_id, base_product_id
                );
                serialize_future(service.get_base_product_with_views_update(base_product_id))
            }

            // GET /base_products/<base_product_id>/custom_attributes
            (&Get, Some(Route::BaseProductCustomAttributes(base_product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /base_products/{}/custom_attributes",
                    user_id, base_product_id
                );
                serialize_future(service.get_custom_attributes_by_base_product(base_product_id))
            }

            // GET base_products/by_product/<product_id>
            (&Get, Some(Route::BaseProductByProduct(product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET base_products/by_product/{}",
                    user_id, product_id
                );
                serialize_future(service.get_base_product_by_product(product_id))
            }

            // GET /base_products
            (&Get, Some(Route::BaseProducts)) => {
                debug!("User with id = '{:?}' is requesting  // GET /base_products", user_id);
                if let (Some(offset), Some(count)) =
                    parse_query!(req.query().unwrap_or_default(), "offset" => BaseProductId, "count" => i32)
                {
                    serialize_future(service.list_base_products(offset, count))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // GET /base_products failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /base_products
            (&Post, Some(Route::BaseProducts)) => {
                debug!("User with id = '{:?}' is requesting  // POST /base_products", user_id);
                serialize_future(
                    parse_body::<NewBaseProduct>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /base_products in NewBaseProduct failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |new_base_product| {
                            new_base_product
                                .validate()
                                .map_err(|e| {
                                    format_err!("Validation of NewBaseProduct failed!")
                                        .context(Error::Validate(e))
                                        .into()
                                }).into_future()
                                .and_then(move |_| service.create_base_product(new_base_product))
                        }),
                )
            }

            // POST /base_products/with_variants
            (&Post, Some(Route::BaseProductWithVariants)) => {
                debug!("User with id = '{:?}' is requesting  // POST /base_products/with_variants", user_id);
                serialize_future(
                    parse_body::<NewBaseProductWithVariants>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /base_products/with_variants in NewBaseProductWithVariants failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |new_base_product| {
                            new_base_product
                                .validate()
                                .map_err(|e| {
                                    format_err!("Validation of NewBaseProductWithVariants failed!")
                                        .context(Error::Validate(e))
                                        .into()
                                }).into_future()
                                .and_then(move |_| service.create_base_product_with_variants(new_base_product))
                        }),
                )
            }

            // PUT /base_products/<base_product_id>
            (&Put, Some(Route::BaseProduct(base_product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // PUT /base_products/{}",
                    user_id, base_product_id
                );
                serialize_future(
                    parse_body::<UpdateBaseProduct>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /base_products/<base_product_id> in UpdateBaseProduct failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |update_base_product| {
                            update_base_product
                                .validate()
                                .map_err(|e| {
                                    format_err!("Validation of UpdateBaseProduct failed!")
                                        .context(Error::Validate(e))
                                        .into()
                                }).into_future()
                                .and_then(move |_| service.update_base_product(base_product_id, update_base_product))
                        }),
                )
            }

            // DELETE /base_products/<base_product_id>
            (&Delete, Some(Route::BaseProduct(base_product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /base_products/{}",
                    user_id, base_product_id
                );
                serialize_future(service.deactivate_base_product(base_product_id))
            }

            // POST /base_products/search
            (&Post, Some(Route::BaseProductsSearch)) => {
                debug!("User with id = '{:?}' is requesting  // POST /products/search", user_id);
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<SearchProductsByName>(req.body())
                            .map_err(|e| {
                                e.context("Parsing body // POST /products/search in SearchProductsByName failed!")
                                    .context(Error::Parse)
                                    .into()
                            }).and_then(move |prod| service.search_base_products_by_name(prod, count, offset)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // POST /products/search failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /base_products/auto_complete
            (&Post, Some(Route::BaseProductsAutoComplete)) => {
                debug!("User with id = '{:?}' is requesting  // POST /products/auto_complete", user_id);
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<AutoCompleteProductName>(req.body())
                            .map_err(|e| {
                                e.context("Parsing body // POST /products/auto_complete in AutoCompleteProductName failed!")
                                    .context(Error::Parse)
                                    .into()
                            }).and_then(move |name| service.base_products_auto_complete(name, count, offset)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // POST /products/auto_complete failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /base_products/most_discount
            (&Post, Some(Route::BaseProductsMostDiscount)) => {
                debug!("User with id = '{:?}' is requesting  // POST /products/most_discount", user_id);
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<MostDiscountProducts>(req.body())
                            .map_err(|e| {
                                e.context("Parsing body // POST /products/most_discount in MostDiscountProducts failed!")
                                    .context(Error::Parse)
                                    .into()
                            }).and_then(move |prod| service.search_base_products_most_discount(prod, count, offset)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // POST /products/most_discount failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /base_products/most_viewed
            (&Post, Some(Route::BaseProductsMostViewed)) => {
                debug!("User with id = '{:?}' is requesting  // POST /products/most_viewed", user_id);
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<MostViewedProducts>(req.body())
                            .map_err(|e| {
                                e.context("Parsing body // POST /products/most_viewed in MostViewedProducts failed!")
                                    .context(Error::Parse)
                                    .into()
                            }).and_then(move |prod| service.search_base_products_most_viewed(prod, count, offset)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // POST /products/most_viewed failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /base_products/search/filters/price
            (&Post, Some(Route::BaseProductsSearchFiltersPrice)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /products/search/filters/price",
                    user_id
                );
                serialize_future(
                    parse_body::<SearchProductsByName>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /products/search/filters/price in SearchProductsByName failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |search_prod| service.search_base_products_filters_price(search_prod)),
                )
            }
            // POST /base_products/search/filters/category
            (&Post, Some(Route::BaseProductsSearchFiltersCategory)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /products/search/filters/category",
                    user_id
                );
                serialize_future(
                    parse_body::<SearchProductsByName>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /products/search/filters/category in SearchProductsByName failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |search_prod| service.search_base_products_filters_category(search_prod)),
                )
            }
            // POST /base_products/search/filters/attributes
            (&Post, Some(Route::BaseProductsSearchFiltersAttributes)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /products/search/filters/attributes",
                    user_id
                );
                serialize_future(
                    parse_body::<SearchProductsByName>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /products/search/filters/attributes in SearchProductsByName failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |search_prod| service.search_base_products_attributes(search_prod)),
                )
            }
            // POST /base_products/search/filters/count
            (&Post, Some(Route::BaseProductsSearchFiltersCount)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /products/search/filters/count",
                    user_id
                );
                serialize_future(
                    parse_body::<SearchProductsByName>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /products/search/filters/count in SearchProductsByName failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |search_prod| service.search_base_products_filters_count(search_prod)),
                )
            }

            // POST /base_products/publish
            (&Post, Some(Route::BaseProductPublish)) => {
                debug!("User with id = '{:?}' is requesting  // POST /base_products/publish", user_id);
                serialize_future(
                    parse_body::<Vec<BaseProductId>>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /base_products/publish in Vec<BaseProductId> failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |base_product_ids| {
                            service.set_moderation_status_base_product(base_product_ids, ModerationStatus::Published)
                        }),
                )
            }

            // POST /base_products/draft
            (&Post, Some(Route::BaseProductDraft)) => {
                debug!("User with id = '{:?}' is requesting  // POST /base_products/draft", user_id);
                serialize_future(
                    parse_body::<Vec<BaseProductId>>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /base_products/draft in Vec<BaseProductId> failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |base_product_ids| {
                            service.set_moderation_status_base_product(base_product_ids, ModerationStatus::Draft)
                        }),
                )
            }

            // POST /custom_attributes
            (&Post, Some(Route::CustomAttributes)) => {
                debug!("User with id = '{:?}' is requesting  // POST /custom_attributes", user_id);
                serialize_future(
                    parse_body::<NewCustomAttribute>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /custom_attributes in NewCustomAttribute failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |payload| service.create_custom_attribute(payload)),
                )
            }

            // GET /custom_attributes
            (&Get, Some(Route::CustomAttributes)) => {
                debug!("User with id = '{:?}' is requesting  // GET /custom_attributes", user_id);
                serialize_future(service.list_custom_attributes())
            }

            // GET /custom_attributes/:id
            (&Get, Some(Route::CustomAttribute(custom_attributes_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /custom_attributes/{}",
                    user_id, custom_attributes_id
                );
                serialize_future(service.get_custom_attribute(custom_attributes_id))
            }

            // DELETE /custom_attributes/:id
            (Delete, Some(Route::CustomAttribute(custom_attributes_id))) => {
                debug!("Received request to delete custom_attributes by user id {:?}", user_id);
                serialize_future({ service.delete_custom_attribute(custom_attributes_id) })
            }

            (Get, Some(Route::RolesByUserId { user_id })) => {
                debug!("Received request to get roles by user id {}", user_id);
                serialize_future({ service.get_roles(user_id) })
            }
            (Post, Some(Route::Roles)) => serialize_future({
                parse_body::<NewUserRole>(req.body()).and_then(move |data| {
                    debug!("Received request to create role {:?}", data);
                    service.create_user_role(data)
                })
            }),
            (Delete, Some(Route::Roles)) => serialize_future({
                parse_body::<RemoveUserRole>(req.body()).and_then(move |data| {
                    debug!("Received request to remove role {:?}", data);
                    service.delete_user_role(data)
                })
            }),
            (Delete, Some(Route::RolesByUserId { user_id })) => {
                debug!("Received request to delete role by user id {}", user_id);
                serialize_future({ service.delete_user_role_by_user_id(user_id) })
            }
            (Delete, Some(Route::RoleById { id })) => {
                debug!("Received request to delete role by id {}", id);
                serialize_future({ service.delete_user_role_by_id(id) })
            }

            // GET /attributes/<attribute_id>
            (&Get, Some(Route::Attribute(attribute_id))) => {
                debug!("User with id = '{:?}' is requesting  // GET /attributes/{}", user_id, attribute_id);
                serialize_future(service.get_attribute(attribute_id))
            }

            // GET /attributes
            (&Get, Some(Route::Attributes)) => {
                debug!("User with id = '{:?}' is requesting  // GET /attributes", user_id);
                serialize_future(service.list_attributes())
            }

            // POST /attributes
            (&Post, Some(Route::Attributes)) => {
                debug!("User with id = '{:?}' is requesting  // POST /attributes", user_id);
                serialize_future(
                    parse_body::<NewAttribute>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /attributes in NewAttribute failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |new_attribute| {
                            new_attribute
                                .validate()
                                .map_err(|e| format_err!("Validation of NewAttribute failed!").context(Error::Validate(e)).into())
                                .into_future()
                                .and_then(move |_| service.create_attribute(new_attribute))
                        }),
                )
            }

            // PUT /attributes/<attribute_id>
            (&Put, Some(Route::Attribute(attribute_id))) => {
                debug!("User with id = '{:?}' is requesting  // PUT /attributes/{}", user_id, attribute_id);
                serialize_future(
                    parse_body::<UpdateAttribute>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /attributes/<attribute_id> in UpdateAttribute failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |update_attribute| {
                            update_attribute
                                .validate()
                                .map_err(|e| {
                                    format_err!("Validation of UpdateAttribute failed!")
                                        .context(Error::Validate(e))
                                        .into()
                                }).into_future()
                                .and_then(move |_| service.update_attribute(attribute_id, update_attribute))
                        }),
                )
            }

            // GET /categories/<category_id>
            (&Get, Some(Route::Category(category_id))) => {
                debug!("User with id = '{:?}' is requesting  // GET /categories/{}", user_id, category_id);
                serialize_future(service.get_category(category_id))
            }

            // POST /categories
            (&Post, Some(Route::Categories)) => {
                debug!("User with id = '{:?}' is requesting  // POST /categories", user_id);
                serialize_future(
                    parse_body::<NewCategory>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /categories in NewCategory failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |new_category| {
                            new_category
                                .validate()
                                .map_err(|e| format_err!("Validation of NewCategory failed!").context(Error::Validate(e)).into())
                                .into_future()
                                .and_then(move |_| service.create_category(new_category))
                        }),
                )
            }

            // PUT /categories/<category_id>
            (&Put, Some(Route::Category(category_id))) => {
                debug!("User with id = '{:?}' is requesting  // PUT /categories/{}", user_id, category_id);
                serialize_future(
                    parse_body::<UpdateCategory>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /categories/<category_id> in UpdateCategory failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |update_category| {
                            update_category
                                .validate()
                                .map_err(|e| {
                                    format_err!("Validation of UpdateCategory failed!")
                                        .context(Error::Validate(e))
                                        .into()
                                }).into_future()
                                .and_then(move |_| service.update_category(category_id, update_category))
                        }),
                )
            }

            // GET /categories
            (&Get, Some(Route::Categories)) => {
                debug!("User with id = '{:?}' is requesting  // GET /categories", user_id);
                serialize_future(service.get_all_categories())
            }

            // GET /categories/<category_id>/attributes
            (&Get, Some(Route::CategoryAttr(category_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /categories/{}/attributes",
                    user_id, category_id
                );
                serialize_future(service.find_all_attributes_for_category(category_id))
            }

            // POST /categories/attributes
            (&Post, Some(Route::CategoryAttrs)) => {
                debug!("User with id = '{:?}' is requesting  // POST /categories/attributes", user_id);
                serialize_future(
                    parse_body::<NewCatAttr>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /categories/attributes in CategoryAttrs failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |new_category_attr| service.add_attribute_to_category(new_category_attr)),
                )
            }

            // DELETE /categories/attributes
            (&Delete, Some(Route::CategoryAttrs)) => {
                debug!("User with id = '{:?}' is requesting  // DELETE /categories/attributes", user_id);
                serialize_future(
                    parse_body::<OldCatAttr>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // DELETE /categories/attributes in OldCatAttr failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |old_category_attr| service.delete_attribute_from_category(old_category_attr)),
                )
            }

            // GET /currency_exchange
            (&Get, Some(Route::CurrencyExchange)) => {
                debug!("User with id = '{:?}' is requesting  // GET /currency_exchange", user_id);
                serialize_future(service.get_latest_currencies())
            }

            // POST /currency_exchange
            (&Post, Some(Route::CurrencyExchange)) => {
                debug!("User with id = '{:?}' is requesting  // POST /currency_exchange", user_id);
                serialize_future(
                    parse_body::<NewCurrencyExchange>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /currency_exchange in NewCurrencyExchange failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |new_currency_exchange| service.update_currencies(new_currency_exchange)),
                )
            }

            // GET /wizard_stores
            (&Get, Some(Route::WizardStores)) => {
                debug!("User with id = '{:?}' is requesting  // GET /wizard_stores", user_id);
                serialize_future(service.get_wizard_store())
            }

            // POST /wizard_stores
            (&Post, Some(Route::WizardStores)) => {
                debug!("User with id = '{:?}' is requesting  // POST /wizard_stores", user_id);
                serialize_future(service.create_wizard_store())
            }

            // PUT /wizard_stores
            (&Put, Some(Route::WizardStores)) => {
                debug!("User with id = '{:?}' is requesting  // PUT /wizard_stores", user_id);
                serialize_future(
                    parse_body::<UpdateWizardStore>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /wizard_stores in UpdateWizardStore failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |update_wizard| {
                            update_wizard
                                .validate()
                                .map_err(|e| {
                                    format_err!("Validation of UpdateWizardStore failed!")
                                        .context(Error::Validate(e))
                                        .into()
                                }).into_future()
                                .and_then(move |_| service.update_wizard_store(update_wizard))
                        }),
                )
            }

            // DELETE /wizard_stores
            (&Delete, Some(Route::WizardStores)) => {
                debug!("User with id = '{:?}' is requesting  // DELETE /wizard_stores", user_id);
                serialize_future(service.delete_wizard_store())
            }

            // GET /moderator_product_comments/<base_product_id>
            (&Get, Some(Route::ModeratorBaseProductComment(base_product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /moderator_product_comments/{}",
                    user_id, base_product_id
                );
                serialize_future(service.get_latest_for_product(base_product_id))
            }

            // POST /moderator_product_comments
            (&Post, Some(Route::ModeratorProductComments)) => {
                debug!("User with id = '{:?}' is requesting  // POST /moderator_product_comments", user_id);
                serialize_future(
                    parse_body::<NewModeratorProductComments>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /moderator_product_comments in NewModeratorProductComments failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |new_comments| service.create_product_comment(new_comments)),
                )
            }

            // GET /moderator_store_comments/<store_id>
            (&Get, Some(Route::ModeratorStoreComment(store_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /moderator_store_comments/{}",
                    user_id, store_id
                );
                serialize_future(service.get_latest_for_store(store_id))
            }

            // POST /moderator_store_comments
            (&Post, Some(Route::ModeratorStoreComments)) => {
                debug!("User with id = '{:?}' is requesting  // POST /moderator_store_comments", user_id);
                serialize_future(
                    parse_body::<NewModeratorStoreComments>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /moderator_store_comments in NewModeratorProductComments failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |new_comments| service.create_store_comment(new_comments)),
                )
            }

            // GET /products/<product_id>/seller_price
            (&Get, Some(Route::SellerProductPrice(product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /products/{}/seller_price",
                    user_id, product_id
                );
                serialize_future(service.get_product_seller_price(product_id))
            }

            // POST /stores/moderator_search
            (&Post, Some(Route::ModeratorStoreSearch)) => {
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => StoreId, "count" => i64) {
                    debug!("Received request to search {} stores starting from {}", count, offset);
                    serialize_future(
                        parse_body::<ModeratorStoreSearchTerms>(req.body())
                            .map_err(|e| {
                                e.context("Parsing body // POST /stores/moderator_search in ModeratorStoreSearchTerms failed!")
                                    .context(Error::Parse)
                                    .into()
                            }).inspect(|payload| {
                                debug!("Received request to search for store whith payload {:?}", payload);
                            }).and_then(move |payload| service.moderator_search_stores(offset, count, payload)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // POST /stores/moderator_search failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /base_products/moderator_search
            (&Post, Some(Route::ModeratorBaseProductSearch)) => {
                if let (Some(offset), Some(count)) =
                    parse_query!(req.query().unwrap_or_default(), "offset" => BaseProductId, "count" => i64)
                {
                    debug!("Received request to search {} base_products starting from {}", count, offset);
                    serialize_future(
                        parse_body::<ModeratorBaseProductSearchTerms>(req.body())
                            .map_err(|e| {
                                e.context("Parsing body // POST /base_products/moderator_search in ModeratorBaseProductSearchTerms failed!")
                                    .context(Error::Parse)
                                    .into()
                            }).inspect(|payload| {
                                debug!("Received request to search for base_product whith payload {:?}", payload);
                            }).and_then(move |payload| service.moderator_search_base_product(offset, count, payload)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // POST /base_products/moderator_search failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // Fallback
            (m, _) => Box::new(future::err(
                format_err!("Request to non existing endpoint in stores microservice! {:?} {:?}", m, path)
                    .context(Error::NotFound)
                    .into(),
            )),
        }.map_err(|err| {
            let wrapper = ErrorMessageWrapper::<Error>::from(&err);
            if wrapper.inner.code == 500 {
                log_and_capture_error(&err);
            }
            err
        });

        Box::new(fut)
    }
}
