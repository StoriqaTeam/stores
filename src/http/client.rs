use std::fmt;
use std::mem;

use tokio_core::reactor::{Handle};
use hyper;
use futures::future::IntoFuture;
use futures::{future, Future};
use futures::sync::{mpsc, oneshot};
use futures::stream::{Stream};
use futures::sink::Sink;
use serde_json;
use serde::de::Deserialize;
use hyper_tls;

use ::config::Config;

pub type ClientResult = Result<String, Error>;
pub type HyperClient = hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>;

pub struct Client {
    client: HyperClient,
    tx: mpsc::Sender<Payload>,
    rx: mpsc::Receiver<Payload>,
    max_retries: usize,
}

impl Client {
    pub fn new(config: &Config, handle: &Handle) -> Self {
        let max_retries = config.client.http_client_retries;
        let (tx, rx) = mpsc::channel::<Payload>(config.client.http_client_buffer_size);
        let client = hyper::Client::configure()
            .connector(hyper_tls::HttpsConnector::new(4, &handle).unwrap())
            .build(&handle);

        Client { client, tx, rx, max_retries }
    }

    pub fn stream(self) -> Box<Stream<Item=(), Error=()>> {
        let Self { client, tx: _, rx, max_retries: _ } = self;
        Box::new(
            rx.and_then(move |payload| {
                Self::send_request(&client, payload).map(|_| ()).map_err(|_| ())
            })
        )
    }

    pub fn handle(&self) -> ClientHandle {
        ClientHandle {
            tx: self.tx.clone(),
            max_retries: self.max_retries,
        }
    }

    fn send_request(client: &HyperClient, payload: Payload) -> Box<Future<Item=(), Error=()>> {
        let Payload { url, method, body: maybe_body, headers: maybe_headers, callback } = payload;

        let uri = match url.parse() {
        Ok(val) => val,
        Err(err) => {
            error!("Url `{}` passed to http client cannot be parsed: `{}`", url, err);
            return Box::new(callback.send(Err(Error::Parse(format!("Cannot parse url `{}`", url)))).into_future().map(|_| ()).map_err(|_| ()))
        }
        };
        let mut req = hyper::Request::new(method, uri);

        if let Some(headers) = maybe_headers {
            mem::replace(req.headers_mut(), headers);
        }

        for body in maybe_body.iter() {
            req.set_body(body.clone());
        }

        let task = client.request(req)
        .map_err(|err| Error::Network(err))
        .and_then(move |res| {
            let status = res.status();
            let body_future: Box<future::Future<Item = String, Error = Error>> =
            Box::new(Self::read_body(res.body()).map_err(|err| Error::Network(err)));
            match status {
            hyper::StatusCode::Ok =>
                body_future,

            _ =>
                Box::new(
                body_future.and_then(move |body| {
                    let message = serde_json::from_str::<ErrorMessage>(&body).ok();
                    let error = Error::Api(status, message);
                    future::err(error)
                })
                )
            }
            })
            .then(|result| callback.send(result))
            .map(|_| ()).map_err(|_| ());

        Box::new(task)
    }

    fn read_body(body: hyper::Body) -> Box<Future<Item=String, Error=hyper::Error>> {
        Box::new(
            body
                .fold(Vec::new(), |mut acc, chunk| {
                    acc.extend_from_slice(&*chunk);
                    future::ok::<_, hyper::Error>(acc)
                })
                .and_then(|bytes| {
                    match String::from_utf8(bytes) {
                        Ok(data) => future::ok(data),
                        Err(err) => future::err(hyper::Error::Utf8(err.utf8_error()))
                    }
                })
        )
    }
}

#[derive(Clone)]
pub struct ClientHandle {
  tx: mpsc::Sender<Payload>,
  max_retries: usize,
}

impl ClientHandle {

    pub fn request<T>(&self, method: hyper::Method, url: String, body: Option<String>, headers: Option<hyper::Headers>) -> Box<Future<Item=T, Error=Error>>
        where T: for <'a> Deserialize<'a> + 'static
    {
        Box::new(
            self.send_request_with_retries(method, url, body, headers, None, self.max_retries)
                .and_then(|response| {
                    serde_json::from_str::<T>(&response)
                        .map_err(|err| Error::Parse(format!("{}", err)))
                })
        )
    }

