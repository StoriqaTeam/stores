//! `Controller` is a top layer that handles all http-related
//! stuff like reading bodies, parsing params, forming a response.
//! Basically it provides inputs to `Service` layer and converts outputs
//! of `Service` layer to http responses

pub mod routes;
pub mod utils;

use std::sync::Arc;
use std::str::FromStr;

use futures::Future;
use futures::future;
use futures::IntoFuture;
use hyper::{Delete, Get, Post, Put};
use hyper::header::Authorization;
use hyper::server::Request;
use futures_cpupool::CpuPool;
use validator::Validate;

use stq_http::controller::Controller;
use stq_http::request_util::serialize_future;
use stq_http::errors::ControllerError as Error;
use stq_http::request_util::ControllerFuture;
use stq_http::request_util::{parse_body, read_body};
use stq_router::RouteParser;
use stq_http::client::ClientHandle;

use services::system::{SystemService, SystemServiceImpl};
use services::stores::{StoresService, StoresServiceImpl};
use services::products::{ProductsService, ProductsServiceImpl};
use services::base_products::{BaseProductsService, BaseProductsServiceImpl};
use services::user_roles::{UserRolesService, UserRolesServiceImpl};
use services::attributes::{AttributesService, AttributesServiceImpl};
use services::categories::{CategoriesService, CategoriesServiceImpl};
use repos::types::DbPool;
use repos::acl::RolesCacheImpl;
use repos::categories::CategoryCacheImpl;
use repos::attributes::AttributeCacheImpl;

use models;
use self::routes::Route;
use config::Config;

/// Controller handles route parsing and calling `Service` layer
#[derive(Clone)]
pub struct ControllerImpl {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub route_parser: Arc<RouteParser<Route>>,
    pub config: Config,
    pub client_handle: ClientHandle,
    pub roles_cache: RolesCacheImpl,
    pub categories_cache: CategoryCacheImpl,
    pub attributes_cache: AttributeCacheImpl,
}

impl ControllerImpl {
    /// Create a new controller based on services
    pub fn new(
        db_pool: DbPool,
        cpu_pool: CpuPool,
        client_handle: ClientHandle,
        config: Config,
        roles_cache: RolesCacheImpl,
        categories_cache: CategoryCacheImpl,
        attributes_cache: AttributeCacheImpl,
    ) -> Self {
        let route_parser = Arc::new(routes::create_route_parser());
        Self {
            route_parser,
            db_pool,
            cpu_pool,
            client_handle,
            config,
            roles_cache,
            categories_cache,
            attributes_cache,
        }
    }
}

