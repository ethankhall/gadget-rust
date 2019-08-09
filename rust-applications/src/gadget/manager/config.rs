use std::path::Path;

use azure_sdk_core::prelude::*;
use azure_sdk_storage_blob::prelude::*;
use azure_sdk_storage_core::prelude::*;
use tokio_core::reactor::Core;

use rusoto_core::{HttpClient, Region};
use rusoto_credential::StaticProvider;
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3, S3Client};

pub trait DataFetcher {
    fn do_fetch(&self) -> Option<String>;
}

pub trait DataPusher: DataFetcher {
    fn push_blob(&self, config_file: &Path);
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Fetcher {
    #[serde(rename = "web")]
    Web(WebPoller),
    #[serde(rename = "azure")]
    Azure(AzureBlob),
    #[serde(rename = "s3")]
    S3(S3Blob),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FetcherConfig {
    pub config_file: String,
    pub fetcher: Fetcher,
    pub frequency: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PusherConfig {
    pub pusher: Pusher,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Pusher {
    #[serde(rename = "azure")]
    Azure(AzureBlob),
    #[serde(rename = "s3")]
    S3(S3Blob),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct S3Blob {
    pub endpoint: String,
    pub secret: String,
    pub key: String,
    pub bucket: String,
    pub path: String,
}

impl DataPusher for S3Blob {
    fn push_blob(&self, config_path: &Path) {
        let region = Region::Custom {
            name: s!("us-east-1"),
            endpoint: s!(self.endpoint),
        };

        let credentials = StaticProvider::new(self.key.clone(), self.secret.clone(), None, None);
        let client = S3Client::new_with(HttpClient::new().unwrap(), credentials, region);

        let data = match std::fs::read(config_path) {
            Ok(body) => body,
            Err(e) => {
                warn!("📕 Unable to read config file: {}", e);
                return;
            }
        };

        let req = PutObjectRequest {
            bucket: self.bucket.clone(),
            key: self.path.clone(),
            body: Some(data.into()),
            ..Default::default()
        };

        match client.put_object(req).sync() {
            Ok(_) => {
                info!("✔️ Upload complete");
            }
            Err(e) => {
                error!("⚠️ Unable to upload blob! {}", e);
            }
        }
    }
}

impl DataFetcher for S3Blob {
    fn do_fetch(&self) -> Option<String> {
        use std::io::Read;

        let region = Region::Custom {
            name: s!("us-east-1"),
            endpoint: s!(self.endpoint),
        };

        let credentials = StaticProvider::new(self.key.clone(), self.secret.clone(), None, None);
        let client = S3Client::new_with(HttpClient::new().unwrap(), credentials, region);

        let get_req = GetObjectRequest {
            bucket: self.bucket.clone(),
            key: self.path.clone(),
            ..Default::default()
        };

        match client.get_object(get_req).sync() {
            Ok(result) => {
                let mut stream = result.body.unwrap().into_blocking_read();
                let mut body = Vec::new();
                stream.read_to_end(&mut body).unwrap();
                Some(String::from_utf8(body).expect("File to be valid"))
            }
            Err(e) => {
                warn!("Unable to parse data: {}", e);
                None
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AzureBlob {
    pub account: String,
    pub master_key: String,
    pub container_name: String,
    pub blob_name: String,
}

impl DataPusher for AzureBlob {
    fn push_blob(&self, config_path: &Path) {
        let mut core = Core::new().expect("To be able to create reactor");
        let client =
            Client::new(&self.account, &self.master_key).expect("To be able to make client");

        let data = match std::fs::read(config_path) {
            Ok(body) => body,
            Err(e) => {
                warn!("📕 Unable to read config file: {}", e);
                return;
            }
        };

        // this is not mandatory but it helps preventing
        // spurious data to be uploaded.
        let digest = md5::compute(&data[..]);

        // The required parameters are container_name, blob_name and body.
        let future = client
            .put_block_blob()
            .with_container_name(&self.container_name)
            .with_blob_name(&self.blob_name)
            .with_content_type("application/yaml")
            .with_body(&data[..])
            .with_content_md5(&digest[..])
            .finalize();

        match core.run(future) {
            Ok(_) => {
                info!("✔️ Upload complete");
            }
            Err(e) => {
                error!("⚠️ Unable to upload blob! {}", e);
            }
        };
    }
}

impl DataFetcher for AzureBlob {
    fn do_fetch(&self) -> Option<String> {
        let mut core = Core::new().expect("To be able to create reactor");
        let client =
            Client::new(&self.account, &self.master_key).expect("To be able to make client");
        let future = client
            .get_blob()
            .with_container_name(&self.container_name)
            .with_blob_name(&self.blob_name)
            .finalize();

        return match core.run(future) {
            Err(e) => {
                warn!("Unable to access blob: {}", e);
                None
            }
            Ok(response) => match String::from_utf8(response.data) {
                Ok(data) => Some(data),
                Err(e) => {
                    warn!("Unable to parse data: {}", e);
                    None
                }
            },
        };
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebPoller {
    pub fetch_url: String,
    pub fetch_id: String,
}

impl DataFetcher for WebPoller {
    fn do_fetch(&self) -> Option<String> {
        let url = &self.fetch_url;
        let mut resp_body = match crate::HTTP_CLIENT
            .get(url)
            .header("GADGET-FETCH-ID", self.fetch_id.clone())
            .send()
        {
            Ok(resp) => resp,
            Err(e) => {
                warn!("📮 Unable to pull configs: {}", e);
                return None;
            }
        };

        match resp_body.text() {
            Ok(text) => Some(text),
            Err(e) => {
                warn!("⚠️ Unable to get text: {}", e);
                None
            }
        }
    }
}
