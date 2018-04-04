//! `Controller` is a top layer that handles all http-related
//! stuff like reading bodies, parsing params, forming a response.
//! Basically it provides inputs to `Service` layer and converts outputs
//! of `Service` layer to http responses

pub mod routes;
pub mod utils;

use std::sync::Arc;
use std::str::FromStr;

use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;

use futures::Future;
use futures::future;
use futures::IntoFuture;
use hyper::{Delete, Get, Post, Put};
use hyper::header::{Authorization, Cookie};
use hyper::server::Request;
use futures_cpupool::CpuPool;
use validator::Validate;
use r2d2::{ManageConnection, Pool};

use stq_http::controller::Controller;
use stq_http::request_util::serialize_future;
use stq_http::errors::ControllerError as Error;
use stq_http::request_util::ControllerFuture;
use stq_http::request_util::{parse_body, read_body};
use stq_http::client::ClientHandle;
use stq_router::RouteParser;

use models;
use services::system::{SystemService, SystemServiceImpl};
use services::stores::{StoresService, StoresServiceImpl};
use services::products::{ProductsService, ProductsServiceImpl};
use services::base_products::{BaseProductsService, BaseProductsServiceImpl};
use services::user_roles::{UserRolesService, UserRolesServiceImpl};
use services::attributes::{AttributesService, AttributesServiceImpl};
use services::categories::{CategoriesService, CategoriesServiceImpl};
use repos::categories::CategoryCacheImpl;
use repos::attributes::AttributeCacheImpl;
use repos::roles_cache::RolesCacheImpl;
use repos::repo_factory::*;
use self::routes::Route;
use config::Config;

/// Controller handles route parsing and calling `Service` layer
#[derive(Clone)]
pub struct ControllerImpl<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub route_parser: Arc<RouteParser<Route>>,
    pub config: Config,
    pub repo_factory: F,
    pub client_handle: ClientHandle,
    pub roles_cache: RolesCacheImpl,
    pub categories_cache: CategoryCacheImpl,
    pub attributes_cache: AttributeCacheImpl,
}

impl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> ControllerImpl<T, M, F>
{
    /// Create a new controller based on services
    pub fn new(
        db_pool: Pool<M>,
        cpu_pool: CpuPool,
        client_handle: ClientHandle,
        config: Config,
        repo_factory: F,
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
            repo_factory,
            roles_cache,
            categories_cache,
            attributes_cache,
        }
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
            .map(move |auth| auth.0.clone())
            .and_then(|id| i32::from_str(&id).ok());

        let uuid_header = headers.get::<Cookie>();
        let uuid = uuid_header.map(move |cookie| cookie.get("UUID"));

        debug!(
            "User with id = '{:?}' and uuid = {:?} is requesting {}",
            user_id,
            uuid,
            req.path()
        );

