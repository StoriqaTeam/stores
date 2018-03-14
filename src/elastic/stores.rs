//! StoresSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentLength, ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use futures::future::*;
use serde_json;
use elastic_responses::SearchResponse;
use stq_http::client::ClientHandle;
use stq_static_resources::Translation;

use models::{ElasticIndex, ElasticStore, SearchStore};
use repos::error::RepoError as Error;
use repos::types::RepoFuture;

/// StoresSearch repository, responsible for handling stores
pub struct StoresElasticImpl {
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

pub trait StoresElastic {
    /// Find specific store by name limited by `count` parameters
    fn find_by_name(&self, search_store: SearchStore, count: i64, offset: i64) -> RepoFuture<Vec<ElasticStore>>;

    /// Auto complete
    fn auto_complete(&self, name: String, count: i64, offset: i64) -> RepoFuture<Vec<String>>;
}

impl StoresElasticImpl {
    pub fn new(client_handle: ClientHandle, elastic_address: String) -> Self {
        Self {
            client_handle,
            elastic_address,
        }
    }
}

impl StoresElastic for StoresElasticImpl {
    /// Find specific stores by name limited by `count` parameters
    fn find_by_name(&self, search_store: SearchStore, count: i64, offset: i64) -> RepoFuture<Vec<ElasticStore>> {
        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "nested" : {
                    "path" : "name",
                    "query" : {
                        "match": {
                            "name.text" : search_store.name
                        }
                    }
                }
            }
        }).to_string();
        let url = format!(
            "http://{}/{}/_doc/_search",
            self.elastic_address,
            ElasticIndex::Store
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticStore>>(Method::Get, url, Some(query), Some(headers))
                .map_err(Error::from)
                .and_then(|res| future::ok(res.into_documents().collect::<Vec<ElasticStore>>())),
        )
    }

    /// Auto Complete
    fn auto_complete(&self, name: String, count: i64, offset: i64) -> RepoFuture<Vec<String>> {
        let query = json!({
            "from" : offset, "size" : count,
            "query": {
                "nested" : {
                    "path" : "name",
                    "query" : {
                        "match": {
                            "name.text" : name
                        }
                    }
                }
            }
        }).to_string();
        let url = format!(
            "http://{}/{}/_doc/_search",
            self.elastic_address,
            ElasticIndex::Store
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(ContentLength(query.len() as u64));
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticStore>>(Method::Get, url, Some(query), Some(headers))
                .map_err(Error::from)
                .and_then(|res| {
                    res.into_documents()
                        .map(move |el_product| {
                            serde_json::from_value::<Vec<Translation>>(el_product.name)
                                .map_err(|e| Error::Unknown(e.into()))
                                .and_then(|translations| {
                                    translations
                                        .into_iter()
                                        .find(|transl| transl.text.contains(&name))
                                        .ok_or(Error::NotFound)
                                        .map(|t| t.text)
                                })
                        })
                        .collect::<Result<Vec<String>, Error>>()
                        .into_future()
                }),
        )
    }
}
