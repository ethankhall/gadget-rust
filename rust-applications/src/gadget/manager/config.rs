pub trait Fetcher {
    fn do_fetch(&self) -> Option<String>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Poller {
    Web(WebPoller)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManagerConfig {
    pub poller: Poller,
    pub frequency: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebPoller {
    pub fetch_url: String,
    pub fetch_id: String,
}

impl Fetcher for WebPoller {
    fn do_fetch(&self) -> Option<String> {
        let url = &self.fetch_url;
        let mut resp_body = match crate::HTTP_CLIENT.get(url).header("GADGET-FETCH-ID", self.fetch_id.clone()).send() {
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