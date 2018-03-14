//! AttributesSearch repo, presents CRUD operations with db for users
use std::convert::From;

use hyper::header::{ContentType, Headers};
use hyper::Method;
use future;
use futures::Future;
use futures::IntoFuture;
use serde_json;
use elastic_responses::{SearchResponse, UpdateResponse};
use stq_http::client::ClientHandle;

use models::{ElasticAttribute, ElasticIndex, IndexResponse, SearchAttribute};
use repos::error::RepoError as Error;
use repos::types::RepoFuture;

/// AttributesSearch repository, responsible for handling attributes
pub struct AttributesSearchRepoImpl {
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

pub trait AttributesSearchRepo {
    /// Find specific attribute by name
    fn find_by_name(&self, search_attribute: SearchAttribute) -> RepoFuture<ElasticAttribute>;

    /// Checks name exists
    fn name_exists(&self, name: String) -> RepoFuture<bool>;

    /// Creates new attribute
    fn create(&self, attribute: ElasticAttribute) -> RepoFuture<()>;

    /// Updates specific attribute
    fn update(&self, attribute: ElasticAttribute) -> RepoFuture<()>;
}

impl AttributesSearchRepoImpl {
    pub fn new(client_handle: ClientHandle, elastic_address: String) -> Self {
        Self {
            client_handle,
            elastic_address,
        }
    }
}

impl AttributesSearchRepo for AttributesSearchRepoImpl {
    /// Find specific attributes by name
    fn find_by_name(&self, search_attribute: SearchAttribute) -> RepoFuture<ElasticAttribute> {
        let query = json!({
            "query": {
                "bool": {
                    "must": {
                        "nested": {
                            "path": "name",
                            "query": {
                                "term": {
                                    "name.text": search_attribute.name
                                }
                            }
                        }
                    }
                }
            }
        }).to_string();

        let url = format!(
            "http://{}/{}/_doc/_search",
            self.elastic_address,
            ElasticIndex::Attribute
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticAttribute>>(Method::Get, url, Some(query), Some(headers))
                .map_err(Error::from)
                .and_then(|res| {
                    res.into_documents()
                        .next()
                        .ok_or(Error::NotFound)
                        .into_future()
                }),
        )
    }

    /// Checks name exists
    fn name_exists(&self, name: String) -> RepoFuture<bool> {
        let query = json!({
             "query": {
                "bool": {
                    "must": {
                        "nested": {
                            "path": "name",
                            "query": {
                                "term": {
                                    "name.text": name
                                }
                            }
                        }
                    }
                }
             }
        }).to_string();
        let url = format!(
            "http://{}/{}/_doc/_search",
            self.elastic_address,
            ElasticIndex::Attribute
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        Box::new(
            self.client_handle
                .request::<SearchResponse<ElasticAttribute>>(Method::Get, url, Some(query), Some(headers))
                .map_err(Error::from)
                .and_then(|res| future::ok(res.into_documents().next().is_some())),
        )
    }

    /// Creates new attribute
    fn create(&self, attribute: ElasticAttribute) -> RepoFuture<()> {
        let body = serde_json::to_string(&attribute).unwrap();
        let url = format!(
            "http://{}/{}/_doc/{}/_create",
            self.elastic_address,
            ElasticIndex::Attribute,
            attribute.id
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());

        Box::new(
            self.client_handle
                .request::<IndexResponse>(Method::Post, url, Some(body), Some(headers))
                .map_err(Error::from)
                .and_then(|res| {
                    if res.is_created() {
                        future::ok(())
                    } else {
                        future::err(Error::NotFound)
                    }
                }),
        )
    }

    /// Updates specific attribute
    fn update(&self, attribute: ElasticAttribute) -> RepoFuture<()> {
        let body = json!({
            "doc": attribute,
        }).to_string();
        let url = format!(
            "http://{}/{}/_doc/{}/_update",
            self.elastic_address,
            ElasticIndex::Attribute,
            attribute.id
        );
        let mut headers = Headers::new();
        headers.set(ContentType::json());

        Box::new(
            self.client_handle
                .request::<UpdateResponse>(Method::Post, url, Some(body), Some(headers))
                .map_err(Error::from)
                .and_then(|res| {
                    if res.updated() {
                        future::ok(())
                    } else {
                        future::err(Error::NotFound)
                    }
                }),
        )
    }
}
