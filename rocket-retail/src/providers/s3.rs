use std::str::FromStr;

use failure::Error as FailureError;
use futures::future::{ok, Future, FutureResult};
use rusoto_core::credential::{AwsCredentials, CredentialsError};
use rusoto_core::{HttpClient, ProvideAwsCredentials};
use rusoto_s3::{PutObjectRequest, S3Client as CrateS3Client, StreamingBody, S3};

use crate::config::{Config, S3 as S3Config};
use crate::errors::S3Error;
use crate::models::*;

pub struct S3Provider {
    config: Config,
    s3client: CrateS3Client,
}

impl S3Provider {
    pub fn with_config(config: Config) -> Result<S3Provider, FailureError> {
        let S3Config { key, secret, .. } = config.s3.clone();
        let http_client = HttpClient::new()?;
        let s3credentials = Credentials::new(key.to_string(), secret.to_string());
        let region = FromStr::from_str(config.s3.region.as_str())?;
        Ok(S3Provider {
            config: config.clone(),
            s3client: CrateS3Client::new_with(http_client, s3credentials, region),
        })
    }

    pub fn upload_catalog(&self, catalog: RocketRetailCatalog) -> impl Future<Item = String, Error = FailureError> {
        println!("Uploading config to S3...");
        let s3config = self.config.s3.clone();
        let file_name = format!("{}_{}.{}", self.config.file_name.clone(), DEFAULT_LANG, DEFAULT_FILE_EXTENSION);
        let data = {
            let mut data: Vec<u8> = vec![];
            catalog.to_xml_document().write(&mut data).unwrap();
            data
        };
        self.s3client
            .upload(s3config.bucket.clone(), file_name.clone(), Some("text/xml".to_string()), data)
            .map(move |_| {
                let url = format!(
                    "https://s3.{}.amazonaws.com/{}/{}",
                    s3config.region.clone(),
                    s3config.bucket.clone(),
                    file_name
                );
                url
            })
            .map_err(From::from)
    }
}

pub trait S3Client: Send + Sync + 'static {
    /// Uploads raw bytes to s3 with filename `key` and content-type (used for serving file from s3)
    fn upload(&self, bucket: String, key: String, content_type: Option<String>, bytes: Vec<u8>) -> Box<Future<Item = (), Error = S3Error>>;
}

impl S3Client for CrateS3Client {
    fn upload(&self, bucket: String, key: String, content_type: Option<String>, bytes: Vec<u8>) -> Box<Future<Item = (), Error = S3Error>> {
        let request = PutObjectRequest {
            acl: Some("public-read".to_string()),
            body: Some(StreamingBody::from(bytes)),
            bucket,
            key,
            content_type,
            ..Default::default()
        };

        Box::new(self.put_object(request).map(|_| ()).map_err(S3Error::from))
    }
}

pub struct Credentials {
    key: String,
    secret: String,
}

impl Credentials {
    pub fn new(key: String, secret: String) -> Self {
        Self { key, secret }
    }
}

impl ProvideAwsCredentials for Credentials {
    type Future = FutureResult<AwsCredentials, CredentialsError>;

    fn credentials(&self) -> Self::Future {
        ok(AwsCredentials::new(self.key.clone(), self.secret.clone(), None, None))
    }
}
