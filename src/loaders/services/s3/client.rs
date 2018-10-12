//! Client for AWS S3

use futures::prelude::*;
use rusoto_s3::{PutObjectRequest, S3Client as CrateS3Client, StreamingBody, S3};

use super::error::S3Error;

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