        let cached_categories = self.categories_cache.clone();
        let cached_attributes = self.attributes_cache.clone();
        let system_service = SystemServiceImpl::new();
        let stores_service = StoresServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            user_id,
            self.client_handle.clone(),
            self.config.server.elastic.clone(),
            self.repo_factory.clone(),
        );
        let products_service = ProductsServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            user_id,
            self.client_handle.clone(),
            self.config.server.elastic.clone(),
            self.repo_factory.clone(),
        );

        let base_products_service = BaseProductsServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            user_id,
            self.client_handle.clone(),
            self.config.server.elastic.clone(),
            self.repo_factory.clone(),
        );

        let user_roles_service = UserRolesServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            self.roles_cache.clone(),
            self.repo_factory.clone(),
        );
        let attributes_service = AttributesServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            cached_attributes,
            user_id,
            self.repo_factory.clone(),
        );

        let categories_service = CategoriesServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            cached_categories,
            user_id,
            self.repo_factory.clone(),
        );

        match (req.method(), self.route_parser.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /healthcheck",
                    user_id
                );
                serialize_future(system_service.healthcheck().map_err(Error::from))
            }

            // GET /stores/<store_id>
            (&Get, Some(Route::Store(store_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /stores/{}",
                    user_id, store_id
                );
                serialize_future(stores_service.get(store_id))
            }

            // GET /stores
            (&Get, Some(Route::Stores)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /stores",
                    user_id
                );
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(stores_service.list(offset, count))
                } else {
                    error!("Parsing query parameters // GET /stores failed!");
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // GET /stores/:id/products route
            (&Get, Some(Route::StoreProducts(store_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /stores/:id/products route",
                    user_id
                );
                if let (skip_base_product_id, Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "skip_base_product_id" => i32, "offset" => i32, "count" => i32)
                {
                    serialize_future(base_products_service.get_products_of_the_store(store_id, skip_base_product_id, offset, count))
                } else {
                    error!("Parsing query parameters // GET /stores/:id/product failed!");
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // GET /stores/:id/products/count route
            (&Get, Some(Route::StoreProductsCount(store_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /stores/{}",
                    user_id, store_id
                );
                serialize_future(stores_service.get_products_count(store_id))
            }

            // POST /stores/search
            (&Post, Some(Route::StoresSearch)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /stores/search",
                    user_id
                );
                println!("req body - {:?}", req.body_ref());
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<models::SearchStore>(req.body())
                            .map_err(|_| {
                                error!("Parsing body // POST /stores/search in models::SearchStore failed!");
                                Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                            })
                            .and_then(move |store_search| {
                                stores_service
                                    .find_by_name(store_search, count, offset)
                                    .map_err(Error::from)
                            }),
                    )
                } else {
                    error!("Parsing query parameters // POST /stores/search failed!");
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // POST /stores/auto_complete
            (&Post, Some(Route::StoresAutoComplete)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /stores/auto_complete",
                    user_id
                );
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        read_body(req.body())
                            .map_err(|_| {
                                error!("Parsing body // POST /stores/auto_complete in String failed!");
                                Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                            })
                            .and_then(move |name| {
                                stores_service
                                    .auto_complete(name, count, offset)
                                    .map_err(Error::from)
                            }),
                    )
                } else {
                    error!("Parsing query parameters // POST /stores/auto_complete failed!");
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // POST /stores
            (&Post, Some(Route::Stores)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /stores",
                    user_id
                );
                serialize_future(
                    parse_body::<models::NewStore>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // POST /stores in models::NewStore failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
                        .and_then(move |new_store| {
                            new_store
                                .validate()
                                .map_err(Error::Validate)
                                .into_future()
                                .and_then(move |_| stores_service.create(new_store).map_err(Error::from))
                        }),
                )
            }

            // PUT /stores/<store_id>
            (&Put, Some(Route::Store(store_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // PUT /stores/{}",
                    user_id, store_id
                );
                serialize_future(
                    parse_body::<models::UpdateStore>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // PUT /stores/<store_id> in models::UpdateStore failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
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
                )
            }

            // DELETE /stores/<store_id>
            (&Delete, Some(Route::Store(store_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /stores/{}",
                    user_id, store_id
                );
                serialize_future(stores_service.deactivate(store_id))
            }

            // GET /products/<product_id>
            (&Get, Some(Route::Product(product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /products/{}",
                    user_id, product_id
                );
                serialize_future(products_service.get(product_id))
            }

            // GET /products
            (&Get, Some(Route::Products)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /products",
                    user_id
                );
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(products_service.list(offset, count))
                } else {
                    error!("Parsing query parameters // GET /products failed!");
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // POST /products
            (&Post, Some(Route::Products)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /products",
                    user_id
                );
                serialize_future(
                    parse_body::<models::NewProductWithAttributes>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // POST /products in models::NewProductWithAttributes failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
                        .and_then(move |new_product| {
                            new_product
                                .product
                                .validate()
                                .map_err(Error::Validate)
                                .into_future()
                                .and_then(move |_| products_service.create(new_product).map_err(Error::from))
                        }),
                )
            }

            // PUT /products/<product_id>
            (&Put, Some(Route::Product(product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // PUT /products/{}",
                    user_id, product_id
                );
                serialize_future(
                    parse_body::<models::UpdateProductWithAttributes>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // PUT /products/<product_id> in models::UpdateProductWithAttributes failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
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
                )
            }

            // DELETE /products/<product_id>
            (&Delete, Some(Route::Product(product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /products/{}",
                    user_id, product_id
                );
                serialize_future(products_service.deactivate(product_id))
            }

            // GET /base_products/<base_product_id>
            (&Get, Some(Route::BaseProduct(base_product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /base_products/{}",
                    user_id, base_product_id
                );
                serialize_future(base_products_service.get(base_product_id))
            }

            // GET /base_products/<base_product_id>/with_variants
            (&Get, Some(Route::BaseProductWithVariant(base_product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /base_products/{}/with_variants",
                    user_id, base_product_id
                );
                serialize_future(base_products_service.get_with_variants(base_product_id))
            }

            

            // GET /base_products
            (&Get, Some(Route::BaseProducts)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /base_products",
                    user_id
                );
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(base_products_service.list(offset, count))
                } else {
                    error!("Parsing query parameters // GET /base_products failed!");
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // POST /base_products
            (&Post, Some(Route::BaseProducts)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /base_products",
                    user_id
                );
                serialize_future(
                    parse_body::<models::NewBaseProduct>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // POST /base_products in models::NewBaseProduct failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
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
                )
            }

            // PUT /base_products/<base_product_id>
            (&Put, Some(Route::BaseProduct(base_product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // PUT /base_products/{}",
                    user_id, base_product_id
                );
                serialize_future(
                    parse_body::<models::UpdateBaseProduct>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // PUT /base_products/<base_product_id> in models::UpdateBaseProduct failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
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
                )
            }

            // DELETE /base_products/<base_product_id>
            (&Delete, Some(Route::BaseProduct(base_product_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /base_products/{}",
                    user_id, base_product_id
                );
                serialize_future(base_products_service.deactivate(base_product_id))
            }

            // POST /products/search
            (&Post, Some(Route::ProductsSearch)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /products/search",
                    user_id
                );
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<models::SearchProductsByName>(req.body())
                            .map_err(|_| {
                                error!("Parsing body // POST /products/search in models::SearchProductsByName failed!");
                                Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                            })
                            .and_then(move |prod| {
                                base_products_service
                                    .search_by_name(prod, count, offset)
                                    .map_err(Error::from)
                            }),
                    )
                } else {
                    error!("Parsing query parameters // POST /products/search failed!");
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // POST /products/auto_complete
            (&Post, Some(Route::ProductsAutoComplete)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /products/auto_complete",
                    user_id
                );
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        read_body(req.body())
                            .map_err(|_| {
                                error!("Parsing body // POST /products/auto_complete in String failed!");
                                Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                            })
                            .and_then(move |name| {
                                base_products_service
                                    .auto_complete(name, count, offset)
                                    .map_err(Error::from)
                            }),
                    )
                } else {
                    error!("Parsing query parameters // POST /products/auto_complete failed!");
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // POST /products/most_discount
            (&Post, Some(Route::ProductsMostDiscount)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /products/most_discount",
                    user_id
                );
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<models::MostDiscountProducts>(req.body())
                            .map_err(|_| {
                                error!("Parsing body // POST /products/most_discount in models::MostDiscountProducts failed!");
                                Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                            })
                            .and_then(move |prod| {
                                base_products_service
                                    .search_most_discount(prod, count, offset)
                                    .map_err(Error::from)
                            }),
                    )
                } else {
                    error!("Parsing query parameters // POST /products/most_discount failed!");
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // POST /products/most_viewed
            (&Post, Some(Route::ProductsMostViewed)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /products/most_viewed",
                    user_id
                );
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i32) {
                    serialize_future(
                        parse_body::<models::MostViewedProducts>(req.body())
                            .map_err(|_| {
                                error!("Parsing body // POST /products/most_viewed in models::MostViewedProducts failed!");
                                Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                            })
                            .and_then(move |prod| {
                                base_products_service
                                    .search_most_viewed(prod, count, offset)
                                    .map_err(Error::from)
                            }),
                    )
                } else {
                    error!("Parsing query parameters // POST /products/most_viewed failed!");
                    Box::new(future::err(Error::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // POST /products/search_filters
            (&Post, Some(Route::ProductsSearchFilters)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /products/search_filters",
                    user_id
                );
                serialize_future(
                    read_body(req.body())
                        .map_err(|_| {
                            error!("Parsing body // POST /products/search_filters in String failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
                        .and_then(move |name| {
                            base_products_service
                                .search_filters(name)
                                .map_err(Error::from)
                        }),
                )
            }

            // GET /user_role/<user_id>
            (&Get, Some(Route::UserRole(user_id_arg))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /user_role/{}",
                    user_id, user_id_arg
                );
                serialize_future(user_roles_service.get_roles(user_id_arg))
            }

            // POST /user_roles
            (&Post, Some(Route::UserRoles)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /user_roles",
                    user_id
                );
                serialize_future(
                    parse_body::<models::NewUserRole>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // POST /user_roles in models::NewUserRole failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
                        .and_then(move |new_role| user_roles_service.create(new_role).map_err(Error::from)),
                )
            }

            // DELETE /user_roles
            (&Delete, Some(Route::UserRoles)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /user_roles",
                    user_id
                );
                serialize_future(
                    parse_body::<models::OldUserRole>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // DELETE /user_roles/<user_role_id> in models::OldUserRole failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
                        .and_then(move |old_role| user_roles_service.delete(old_role).map_err(Error::from)),
                )
            }

            // POST /roles/default/<user_id>
            (&Post, Some(Route::DefaultRole(user_id_arg))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /roles/default/{}",
                    user_id, user_id_arg
                );
                serialize_future(user_roles_service.create_default(user_id_arg))
            }

            // DELETE /roles/default/<user_id>
            (&Delete, Some(Route::DefaultRole(user_id_arg))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /roles/default/{}",
                    user_id, user_id_arg
                );
                serialize_future(user_roles_service.delete_default(user_id_arg))
            }

            // GET /attributes/<attribute_id>
            (&Get, Some(Route::Attribute(attribute_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /attributes/{}",
                    user_id, attribute_id
                );
                serialize_future(attributes_service.get(attribute_id))
            }

            // POST /attributes
            (&Post, Some(Route::Attributes)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /attributes",
                    user_id
                );
                serialize_future(
                    parse_body::<models::NewAttribute>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // POST /attributes in models::NewAttribute failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
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
                )
            }

            // PUT /attributes/<attribute_id>
            (&Put, Some(Route::Attribute(attribute_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // PUT /attributes/{}",
                    user_id, attribute_id
                );
                serialize_future(
                    parse_body::<models::UpdateAttribute>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // PUT /attributes/<attribute_id> in models::UpdateAttribute failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
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
                )
            }

            // GET /categories/<category_id>
            (&Get, Some(Route::Category(category_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /categories/{}",
                    user_id, category_id
                );
                serialize_future(categories_service.get(category_id))
            }

            // POST /categories
            (&Post, Some(Route::Categories)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /categories",
                    user_id
                );
                serialize_future(
                    parse_body::<models::NewCategory>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // POST /categories in models::NewCategory failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
                        .and_then(move |new_category| {
                            new_category
                                .validate()
                                .map_err(Error::Validate)
                                .into_future()
                                .and_then(move |_| categories_service.create(new_category).map_err(Error::from))
                        }),
                )
            }

            // PUT /categories/<category_id>
            (&Put, Some(Route::Category(category_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // PUT /categories/{}",
                    user_id, category_id
                );
                serialize_future(
                    parse_body::<models::UpdateCategory>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // PUT /categories/<category_id> in models::UpdateCategory failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
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
                )
            }

            // GET /categories
            (&Get, Some(Route::Categories)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /categories",
                    user_id
                );
                serialize_future(categories_service.get_all())
            }

            // GET /categories/<category_id>/attributes
            (&Get, Some(Route::CategoryAttr(category_id))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // GET /categories/{}/attributes",
                    user_id, category_id
                );
                serialize_future(categories_service.find_all_attributes(category_id))
            }

            // POST /categories/attributes
            (&Post, Some(Route::CategoryAttrs)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /categories/attributes",
                    user_id
                );
                serialize_future(
                    parse_body::<models::NewCatAttr>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // POST /categories/attributes in models::CategoryAttrs failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
                        .and_then(move |new_category_attr| {
                            categories_service
                                .add_attribute_to_category(new_category_attr)
                                .map_err(Error::from)
                        }),
                )
            }

            // DELETE /categories/attributes
            (&Delete, Some(Route::CategoryAttrs)) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /categories/attributes",
                    user_id
                );
                serialize_future(
                    parse_body::<models::OldCatAttr>(req.body())
                        .map_err(|_| {
                            error!("Parsing body // DELETE /categories/attributes in models::OldCatAttr failed!");
                            Error::UnprocessableEntity(format_err!("Error parsing request from gateway body"))
                        })
                        .and_then(move |old_category_attr| {
                            categories_service
                                .delete_attribute_from_category(old_category_attr)
                                .map_err(Error::from)
                        }),
                )
            }

            // Fallback
            _ => {
                error!(
                    "User with id = '{:?}' requests non existing endpoint in stores microservice!",
                    user_id
                );
                Box::new(future::err(Error::NotFound))
            }
        }
    }
}