    fn send_request_with_retries(&self, method: hyper::Method, url: String, body: Option<String>, headers: Option<hyper::Headers>, last_err: Option<Error>, retries: usize) -> Box<Future<Item=String, Error=Error>> {
        if retries == 0 {
            let error = last_err.unwrap_or(Error::Unknown("Unexpected missing error in send_request_with_retries".to_string()));
            Box::new(
                future::err(error)
            )
        } else {
            let self_clone = self.clone();
            let method_clone = method.clone();
            let body_clone = body.clone();
            let url_clone = url.clone();
            let headers_clone = headers.clone();
            Box::new(
                self.send_request(method, url, body, headers)
                    .or_else(move |err| {
                        match err {
                            Error::Network(err) => {
                                warn!("Failed to fetch `{}` with error `{}`, retrying... Retries left {}", url_clone, err, retries);
                                self_clone.send_request_with_retries(method_clone, url_clone, body_clone, headers_clone, Some(Error::Network(err)), retries - 1)
                            }
                            _ => Box::new(future::err(err))
                        }
                    })
            )

        }
    }

    fn send_request(&self, method: hyper::Method, url: String, body: Option<String>, headers: Option<hyper::Headers>) -> Box<Future<Item=String, Error=Error>> {
        info!("Starting outbound http request: {} {} with body {} and headers {}", method, url, body.clone().unwrap_or_default(), headers.clone().unwrap_or_default());
        let url_clone = url.clone();
        let method_clone = method.clone();

        let (tx, rx) = oneshot::channel::<ClientResult>();
        let payload = Payload {
            url,
            method,
            body,
            headers,
            callback: tx,
        };


        let future = self.tx.clone().send(payload)
        .map_err(|err| {
            Error::Unknown(format!("Unexpected error sending http client request params to channel: {}", err))
        })
        .and_then(|_| {
            rx.map_err(|err| {
                Error::Unknown(format!("Unexpected error receiving http client response from channel: {}", err))
            })
        })
        .and_then(|result| result)
        .map_err(move |err| {
            error!("{} {} : {}", method_clone, url_clone, err);
            err
        });

        Box::new(future)
    }
}

struct Payload {
    pub url: String,
    pub method: hyper::Method,
    pub body: Option<String>,
    pub headers: Option<hyper::Headers>,
    pub callback: oneshot::Sender<ClientResult>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorMessage {
    pub code: u16,
    pub message: String
}

#[derive(Debug)]
pub enum Error {
    Api(hyper::StatusCode, Option<ErrorMessage>),
    Network(hyper::Error),
    Parse(String),
    Unknown(String),
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Api(ref status, Some(ref error_message)) => {
                write!(f, "Http client 100: Api error: status: {}, code: {}, message: {}", status, error_message.code, error_message.message)
            },
            &Error::Api(status, None) => {
                write!(f, "Http client 100: Api error: status: {}", status)
            },
            &Error::Network(ref err) => {
                write!(f, "Http client 200: Network error: {:?}", err)
            },
            &Error::Parse(ref err) => {
                write!(f, "Http client 300: Parse error: {}", err)
            }
            &Error::Unknown(ref err) => {
                write!(f, "Http client 400: Unknown error: {}", err)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use hyper::mime;
    use hyper::{StatusCode, Response};
    use hyper::header::{ContentLength, ContentType};
    use tokio_core::reactor::Core;
    use serde_json;

    use ::models::NewStore;
    use ::controller::utils::{parse_body, read_body};

    #[test]
    fn test_read_body() {
        let message = "Ok".to_string();
        let message_str = serde_json::to_string(&message).unwrap();
        let res = response_with_body(message_str.clone());
        let body = res.body();
        let mut core = Core::new().unwrap();
        let work = read_body(body);
        let result = core.run(work).unwrap();
        assert_eq!(result, message_str);
    }

    #[test]
    fn test_parse_body() {
        let message = NewStore {
            name: "new store".to_string(),
            currency_id: 1,
            short_description: "short description".to_string(),
            long_description: None,
            slug: "myname".to_string(),
            cover: None,
            logo: None,
            phone: "1234567".to_string(),
            email: "example@mail.com".to_string(),
            address: "town city street".to_string(),
            facebook_url: None,
            twitter_url: None,
            instagram_url: None,
            pinterest_url: None,
        };
        let message_str = serde_json::to_string(&message).unwrap();
        let res = response_with_body(message_str.clone());
        let mut core = Core::new().unwrap();
        let work = parse_body::<NewStore>(res.body());
        let result = core.run(work).unwrap();
        assert_eq!(result.name, message.name);
    }

    fn response_with_body(body: String) -> Response {
        Response::new()
            .with_header(ContentLength(body.len() as u64))
            .with_header(ContentType(mime::APPLICATION_JSON))
            .with_status(StatusCode::Ok)
            .with_body(body)
    }
}
