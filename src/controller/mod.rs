//! `Controller` is a top layer that handles all http-related
//! stuff like reading bodies, parsing params, forming a response.
//! Basically it provides inputs to `Service` layer and converts outputs
//! of `Service` layer to http responses

pub mod context;
pub mod routes;
pub mod utils;

use std::str::FromStr;

use diesel::{connection::AnsiTransactionManager, pg::Pg, Connection};
use failure::Fail;
use futures::{future, Future, IntoFuture};
use hyper::{
    header::{Authorization, Cookie},
    server::Request,
    Delete, Get, Post, Put,
};
use r2d2::ManageConnection;
use validator::Validate;

use stq_http::{
    controller::{Controller, ControllerFuture},
    errors::ErrorMessageWrapper,
    request_util::{self, parse_body, read_body, serialize_future, Currency as CurrencyHeader},
};

use stq_static_resources::{Currency, ModerationStatus};
use stq_types::*;

use self::routes::Route;
use controller::context::{DynamicContext, StaticContext};
use errors::Error;
use models::*;
use repos::repo_factory::*;
use repos::CouponSearch;
use sentry_integration::log_and_capture_error;
use services::attribute_values::{AttributeValuesService, NewAttributeValuePayload};
use services::attributes::AttributesService;
use services::base_products::BaseProductsService;
use services::categories::CategoriesService;
use services::coupons::CouponsService;
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

        let correlation_token = request_util::get_correlation_token(&req);

        let dynamic_context = DynamicContext::new(user_id, currency, correlation_token);

        let service = Service::new(self.static_context.clone(), dynamic_context);

        let path = req.path().to_string();

        let fut = match (&req.method().clone(), self.static_context.route_parser.test(req.path())) {
            // GET /stores/<store_id>
            (&Get, Some(Route::Store(store_id))) => {
                let visibility = parse_query!(req.query().unwrap_or_default(), "visibility" => Visibility);
                serialize_future(service.get_store(store_id, visibility))
            }

            // GET /stores/by-slug/<store_slug>
            (&Get, Some(Route::StoreBySlug(store_slug))) => {
                let visibility = parse_query!(req.query().unwrap_or_default(), "visibility" => Visibility);
                serialize_future(service.get_store_by_slug(store_slug, visibility))
            }

            // GET /stores
            (&Get, Some(Route::Stores)) => {
                let params = parse_query!(
                    req.query().unwrap_or_default(),
                    "offset" => StoreId, "count" => i32, "visibility" => Visibility
                );

                if let (Some(offset), Some(count), visibility) = params {
                    serialize_future(service.list_stores(offset, count, visibility))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters failed, action: get stores")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // GET /stores/:id/products route
            (&Get, Some(Route::StoreProducts(store_id))) => {
                let params = parse_query!(
                    req.query().unwrap_or_default(),
                    "skip_base_product_id" => BaseProductId,
                    "offset" => BaseProductId,
                    "count" => i32,
                    "visibility" => Visibility
                );

                if let (skip_base_product_id, Some(offset), Some(count), visibility) = params {
                    serialize_future(service.get_base_products_of_the_store(store_id, skip_base_product_id, offset, count, visibility))
                } else {
                    Box::new(future::err(
                        format_err!(
                            "Parsing query parameters failed, action: get products by store, store id: {}",
                            store_id
                        ).context(Error::Parse)
                        .into(),
                    ))
                }
            }

            // GET /stores/:id/products/count route
            (&Get, Some(Route::StoreProductsCount(store_id))) => {
                let visibility = parse_query!(req.query().unwrap_or_default(), "visibility" => Visibility);
                serialize_future(service.get_store_products_count(store_id, visibility))
            }

            // GET /stores/slug_exists route
            (&Get, Some(Route::StoresSlugExists)) => {
                if let Some(slug) = parse_query!(req.query().unwrap_or_default(), "slug" => String) {
                    serialize_future(service.store_slug_exists(slug))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters failed, action: check exists slug")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /stores/cart
            (&Post, Some(Route::StoresCart)) => serialize_future(
                parse_body::<Vec<CartProduct>>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: Vec<CartProduct>")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |cart_products| service.find_by_cart(cart_products)),
            ),

            // POST /stores/moderate
            (&Post, Some(Route::StoreModerate)) => serialize_future(
                parse_body::<StoreModerate>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: StoreModerate").context(Error::Parse).into())
                    .and_then(move |store_moderate| service.set_store_moderation_status(store_moderate.store_id, store_moderate.status)),
            ),

            // POST /stores/validate_change_moderation_status
            (&Post, Some(Route::StoreValidateChangeModerationStatus)) => serialize_future(
                parse_body::<StoreModerate>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: StoreModerate").context(Error::Parse).into())
                    .and_then(move |store_moderate| {
                        service.validate_change_moderation_status_store(store_moderate.store_id, store_moderate.status)
                    }),
            ),

            // POST /stores/moderation
            (&Post, Some(Route::StoreModeration(store_id))) => serialize_future(service.send_store_to_moderation(store_id)),

            // POST /stores/search
            (&Post, Some(Route::StoresSearch)) => {
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<SearchStore>(req.body())
                            .map_err(|e| e.context("Parsing body failed, target: SearchStore").context(Error::Parse).into())
                            .and_then(move |store_search| service.find_store_by_name(store_search, count, offset)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters failed, action: search stores")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /stores/search/filters/count
            (&Post, Some(Route::StoresSearchFiltersCount)) => serialize_future(
                parse_body::<SearchStore>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: SearchStore").context(Error::Parse).into())
                    .and_then(move |search_store| service.search_store_filters_count(search_store)),
            ),

            // POST /stores/search/filters/country
            (&Post, Some(Route::StoresSearchFiltersCountry)) => serialize_future(
                parse_body::<SearchStore>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: SearchStore").context(Error::Parse).into())
                    .and_then(move |search_store| service.search_store_filters_country(search_store)),
            ),

            // POST /stores/search/filters/category
            (&Post, Some(Route::StoresSearchFiltersCategory)) => serialize_future(
                parse_body::<SearchStore>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: SearchStore").context(Error::Parse).into())
                    .and_then(move |search_store| service.search_store_filters_category(search_store)),
            ),

            // POST /stores/auto_complete
            (&Post, Some(Route::StoresAutoComplete)) => {
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        read_body(req.body())
                            .map_err(|e| e.context("Parsing body failed, target: String").context(Error::Parse).into())
                            .and_then(move |name| service.store_auto_complete(name, count, offset)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters failed, action: stores auto_complete")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /stores
            (&Post, Some(Route::Stores)) => serialize_future(
                parse_body::<NewStore>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: NewStore").context(Error::Parse).into())
                    .and_then(move |new_store| {
                        new_store
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: NewStore")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.create_store(new_store))
                    }),
            ),

            // GET /stores/count
            (&Get, Some(Route::StoreCount)) => {
                let visibility = parse_query!(
                    req.query().unwrap_or_default(),
                    "visibility" => Visibility
                );

                serialize_future({ service.count(visibility) })
            }

            // PUT /stores/<store_id>
            (&Put, Some(Route::Store(store_id))) => serialize_future(
                parse_body::<UpdateStore>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: UpdateStore").context(Error::Parse).into())
                    .and_then(move |update_store| {
                        update_store
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: UpdateStore")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.update_store(store_id, update_store))
                    }),
            ),

            // DELETE /stores/<store_id>
            (&Delete, Some(Route::Store(store_id))) => serialize_future(service.deactivate_store(store_id)),

            // Get /stores/by_user_id/<user_id>
            (&Get, Some(Route::StoreByUser(user_id_arg))) => serialize_future(service.get_store_by_user(user_id_arg)),

            // DELETE /stores/by_user_id/<user_id>
            (&Delete, Some(Route::StoreByUser(user_id_arg))) => serialize_future(service.delete_store_by_user(user_id_arg)),

            // POST /stores/<store_id>/publish
            (&Post, Some(Route::StorePublish(store_id))) => {
                serialize_future(service.set_store_moderation_status(store_id, ModerationStatus::Published))
            }

            // POST /stores/<store_id>/draft
            (&Post, Some(Route::StoreDraft(store_id))) => serialize_future(service.set_store_moderation_status_draft(store_id)),

            // GET /products/<product_id>
            (&Get, Some(Route::Product(product_id))) => serialize_future(service.get_product(product_id)),

            // GET /products/by_base_product/<base_product_id> route
            (&Get, Some(Route::ProductsByBaseProduct(base_product_id))) => {
                serialize_future(service.find_products_with_base_id(base_product_id))
            }

            // GET /products/<product_id>/attributes route
            (&Get, Some(Route::ProductAttributes(product_id))) => serialize_future(service.find_products_attributes(product_id)),

            // GET /products
            (&Get, Some(Route::Products)) => {
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(service.list_products(offset, count))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters failed, action: get products")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // GET /products/store_id
            (&Get, Some(Route::ProductStoreId)) => {
                let params = parse_query!(
                    req.query().unwrap_or_default(),
                    "product_id" => ProductId,
                    "visibility" => Visibility
                );

                if let (Some(product_id), visibility) = params {
                    serialize_future(service.get_product_store_id(product_id, visibility))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters failed, action: get store id by product")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /products
            (&Post, Some(Route::Products)) => serialize_future(
                parse_body::<NewProductWithAttributes>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: NewProductWithAttributes")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |new_product| {
                        new_product
                            .product
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: NewProductWithAttributes")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.create_product(new_product))
                    }),
            ),

            // PUT /products/<product_id>
            (&Put, Some(Route::Product(product_id))) => serialize_future(
                parse_body::<UpdateProductWithAttributes>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: UpdateProductWithAttributes")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |update_product| {
                        let validation = if let Some(product) = update_product.product.clone() {
                            product
                                .validate()
                                .map_err(|e| {
                                    format_err!("Validation failed, target: UpdateProductWithAttributes")
                                        .context(Error::Validate(e))
                                        .into()
                                }).into_future()
                        } else {
                            future::ok(())
                        };
                        validation.and_then(move |_| service.update_product(product_id, update_product))
                    }),
            ),

            // DELETE /products/<product_id>
            (&Delete, Some(Route::Product(product_id))) => serialize_future(service.deactivate_product(product_id)),

            // GET /base_products/<base_product_id>
            (&Get, Some(Route::BaseProduct(base_product_id))) => {
                let visibility = parse_query!(req.query().unwrap_or_default(), "visibility" => Visibility);
                serialize_future(service.get_base_product(base_product_id, visibility))
            }

            // GET /store/by-slug/<store_slug>/base_products/by-slug/<base_product_slug>
            (&Get, Some(Route::BaseProductBySlug(store_slug, base_product_slug))) => {
                let visibility = parse_query!(req.query().unwrap_or_default(), "visibility" => Visibility);
                serialize_future(service.get_base_product_by_slug(StoreIdentifier::Slug(store_slug), base_product_slug, visibility))
            }

            // GET /base_products/<base_product_id>/update_view
            (&Get, Some(Route::BaseProductWithViewsUpdate(base_product_id))) => {
                serialize_future(service.get_base_product_with_views_update(base_product_id))
            }

            // GET /store/by-slug/<store_slug>/base_products/by-slug/<base_product_slug>/update_view
            (&Get, Some(Route::BaseProductBySlugWithViewsUpdate(store_slug, base_product_slug))) => {
                serialize_future(service.get_base_product_by_slug_with_views_update(StoreIdentifier::Slug(store_slug), base_product_slug))
            }

            // GET /base_products/<base_product_id>/custom_attributes
            (&Get, Some(Route::BaseProductCustomAttributes(base_product_id))) => {
                serialize_future(service.get_custom_attributes_by_base_product(base_product_id))
            }

            // GET /base_products/by_product/<product_id>
            (&Get, Some(Route::BaseProductByProduct(product_id))) => {
                let visibility = parse_query!(req.query().unwrap_or_default(), "visibility" => Visibility);

                serialize_future(service.get_base_product_by_product(product_id, visibility))
            }

            // GET /base_products
            (&Get, Some(Route::BaseProducts)) => {
                let params = parse_query!(
                    req.query().unwrap_or_default(),
                    "offset" => BaseProductId, "count" => i32, "visibility" => Visibility
                );

                if let (Some(offset), Some(count), visibility) = params {
                    serialize_future(service.list_base_products(offset, count, visibility))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters failed, action: get base products")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // GET /base_products/count
            (&Get, Some(Route::BaseProductsCount)) => {
                let visibility = parse_query!(
                    req.query().unwrap_or_default(),
                    "visibility" => Visibility
                );

                serialize_future(service.base_product_count(visibility))
            }

            // POST /base_products
            (&Post, Some(Route::BaseProducts)) => serialize_future(
                parse_body::<NewBaseProduct>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: NewBaseProduct")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |new_base_product| {
                        new_base_product
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: NewBaseProduct")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.create_base_product(new_base_product))
                    }),
            ),

            // POST /base_products/moderate
            (&Post, Some(Route::BaseProductModerate)) => serialize_future(
                parse_body::<BaseProductModerate>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: BaseProductModerate")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |base_product_moderate| {
                        service.set_moderation_status_base_product(base_product_moderate.base_product_id, base_product_moderate.status)
                    }),
            ),

            // POST /base_products/validate_change_moderation_status
            (&Post, Some(Route::BaseProductValidateChangeModerationStatus)) => serialize_future(
                parse_body::<BaseProductModerate>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: BaseProductModerate")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |base_product_moderate| {
                        service.validate_change_moderation_status_base_product(
                            base_product_moderate.base_product_id,
                            base_product_moderate.status,
                        )
                    }),
            ),

            // POST /base_products/moderation
            (&Post, Some(Route::BaseProductModeration(base_product_id))) => {
                serialize_future(service.send_base_product_to_moderation(base_product_id))
            }

            // POST /base_products/with_variants
            (&Post, Some(Route::BaseProductWithVariants)) => serialize_future(
                parse_body::<NewBaseProductWithVariants>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: NewBaseProductWithVariants")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |new_base_product| {
                        new_base_product
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: NewBaseProductWithVariants")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.create_base_product_with_variants(new_base_product))
                    }),
            ),

            // PUT /base_products/<base_product_id>
            (&Put, Some(Route::BaseProduct(base_product_id))) => serialize_future(
                parse_body::<UpdateBaseProduct>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: UpdateBaseProduct")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |update_base_product| {
                        update_base_product
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: UpdateBaseProduct")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.update_base_product(base_product_id, update_base_product))
                    }),
            ),

            // DELETE /base_products/<base_product_id>
            (&Delete, Some(Route::BaseProduct(base_product_id))) => serialize_future(service.deactivate_base_product(base_product_id)),

            // POST /base_products/search
            (&Post, Some(Route::BaseProductsSearch)) => {
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<SearchProductsByName>(req.body())
                            .map_err(|e| {
                                e.context("Parsing body failed, target: SearchProductsByName")
                                    .context(Error::Parse)
                                    .into()
                            }).and_then(move |prod| service.search_base_products_by_name(prod, count, offset)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters failed, action: search base products")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /base_products/auto_complete
            (&Post, Some(Route::BaseProductsAutoComplete)) => {
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<AutoCompleteProductName>(req.body())
                            .map_err(|e| {
                                e.context("Parsing body failed, target: AutoCompleteProductName")
                                    .context(Error::Parse)
                                    .into()
                            }).and_then(move |name| service.base_products_auto_complete(name, count, offset)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters failed, action: auto complete base products")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /base_products/most_discount
            (&Post, Some(Route::BaseProductsMostDiscount)) => {
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<MostDiscountProducts>(req.body())
                            .map_err(|e| {
                                e.context("Parsing body failed, target: MostDiscountProducts")
                                    .context(Error::Parse)
                                    .into()
                            }).and_then(move |prod| service.search_base_products_most_discount(prod, count, offset)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters failed, action: most discount products")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /base_products/most_viewed
            (&Post, Some(Route::BaseProductsMostViewed)) => {
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<MostViewedProducts>(req.body())
                            .map_err(|e| {
                                e.context("Parsing body failed, target: MostViewedProducts")
                                    .context(Error::Parse)
                                    .into()
                            }).and_then(move |prod| service.search_base_products_most_viewed(prod, count, offset)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters failed, action: most viewed products")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /base_products/search/filters/price
            (&Post, Some(Route::BaseProductsSearchFiltersPrice)) => serialize_future(
                parse_body::<SearchProductsByName>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: SearchProductsByName")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |search_prod| service.search_base_products_filters_price(search_prod)),
            ),
            // POST /base_products/search/filters/category
            (&Post, Some(Route::BaseProductsSearchFiltersCategory)) => serialize_future(
                parse_body::<SearchProductsByName>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: SearchProductsByName")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |search_prod| service.search_base_products_filters_category(search_prod)),
            ),
            // POST /base_products/search/filters/attributes
            (&Post, Some(Route::BaseProductsSearchFiltersAttributes)) => serialize_future(
                parse_body::<SearchProductsByName>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: SearchProductsByName")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |search_prod| service.search_base_products_attributes(search_prod)),
            ),
            // POST /base_products/search/filters/count
            (&Post, Some(Route::BaseProductsSearchFiltersCount)) => serialize_future(
                parse_body::<SearchProductsByName>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: SearchProductsByName")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |search_prod| service.search_base_products_filters_count(search_prod)),
            ),

            // POST /base_products/publish
            (&Post, Some(Route::BaseProductPublish)) => serialize_future(
                parse_body::<Vec<BaseProductId>>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: Vec<BaseProductId>")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |base_product_ids| {
                        service.set_moderation_status_base_products(base_product_ids, ModerationStatus::Published)
                    }),
            ),

            // POST /base_products/draft
            (&Post, Some(Route::BaseProductDraft(base_product_id))) => {
                serialize_future(service.set_base_product_moderation_status_draft(base_product_id))
            }

            // POST /custom_attributes
            (&Post, Some(Route::CustomAttributes)) => serialize_future(
                parse_body::<NewCustomAttribute>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: NewCustomAttribute")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |payload| service.create_custom_attribute(payload)),
            ),

            // GET /custom_attributes
            (&Get, Some(Route::CustomAttributes)) => serialize_future(service.list_custom_attributes()),

            // GET /custom_attributes/:id
            (&Get, Some(Route::CustomAttribute(custom_attributes_id))) => {
                serialize_future(service.get_custom_attribute(custom_attributes_id))
            }

            // DELETE /custom_attributes/:id
            (Delete, Some(Route::CustomAttribute(custom_attributes_id))) => {
                serialize_future({ service.delete_custom_attribute(custom_attributes_id) })
            }

            // POST /coupons
            (&Post, Some(Route::Coupons)) => serialize_future(
                parse_body::<NewCoupon>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: NewCoupon").context(Error::Parse).into())
                    .and_then(move |new_coupon| {
                        new_coupon
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: NewCoupon")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.create_coupon(new_coupon))
                    }),
            ),

            // GET /coupons/:id
            (&Get, Some(Route::Coupon(coupon_id))) => serialize_future(service.get_coupon(coupon_id)),

            // GET /coupons/generate_code
            (&Get, Some(Route::CouponsGenerateCode)) => serialize_future(service.generate_coupon_code()),

            // POST /coupons/search/code
            (&Post, Some(Route::CouponsSearchCode)) => serialize_future(
                parse_body::<CouponsSearchCodePayload>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: CouponsSearchCodePayload")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |payload| service.get_coupon_by_code(payload)),
            ),

            // POST /coupons/validate/code
            (&Post, Some(Route::CouponsValidateCode)) => serialize_future(
                parse_body::<CouponsSearchCodePayload>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: CouponsSearchCodePayload")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |payload| service.validate_coupon_by_code(payload)),
            ),

            // GET /coupons/:id/validate
            (&Get, Some(Route::CouponValidate(coupon_id))) => serialize_future(service.validate_coupon(coupon_id)),

            // POST /coupons/:coupon_id/base_products/:base_product_id
            (
                &Post,
                Some(Route::CouponScopeBaseProducts {
                    coupon_id,
                    base_product_id,
                }),
            ) => serialize_future(service.add_base_product_coupon(coupon_id, base_product_id)),

            // POST /coupons/:coupon_id/user_id/:user_id
            (
                &Post,
                Some(Route::UsedCoupon {
                    coupon_id,
                    user_id: user_id_arg,
                }),
            ) => serialize_future(service.add_used_coupon(coupon_id, user_id_arg)),

            // GET /coupons/stores/:id
            (&Get, Some(Route::CouponsSearchFiltersStore(store_id))) => {
                let search = CouponSearch::Store(store_id);
                serialize_future(service.find_coupons(search))
            }

            // GET /coupons/:coupon_id/base_products
            (&Get, Some(Route::BaseProductsByCoupon(coupon_id))) => serialize_future(service.find_base_products_by_coupon(coupon_id)),

            // PUT /coupons/:id
            (&Put, Some(Route::Coupon(coupon_id))) => serialize_future(
                parse_body::<UpdateCoupon>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: UpdateCoupon").context(Error::Parse).into())
                    .and_then(move |update_coupon| {
                        update_coupon
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: UpdateCoupon")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.update_coupon(coupon_id, update_coupon))
                    }),
            ),

            // DELETE /coupons/:id
            (Delete, Some(Route::Coupon(coupon_id))) => serialize_future({ service.delete_coupon(coupon_id) }),

            // DELETE /coupons/:coupon_id/base_products/:base_product_id
            (
                &Delete,
                Some(Route::CouponScopeBaseProducts {
                    coupon_id,
                    base_product_id,
                }),
            ) => serialize_future(service.delete_base_product_from_coupon(coupon_id, base_product_id)),

            // DELETE /coupons/:coupon_id/user_id/:user_id
            (
                &Delete,
                Some(Route::UsedCoupon {
                    coupon_id,
                    user_id: user_id_arg,
                }),
            ) => serialize_future(service.delete_used_coupon(coupon_id, user_id_arg)),

            (&Get, Some(Route::RolesByUserId { user_id })) => serialize_future({ service.get_roles(user_id) }),
            (&Post, Some(Route::Roles)) => {
                serialize_future({ parse_body::<NewUserRole>(req.body()).and_then(move |data| service.create_user_role(data)) })
            }
            (&Delete, Some(Route::Roles)) => {
                serialize_future({ parse_body::<RemoveUserRole>(req.body()).and_then(move |data| service.delete_user_role(data)) })
            }
            (&Delete, Some(Route::RolesByUserId { user_id })) => serialize_future({ service.delete_user_role_by_user_id(user_id) }),
            (&Delete, Some(Route::RoleById { id })) => serialize_future({ service.delete_user_role_by_id(id) }),

            // GET /attributes/<attribute_id>
            (&Get, Some(Route::Attribute(attribute_id))) => serialize_future(service.get_attribute(attribute_id)),

            // GET /attributes/values/<attribute_value_id>
            (&Get, Some(Route::AttributeValue(attribute_value_id))) => serialize_future(service.get_attribute_value(attribute_value_id)),

            // DELETE /attributes/values/<attribute_value_id>
            (&Delete, Some(Route::AttributeValue(attribute_value_id))) => {
                serialize_future(service.delete_attribute_value(attribute_value_id))
            }

            // PUT /attributes/values/<attribute_value_id>
            (&Put, Some(Route::AttributeValue(attribute_value_id))) => serialize_future(
                parse_body::<UpdateAttributeValue>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: UpdateAttributeValue")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |update| {
                        update
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: UpdateAttributeValue")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.update_attribute_value(attribute_value_id, update))
                    }),
            ),

            // GET /attributes/<attribute_id>/values
            (&Get, Some(Route::AttributeValues(attribute_id))) => serialize_future(service.get_attribute_values(attribute_id)),

            // POST /attributes/<attribute_id>/values
            (&Post, Some(Route::AttributeValues(attribute_id))) => serialize_future(
                parse_body::<NewAttributeValuePayload>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: NewAttributeValuePayload")
                            .context(Error::Parse)
                            .into()
                    }).map(move |payload| NewAttributeValue {
                        attr_id: attribute_id,
                        code: payload.code,
                        translations: payload.translations,
                    }).and_then(move |new_attribute| {
                        new_attribute
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: NewAttribute")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.create_attribute_value(new_attribute))
                    }),
            ),

            // GET /attributes
            (&Get, Some(Route::Attributes)) => serialize_future(service.list_attributes()),

            // POST /attributes
            (&Post, Some(Route::Attributes)) => serialize_future(
                parse_body::<CreateAttributePayload>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: CreateAttributePayload")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |new_attribute| {
                        new_attribute
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: CreateAttributePayload")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.create_attribute(new_attribute))
                    }),
            ),

            // PUT /attributes/<attribute_id>
            (&Put, Some(Route::Attribute(attribute_id))) => serialize_future(
                parse_body::<UpdateAttribute>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: UpdateAttribute")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |update_attribute| {
                        update_attribute
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: UpdateAttribute")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.update_attribute(attribute_id, update_attribute))
                    }),
            ),

            // DELETE /attributes/<attribute_id>
            (&Delete, Some(Route::Attribute(attribute_id))) => serialize_future(service.delete_attribute(attribute_id)),

            // GET /categories/<category_id>
            (&Get, Some(Route::Category(category_id))) => serialize_future(service.get_category(category_id)),

            // DELETE /categories/<category_id>
            (&Delete, Some(Route::Category(category_id))) => serialize_future(service.delete_category(category_id)),

            // POST /categories
            (&Post, Some(Route::Categories)) => serialize_future(
                parse_body::<NewCategory>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: NewCategory").context(Error::Parse).into())
                    .and_then(move |new_category| {
                        new_category
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: NewCategory")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.create_category(new_category))
                    }),
            ),

            // PUT /categories/<category_id>
            (&Put, Some(Route::Category(category_id))) => serialize_future(
                parse_body::<UpdateCategory>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: UpdateCategory")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |update_category| {
                        update_category
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: UpdateCategory")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.update_category(category_id, update_category))
                    }),
            ),

            // GET /categories
            (&Get, Some(Route::Categories)) => serialize_future(service.get_all_categories()),

            // GET /categories/<category_id>/attributes
            (&Get, Some(Route::CategoryAttr(category_id))) => serialize_future(service.find_all_attributes_for_category(category_id)),

            // POST /categories/attributes
            (&Post, Some(Route::CategoryAttrs)) => serialize_future(
                parse_body::<NewCatAttr>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: CategoryAttrs").context(Error::Parse).into())
                    .and_then(move |new_category_attr| service.add_attribute_to_category(new_category_attr)),
            ),

            // DELETE /categories/attributes
            (&Delete, Some(Route::CategoryAttrs)) => serialize_future(
                parse_body::<OldCatAttr>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: OldCatAttr").context(Error::Parse).into())
                    .and_then(move |old_category_attr| service.delete_attribute_from_category(old_category_attr)),
            ),

            // GET /currency_exchange
            (&Get, Some(Route::CurrencyExchange)) => serialize_future(service.get_latest_currencies()),

            // POST /currency_exchange
            (&Post, Some(Route::CurrencyExchange)) => serialize_future(
                parse_body::<NewCurrencyExchange>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: NewCurrencyExchange")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |new_currency_exchange| service.update_currencies(new_currency_exchange)),
            ),

            // GET /wizard_stores
            (&Get, Some(Route::WizardStores)) => serialize_future(service.get_wizard_store()),

            // POST /wizard_stores
            (&Post, Some(Route::WizardStores)) => serialize_future(service.create_wizard_store()),

            // PUT /wizard_stores
            (&Put, Some(Route::WizardStores)) => serialize_future(
                parse_body::<UpdateWizardStore>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: UpdateWizardStore")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |update_wizard| {
                        update_wizard
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation failed, target: UpdateWizardStore")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.update_wizard_store(update_wizard))
                    }),
            ),

            // DELETE /wizard_stores
            (&Delete, Some(Route::WizardStores)) => serialize_future(service.delete_wizard_store()),

            // GET /moderator_product_comments/<base_product_id>
            (&Get, Some(Route::ModeratorBaseProductComment(base_product_id))) => {
                serialize_future(service.get_latest_for_product(base_product_id))
            }

            // POST /moderator_product_comments
            (&Post, Some(Route::ModeratorProductComments)) => serialize_future(
                parse_body::<NewModeratorProductComments>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: NewModeratorProductComments")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |new_comments| service.create_product_comment(new_comments)),
            ),

            // GET /moderator_store_comments/<store_id>
            (&Get, Some(Route::ModeratorStoreComment(store_id))) => serialize_future(service.get_latest_for_store(store_id)),

            // POST /moderator_store_comments
            (&Post, Some(Route::ModeratorStoreComments)) => serialize_future(
                parse_body::<NewModeratorStoreComments>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: NewModeratorProductComments")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |new_comments| service.create_store_comment(new_comments)),
            ),

            // GET /products/<product_id>/seller_price
            (&Get, Some(Route::SellerProductPrice(product_id))) => serialize_future(service.get_product_seller_price(product_id)),

            // POST /stores/moderator_search
            (&Post, Some(Route::ModeratorStoreSearch)) => {
                let (offset, skip_opt, count_opt) = parse_query!(
                    req.query().unwrap_or_default(),
                    "offset" => StoreId, "skip" => i64, "count" => i64
                );

                let skip = skip_opt.unwrap_or(0);
                let count = count_opt.unwrap_or(0);

                serialize_future(
                    parse_body::<ModeratorStoreSearchTerms>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body failed, target: ModeratorStoreSearchTerms")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |payload| service.moderator_search_stores(offset, skip, count, payload)),
                )
            }

            // POST /base_products/moderator_search
            (&Post, Some(Route::ModeratorBaseProductSearch)) => {
                let (offset, skip_opt, count_opt) = parse_query!(
                    req.query().unwrap_or_default(),
                    "offset" => BaseProductId, "skip" => i64, "count" => i64
                );

                let skip = skip_opt.unwrap_or(0);
                let count = count_opt.unwrap_or(0);

                serialize_future(
                    parse_body::<ModeratorBaseProductSearchTerms>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body failed, target: ModeratorBaseProductSearchTerms")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |payload| service.moderator_search_base_product(offset, skip, count, payload)),
                )
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
