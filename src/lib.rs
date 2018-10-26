//! Stores is a microservice responsible for authentication and managing stores and products
//! The layered structure of the app is
//!
//! `Application -> Controller -> Service -> Repo + HttpClient`
//!
//! Each layer can throw Error with context or cover occurred error with
//! Error in the context. When error is not covered with Error it will
//! be translated to code 500 in the http answer "Internal server error" of microservice.

#![allow(proc_macro_derive_resolution_fallback)]
#![recursion_limit = "128"]
extern crate chrono;
extern crate config as config_crate;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate hyper_tls;
extern crate isolang;
extern crate jsonwebtoken;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate num_traits;
extern crate r2d2;
extern crate regex;
extern crate reqwest;
extern crate rust_decimal;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate stq_http;
extern crate stq_logging;
extern crate stq_router;
extern crate stq_static_resources;
extern crate stq_types;
#[macro_use]
extern crate stq_diesel_macro_derive;
extern crate tokio_core;
extern crate validator;
#[macro_use]
extern crate validator_derive;
#[macro_use]
extern crate sentry;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate tokio;
extern crate treexml;
extern crate uuid;

#[macro_use]
pub mod macros;
pub mod config;
pub mod controller;
pub mod elastic;
pub mod errors;
pub mod loaders;
pub mod models;
pub mod repos;
pub mod schema;
pub mod sentry_integration;
pub mod services;

use std::process;
use std::sync::Arc;
use std::time::Duration;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use futures::future;
use futures::{Future, Stream};
use futures_cpupool::CpuPool;
use hyper::server::Http;
use tokio_core::reactor::Core;

use stq_http::controller::Application;

use config::Config;
use controller::context::StaticContext;
use errors::Error;
use loaders::ticker;
use repos::acl::RolesCacheImpl;
use repos::attributes::AttributeCacheImpl;
use repos::categories::CategoryCacheImpl;
use repos::repo_factory::ReposFactoryImpl;

/// Starts new web service from provided `Config`
pub fn start_server<F: FnOnce() + 'static>(config: Config, port: &Option<String>, callback: F) {
    // Prepare reactor
    let mut core = Core::new().expect("Unexpected error creating event loop core");
    let handle = Arc::new(core.handle());

    let http_config = config.to_http_config();
    let client = stq_http::client::Client::new(&http_config, &handle);
    let client_handle = client.handle();
    let client_stream = client.stream();
    handle.spawn(client_stream.for_each(|_| Ok(())));

    // Prepare database pool
    let database_url: String = config.server.database.parse().expect("Database URL must be set in configuration");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");

    let thread_count = config.server.thread_count;

    // Prepare CPU pool
    let cpu_pool = CpuPool::new(thread_count);

    // Prepare server
    let address = {
        let port = port.as_ref().unwrap_or(&config.server.port);
        format!("{}:{}", config.server.host, port).parse().expect("Could not parse address")
    };

    // Roles cache
    let roles_cache = RolesCacheImpl::default();

    // Categories cache
    let category_cache = CategoryCacheImpl::default();

    // Attributes cache
    let attributes_cache = AttributeCacheImpl::default();

    // Repo factory
    let repo_factory = ReposFactoryImpl::new(roles_cache, category_cache, attributes_cache);

    let context = StaticContext::new(db_pool, cpu_pool, client_handle, Arc::new(config), repo_factory);

    let serve = Http::new()
        .serve_addr_handle(&address, &handle, move || {
            // Prepare application
            let controller = controller::ControllerImpl::new(context.clone());
            let app = Application::<Error>::new(controller);

            Ok(app)
        }).unwrap_or_else(|why| {
            error!("Http Server Initialization Error: {}", why);
            process::exit(1);
        });

    let handle_arc2 = handle.clone();
    handle.spawn(
        serve
            .for_each(move |conn| {
                handle_arc2.spawn(conn.map(|_| ()).map_err(|why| error!("Server Error: {}", why)));
                Ok(())
            }).map_err(|_| ()),
    );

    info!("Listening on http://{}, threads: {}", address, thread_count);
    handle.spawn_fn(move || {
        callback();
        future::ok(())
    });
    core.run(future::empty::<(), ()>()).unwrap();
}

pub fn start_rocket_retail_loader(config: Config) {
    let mut core = Core::new().expect("Unexpected error creating event loop core");
    let handle = Arc::new(core.handle());

    let env = loaders::RocketRetailEnvironment::new(config);
    handle.spawn(create_rocket_retail_loader(env));

    core.run(future::empty::<(), ()>()).unwrap();
}

fn create_rocket_retail_loader(env: loaders::RocketRetailEnvironment) -> impl Future<Item = (), Error = ()> {
    let loader = loaders::RocketRetailLoader::new(env);

    let stream = loader.start();
    stream
        .or_else(|e| {
            error!("Error in rocket retail loader: {}.", e);
            futures::future::ok(())
        }).for_each(|_| futures::future::ok(()))
}

pub fn start_ticker(config: Config) {
    let Config { server, ticker, .. } = config;
    let ticker = ticker.expect("Ticker config not found");

    // Prepare database pool
    let database_url = server.database.parse::<String>().expect("Failed to parse database URL");
    let db_manager = ConnectionManager::<PgConnection>::new(database_url);
    let db_pool = r2d2::Pool::builder().build(db_manager).expect("Failed to create connection pool");

    let http_client = reqwest::async::Client::new();

    let interval = Duration::from_secs(ticker.interval_s);

    let thread_pool = CpuPool::new(ticker.thread_count);

    let ctx = ticker::TickerContext {
        api_endpoint_url: ticker.api_endpoint_url,
        db_pool,
        http_client,
        interval,
        thread_pool,
    };

    Core::new()
        .expect("Unexpected error occurred when creating an event loop core for Ticker")
        .run(ticker::run(ctx))
        .unwrap();
}