impl Controller for ControllerImpl {
    /// Handle a request and get future response
    fn call(&self, req: Request) -> ControllerFuture {
        let headers = req.headers().clone();
        let auth_header = headers.get::<Authorization<String>>();
        let user_id = auth_header
            .map(move |auth| auth.0.clone())
            .and_then(|id| i32::from_str(&id).ok());

        let cached_roles = self.roles_cache.clone();
        let cached_categories = self.categories_cache.clone();
        let cached_attributes = self.attributes_cache.clone();
        let system_service = SystemServiceImpl::new();
        let stores_service = StoresServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            cached_roles.clone(),
            user_id,
            self.client_handle.clone(),
            self.config.server.elastic.clone(),
        );
        let products_service = ProductsServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            cached_roles.clone(),
            user_id,
            self.client_handle.clone(),
            self.config.server.elastic.clone(),
        );

        let base_products_service = BaseProductsServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            cached_roles.clone(),
            user_id,
            self.client_handle.clone(),
            self.config.server.elastic.clone(),
        );

        let user_roles_service = UserRolesServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            cached_roles.clone(),
        );
        let attributes_service = AttributesServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            cached_roles.clone(),
            cached_attributes,
            user_id,
        );

        let categories_service = CategoriesServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            cached_roles.clone(),
            cached_categories,
            user_id,
        );

        match (req.method(), self.route_parser.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) => serialize_future(system_service.healthcheck().map_err(Error::from)),

            // GET /stores/<store_id>
            (&Get, Some(Route::Store(store_id))) => serialize_future(stores_service.get(store_id)),

            // GET /stores
            (&Get, Some(Route::Stores)) => {
                if let (Some(from), Some(count)) = parse_query!(req.query().unwrap_or_default(), "from" => i32, "count" => i64) {
                    serialize_future(stores_service.list(from, count))
                } else {
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // GET /stores/search
            (&Get, Some(Route::StoresSearch)) => {
                if let (Some(count), Some(offset)) = parse_query!(req.query().unwrap_or_default(), "count" => i64, "offset" => i64) {
                    serialize_future(
                        parse_body::<models::SearchStore>(req.body())
                            .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                            .and_then(move |store_search| {
                                stores_service
                                    .find_by_name(store_search, count, offset)
                                    .map_err(Error::from)
                            }),
                    )
                } else {
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // GET /stores/auto_complete
            (&Get, Some(Route::StoresAutoComplete)) => {
                if let (Some(count), Some(offset)) = parse_query!(req.query().unwrap_or_default(), "count" => i64, "offset" => i64) {
                    serialize_future(
                        read_body(req.body())
                            .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                            .and_then(move |name| {
                                stores_service
                                    .auto_complete(name, count, offset)
                                    .map_err(Error::from)
                            }),
                    )
                } else {
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // POST /stores
            (&Post, Some(Route::Stores)) => serialize_future(
                parse_body::<models::NewStore>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |new_store| {
                        new_store
                            .validate()
                            .map_err(Error::Validate)
                            .into_future()
                            .and_then(move |_| stores_service.create(new_store).map_err(Error::from))
                    }),
            ),

            // PUT /stores/<store_id>
            (&Put, Some(Route::Store(store_id))) => serialize_future(
                parse_body::<models::UpdateStore>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |update_store| {
                        update_store
                            .validate()
                            .map_err(Error::Validate)
                            .into_future()
                            .and_then(move |_| {
                                stores_service
                                    .update(store_id, update_store)
                                    .map_err(Error::from)
                            })
                    }),
            ),

            // DELETE /stores/<store_id>
            (&Delete, Some(Route::Store(store_id))) => serialize_future(stores_service.deactivate(store_id)),

            // GET /products/<product_id>
            (&Get, Some(Route::Product(product_id))) => serialize_future(products_service.get(product_id)),

            // GET /products
            (&Get, Some(Route::Products)) => {
                if let (Some(from), Some(count)) = parse_query!(req.query().unwrap_or_default(), "from" => i32, "count" => i64) {
                    serialize_future(products_service.list(from, count))
                } else {
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // POST /products
            (&Post, Some(Route::Products)) => serialize_future(
                parse_body::<models::NewProductWithAttributes>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |new_product| {
                        new_product
                            .product
                            .validate()
                            .map_err(Error::Validate)
                            .into_future()
                            .and_then(move |_| products_service.create(new_product).map_err(Error::from))
                    }),
            ),

            // PUT /products/<product_id>
            (&Put, Some(Route::Product(product_id))) => serialize_future(
                parse_body::<models::UpdateProductWithAttributes>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |update_product| {
                        update_product
                            .product
                            .validate()
                            .map_err(Error::Validate)
                            .into_future()
                            .and_then(move |_| {
                                products_service
                                    .update(product_id, update_product)
                                    .map_err(Error::from)
                            })
                    }),
            ),

            // DELETE /products/<product_id>
            (&Delete, Some(Route::Product(product_id))) => serialize_future(products_service.deactivate(product_id)),

            // GET /base_products/<base_product_id>
            (&Get, Some(Route::BaseProduct(base_product_id))) => serialize_future(base_products_service.get(base_product_id)),

            // GET /base_products/<base_product_id>/with_variants
            (&Get, Some(Route::BaseProductWithVariants(base_product_id))) => {
                serialize_future(base_products_service.get_with_variants(base_product_id))
            }

            // GET /base_products
            (&Get, Some(Route::BaseProducts)) => {
                if let (Some(from), Some(count)) = parse_query!(req.query().unwrap_or_default(), "from" => i32, "count" => i64) {
                    serialize_future(base_products_service.list(from, count))
                } else {
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // POST /base_products
            (&Post, Some(Route::BaseProducts)) => serialize_future(
                parse_body::<models::NewBaseProduct>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |new_base_product| {
                        new_base_product
                            .validate()
                            .map_err(Error::Validate)
                            .into_future()
                            .and_then(move |_| {
                                base_products_service
                                    .create(new_base_product)
                                    .map_err(Error::from)
                            })
                    }),
            ),

            // PUT /base_products/<base_product_id>
            (&Put, Some(Route::BaseProduct(base_product_id))) => serialize_future(
                parse_body::<models::UpdateBaseProduct>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |update_base_product| {
                        update_base_product
                            .validate()
                            .map_err(Error::Validate)
                            .into_future()
                            .and_then(move |_| {
                                base_products_service
                                    .update(base_product_id, update_base_product)
                                    .map_err(Error::from)
                            })
                    }),
            ),

            // DELETE /base_products/<base_product_id>
            (&Delete, Some(Route::BaseProduct(base_product_id))) => serialize_future(base_products_service.deactivate(base_product_id)),

            // GET /products/search
            (&Get, Some(Route::ProductsSearch)) => {
                if let (Some(count), Some(offset)) = parse_query!(req.query().unwrap_or_default(), "count" => i64, "offset" => i64) {
                    serialize_future(
                        parse_body::<models::SearchProductsByName>(req.body())
                            .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                            .and_then(move |prod| {
                                base_products_service
                                    .search_by_name(prod, count, offset)
                                    .map_err(Error::from)
                            }),
                    )
                } else {
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // GET /products/auto_complete
            (&Get, Some(Route::ProductsAutoComplete)) => {
                if let (Some(count), Some(offset)) = parse_query!(req.query().unwrap_or_default(), "count" => i64, "offset" => i64) {
                    serialize_future(
                        read_body(req.body())
                            .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                            .and_then(move |name| {
                                base_products_service
                                    .auto_complete(name, count, offset)
                                    .map_err(Error::from)
                            }),
                    )
                } else {
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // GET /products/most_discount
            (&Get, Some(Route::ProductsMostDiscount)) => {
                if let (Some(count), Some(offset)) = parse_query!(req.query().unwrap_or_default(), "count" => i64, "offset" => i64) {
                    serialize_future(
                        parse_body::<models::MostDiscountProducts>(req.body())
                            .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                            .and_then(move |prod| {
                                base_products_service
                                    .search_most_discount(prod, count, offset)
                                    .map_err(Error::from)
                            }),
                    )
                } else {
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // GET /products/most_viewed
            (&Get, Some(Route::ProductsMostViewed)) => {
                if let (Some(count), Some(offset)) = parse_query!(req.query().unwrap_or_default(), "count" => i64, "offset" => i64) {
                    serialize_future(
                        parse_body::<models::MostViewedProducts>(req.body())
                            .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                            .and_then(move |prod| {
                                base_products_service
                                    .search_most_viewed(prod, count, offset)
                                    .map_err(Error::from)
                            }),
                    )
                } else {
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // GET /user_role/<user_id>
            (&Get, Some(Route::UserRole(user_id))) => serialize_future(user_roles_service.get_roles(user_id)),

            // POST /user_roles
            (&Post, Some(Route::UserRoles)) => serialize_future(
                parse_body::<models::NewUserRole>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |new_role| user_roles_service.create(new_role).map_err(Error::from)),
            ),

            // DELETE /user_roles/<user_role_id>
            (&Delete, Some(Route::UserRoles)) => serialize_future(
                parse_body::<models::OldUserRole>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |old_role| user_roles_service.delete(old_role).map_err(Error::from)),
            ),

            // POST /roles/default/<user_id>
            (&Post, Some(Route::DefaultRole(user_id))) => serialize_future(user_roles_service.create_default(user_id)),

            // DELETE /roles/default/<user_id>
            (&Delete, Some(Route::DefaultRole(user_id))) => serialize_future(user_roles_service.delete_default(user_id)),

            // GET /attributes/<attribute_id>
            (&Get, Some(Route::Attribute(attribute_id))) => serialize_future(attributes_service.get(attribute_id)),

            // POST /attributes
            (&Post, Some(Route::Attributes)) => serialize_future(
                parse_body::<models::NewAttribute>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |new_attribute| {
                        new_attribute
                            .validate()
                            .map_err(Error::Validate)
                            .into_future()
                            .and_then(move |_| {
                                attributes_service
                                    .create(new_attribute)
                                    .map_err(Error::from)
                            })
                    }),
            ),

            // PUT /attributes/<attribute_id>
            (&Put, Some(Route::Attribute(attribute_id))) => serialize_future(
                parse_body::<models::UpdateAttribute>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |update_attribute| {
                        update_attribute
                            .validate()
                            .map_err(Error::Validate)
                            .into_future()
                            .and_then(move |_| {
                                attributes_service
                                    .update(attribute_id, update_attribute)
                                    .map_err(Error::from)
                            })
                    }),
            ),

            // GET /categories/<category_id>
            (&Get, Some(Route::Category(category_id))) => serialize_future(categories_service.get(category_id)),

            // POST /categories
            (&Post, Some(Route::Categories)) => serialize_future(
                parse_body::<models::NewCategory>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |new_category| {
                        new_category
                            .validate()
                            .map_err(Error::Validate)
                            .into_future()
                            .and_then(move |_| categories_service.create(new_category).map_err(Error::from))
                    }),
            ),

            // PUT /categories/<category_id>
            (&Put, Some(Route::Category(category_id))) => serialize_future(
                parse_body::<models::UpdateCategory>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |update_category| {
                        update_category
                            .validate()
                            .map_err(Error::Validate)
                            .into_future()
                            .and_then(move |_| {
                                categories_service
                                    .update(category_id, update_category)
                                    .map_err(Error::from)
                            })
                    }),
            ),

            // GET /categories
            (&Get, Some(Route::Categories)) => serialize_future(categories_service.get_all()),

            // GET /categories/<category_id>/attributes
            (&Get, Some(Route::CategoryAttr(category_id))) => serialize_future(categories_service.find_all_attributes(category_id)),

            // POST /categories/attributes
            (&Post, Some(Route::CategoryAttrs)) => serialize_future(
                parse_body::<models::NewCatAttr>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |new_category_attr| {
                        categories_service
                            .add_attribute_to_category(new_category_attr)
                            .map_err(Error::from)
                    }),
            ),

            // DELETE /categories/attributes
            (&Delete, Some(Route::CategoryAttrs)) => serialize_future(
                parse_body::<models::OldCatAttr>(req.body())
                    .map_err(|_| Error::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |old_category_attr| {
                        categories_service
                            .delete_attribute_from_category(old_category_attr)
                            .map_err(Error::from)
                    }),
            ),

            // Fallback
            _ => Box::new(future::err(Error::NotFound)),
        }
    }
}
