//! Users is a microservice responsible for authentication and managing user profiles.
//! This create is for running the service from `stores_lib`. See `stores_lib` for details.

extern crate stores_lib;
extern crate stq_logging;

fn main() {
    let config = stores_lib::config::Config::new().expect("Can't load app config!");

    // Prepare logger
    stq_logging::init(config.graylog.as_ref());

    stores_lib::start_server(config, &None, || ());
}
