extern crate chrono;
extern crate config as config_crate;
#[macro_use]
extern crate failure;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate rusoto_s3;
extern crate tokio;
extern crate tokio_core;
extern crate tokio_signal;
#[macro_use]
extern crate log;

mod config;
mod errors;
mod models;
mod providers;
mod stores_responses;

use std::sync::Arc;
use std::time::{Duration, Instant};

use failure::Fail;
use futures::future::lazy;
use futures::future::Either;
use futures::stream::Stream;
use futures::sync::mpsc;
use futures::Future;
use tokio::timer::Interval;
use tokio_core::reactor::Core;

use self::config::Config;
use self::models::ToXMLDocument;
use self::providers::catalogs::CatalogProvider;
use self::providers::s3::S3Provider;

fn main() {
    let mut core = Core::new().expect("Unexpected error creating event loop core");
    let handle = Arc::new(core.handle());

    let config = Config::new().expect("Could not create config.");
    let catalog_provider = CatalogProvider::with_config(config.clone()).expect("Could not create catalog provider.");
    let s3_provider = S3Provider::with_config(config.clone()).expect("Could not create S3 provider.");

    let catalog = catalog_provider
        .get_rocket_retail_catalog()
        .expect("Could not retrieve catalog from stores microservice.");

    let interval =
        Interval::new(Instant::now(), Duration::from_secs(config.interval_s as u64)).map_err(|e| e.context("timer creation error").into());

    let stream = interval.and_then(move |_| {
        s3_provider.upload_catalog(catalog.clone()).and_then(|url| {
            println!("Uploaded catalog to {}", url.clone());
            futures::future::ok(())
        })
    });

    let fut = stream
        .or_else(|e| {
            error!("Error occurred: {}.", e);
            futures::future::ok(())
        })
        .for_each(|_| futures::future::ok(()));

    handle.spawn(fut);

    core.run(tokio_signal::ctrl_c().flatten_stream().take(1u64).for_each(|()| {
        info!("Ctrl+C received. Exit");

        Ok(())
    }))
    .unwrap();
}
