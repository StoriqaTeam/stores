extern crate futures;
extern crate hyper;
extern crate rand;
extern crate stores_lib;
extern crate stq_http;
extern crate stq_static_resources;
extern crate stq_types;
extern crate tokio_core;

use std::thread;

#[allow(unused_imports)]
use futures::future;
#[allow(unused_imports)]
use futures::Future;
use hyper::client::HttpConnector;
use hyper::Client;
use rand::Rng;
use std::sync::mpsc::channel;
use tokio_core::reactor::Core;

#[allow(unused_imports)]
use stq_http::request_util::read_body;

type HttpClient = Client<HttpConnector>;

pub struct Context {
    pub client: HttpClient,
    pub base_url: String,
    pub core: Core,
}

pub fn setup() -> Context {
    let (tx, rx) = channel::<bool>();
    let mut rng = rand::thread_rng();
    let port = rng.gen_range(50000, 60000);
    thread::spawn(move || {
        let config = stores_lib::config::Config::new().expect("Can't load app config!");
        stores_lib::start_server(config, &Some(port.to_string()), move || {
            let _ = tx.send(true);
        });
    });
    rx.recv().unwrap();
    let core = Core::new().expect("Unexpected error creating event loop core");
    let client = Client::new(&core.handle());
    Context {
        client,
        base_url: format!("http://localhost:{}", port),
        core,
    }
}
