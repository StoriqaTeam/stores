extern crate failure;
extern crate futures;
#[macro_use]
extern crate log;
extern crate stores_lib;
extern crate stq_logging;
extern crate tokio_core;

use failure::{err_msg, Error as FailureError};
use futures::{future, Future, Stream};
use tokio_core::reactor::Core;

fn main() {
    let config = stores_lib::config::Config::new().expect("Can't load app config!");

    // Prepare sentry integration
    let _sentry = stores_lib::sentry_integration::init(config.sentry.as_ref());

    // Prepare logger
    stq_logging::init(config.graylog.as_ref());

    let ctrl_c = tokio_signal::ctrl_c()
        .flatten_stream()
        .into_future()
        .map_err(|(err, _rest)| FailureError::from(err))
        .and_then(|(ctrl_c, _rest)| match ctrl_c {
            None => future::err(err_msg("Unexpected error: Ctrl+C stream ended")),
            Some(_) => {
                info!("Ctrl+C received. Exiting...");
                future::ok(())
            }
        });

    let fut = stores_lib::start_ticker(config).select(ctrl_c).map_err(|(err, _fut)| err);

    Core::new()
        .expect("Unexpected error occurred when creating an event loop core for Ticker")
        .run(fut)
        .unwrap();
}
