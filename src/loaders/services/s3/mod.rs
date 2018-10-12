//! S3 service handles uploading static assets like images and videos to s3

pub mod client;
pub mod credentials;
pub mod error;

use std::sync::Arc;

use futures::future::Future;
use futures_cpupool::CpuPool;
use rusoto_core::request::{HttpClient, TlsError};
use rusoto_core::Region;
use rusoto_s3::S3Client as CrateS3Client;

use self::client::S3Client;
use self::error::S3Error;

/// S3 service
#[derive(Clone)]
pub struct S3 {
    inner: Arc<S3Client>,
    region: Region,
    bucket: String,
    cpu_pool: CpuPool,
}

impl S3 {
    /// Create s3 service
    ///
    /// * `bucket` - AWS s3 bucket name
    /// * `client` - client that implements S3Client trait
    pub fn new<B>(region: Region, bucket: B, client: Box<S3Client>) -> Self
    where
        B: ToString,
    {
        // s3 doesn't require a region
        Self {
            inner: client.into(),
            region,
            bucket: bucket.to_string(),
            cpu_pool: CpuPool::new_num_cpus(),
        }
    }

    /// Create s3 service
    ///
    /// * `key` - AWS key for s3 (from AWS console).
    /// * `secret` - AWS secret for s3 (from AWS console).
    /// * `bucket` - AWS s3 bucket name
    pub fn create<K, S, B>(key: K, secret: S, region: Region, bucket: B) -> Result<Self, TlsError>
    where
        K: ToString,
        S: ToString,
        B: ToString,
    {
        let credentials = credentials::Credentials::new(key.to_string(), secret.to_string());
        let client = HttpClient::new()?;
        Ok(Self::new(
            region.clone(),
            bucket,
            Box::new(CrateS3Client::new_with(client, credentials, region)),
        ))
    }

    pub fn upload(&self, name: &str, bytes: Vec<u8>) -> Box<Future<Item = (), Error = S3Error>> {
        info!("https://s3.{}.amazonaws.com/{}/{}", self.region.name(), self.bucket, name);

        self.inner
            .upload(self.bucket.clone(), name.to_string(), Some("text/xml".to_string()), bytes)
    }
}
