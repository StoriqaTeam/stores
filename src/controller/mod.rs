//! `Controller` is a top layer that handles all http-related
//! stuff like reading bodies, parsing params, forming a response.
//! Basically it provides inputs to `Service` layer and converts outputs
//! of `Service` layer to http responses

pub mod error;
pub mod routes;
pub mod types;
pub mod utils;

use std::sync::Arc;
use std::str::FromStr;

use futures::Future;
use futures::future;
use hyper::{Delete, Get, Post, Put};
use hyper::header::Authorization;
use hyper::server::Request;
use serde_json;
use futures_cpupool::CpuPool;

use self::error::Error;
use services::system::{SystemService, SystemServiceImpl};
use services::stores::{StoresService, StoresServiceImpl};
use services::products::{ProductsService, ProductsServiceImpl};
use services::user_roles::{UserRolesService, UserRolesServiceImpl};
use repos::types::DbPool;
use repos::acl::RolesCacheImpl;

use models;
use self::utils::parse_body;
use self::types::ControllerFuture;
use self::routes::{Route, RouteParser};
use http::client::ClientHandle;
use config::Config;

/// Controller handles route parsing and calling `Service` layer
pub struct Controller {
    pub r2d2_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub route_parser: Arc<RouteParser>,
    pub config: Config,
    pub client_handle: ClientHandle,
    pub roles_cache: RolesCacheImpl,
}

macro_rules! serialize_future {
    ($e:expr) => (Box::new($e.map_err(|e| Error::from(e)).and_then(|resp| serde_json::to_string(&resp).map_err(|e| Error::from(e)))))
}

impl Controller {
    /// Create a new controller based on services
    pub fn new(r2d2_pool: DbPool, cpu_pool: CpuPool, client_handle: ClientHandle, config: Config, roles_cache: RolesCacheImpl) -> Self {
        let route_parser = Arc::new(routes::create_route_parser());
        Self {
            route_parser,
            r2d2_pool,
            cpu_pool,
            client_handle,
            config,
            roles_cache,
        }
    }

    /// Handle a request and get future response
    pub fn call(&self, req: Request) -> ControllerFuture {
        let headers = req.headers().clone();
        let auth_header = headers.get::<Authorization<String>>();
        let user_id = auth_header
            .map(move |auth| auth.0.clone())
            .and_then(|id| i32::from_str(&id).ok());

        let cached_roles = self.roles_cache.clone();
        let system_service = SystemServiceImpl::new();
        let stores_service = StoresServiceImpl::new(
            self.r2d2_pool.clone(),
            self.cpu_pool.clone(),
            cached_roles.clone(),
            user_id,
            self.client_handle.clone(),
            self.config.server.elastic.clone(),
        );
        let products_service = ProductsServiceImpl::new(
            self.r2d2_pool.clone(),
            self.cpu_pool.clone(),
            cached_roles,
            user_id,
        );
        let user_roles_service = UserRolesServiceImpl::new(self.r2d2_pool.clone(), self.cpu_pool.clone());

        match (req.method(), self.route_parser.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) => serialize_future!(system_service.healthcheck().map_err(|e| Error::from(e))),

            // GET /stores/<store_id>
            (&Get, Some(Route::Store(store_id))) => serialize_future!(stores_service.get(store_id)),

            // GET /stores
            (&Get, Some(Route::Stores)) => {
                if let (Some(from), Some(count)) = parse_query!(req.query().unwrap_or_default(), "from" => i32, "count" => i64) {
                    serialize_future!(stores_service.list(from, count))
                } else {
                    Box::new(future::err(Error::UnprocessableEntity(
                        "Error parsing request from gateway body".to_string(),
                    )))
                }
            }

            // POST /stores
            (&Post, Some(Route::Stores)) => serialize_future!(
                parse_body::<models::NewStore>(req.body())
                    .map_err(|_| Error::UnprocessableEntity("Error parsing request from gateway body".to_string()))
                    .and_then(move |new_store| stores_service.create(new_store).map_err(|e| Error::from(e)))
            ),

            // PUT /stores/<store_id>
            (&Put, Some(Route::Store(store_id))) => serialize_future!(
                parse_body::<models::UpdateStore>(req.body())
                    .map_err(|_| Error::UnprocessableEntity("Error parsing request from gateway body".to_string()))
                    .and_then(move |update_store| stores_service
                        .update(store_id, update_store)
                        .map_err(|e| Error::from(e)))
            ),

            // DELETE /stores/<store_id>
            (&Delete, Some(Route::Store(store_id))) => serialize_future!(stores_service.deactivate(store_id)),

            // GET /products/<product_id>
            (&Get, Some(Route::Product(product_id))) => serialize_future!(products_service.get(product_id)),

            // GET /products
            (&Get, Some(Route::Products)) => {
                if let (Some(from), Some(count)) = parse_query!(req.query().unwrap_or_default(), "from" => i32, "count" => i64) {
                    serialize_future!(products_service.list(from, count))
                } else {
                    Box::new(future::err(Error::UnprocessableEntity(
                        "Error parsing request from gateway body".to_string(),
                    )))
                }
            }

            // POST /products
            (&Post, Some(Route::Products)) => serialize_future!(
                parse_body::<models::NewProduct>(req.body())
                    .map_err(|_| Error::UnprocessableEntity("Error parsing request from gateway body".to_string()))
                    .and_then(move |new_product| products_service
                        .create(new_product)
                        .map_err(|e| Error::from(e)))
            ),

            // PUT /products/<product_id>
            (&Put, Some(Route::Product(product_id))) => serialize_future!(
                parse_body::<models::UpdateProduct>(req.body())
                    .map_err(|_| Error::UnprocessableEntity("Error parsing request from gateway body".to_string()))
                    .and_then(move |update_product| products_service
                        .update(product_id, update_product)
                        .map_err(|e| Error::from(e)))
            ),

            // DELETE /products/<product_id>
            (&Delete, Some(Route::Product(product_id))) => serialize_future!(products_service.deactivate(product_id)),

            // GET /user_role/<user_role_id>
            (&Get, Some(Route::UserRole(user_role_id))) => serialize_future!(user_roles_service.get(user_role_id)),

            // POST /user_roles
            (&Post, Some(Route::UserRoles)) => serialize_future!(
                parse_body::<models::NewUserRole>(req.body())
                    .map_err(|_| Error::UnprocessableEntity("Error parsing request from gateway body".to_string()))
                    .and_then(move |new_store| user_roles_service
                        .create(new_store)
                        .map_err(|e| Error::from(e)))
            ),

            // DELETE /user_roles/<user_role_id>
            (&Delete, Some(Route::UserRoles)) => serialize_future!(
                parse_body::<models::OldUserRole>(req.body())
                    .map_err(|_| Error::UnprocessableEntity("Error parsing request from gateway body".to_string()))
                    .and_then(move |old_role| user_roles_service
                        .delete(old_role)
                        .map_err(|e| Error::from(e)))
            ),

            // Fallback
            _ => Box::new(future::err(Error::NotFound)),
        }
    }
}
