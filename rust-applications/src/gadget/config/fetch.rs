#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FetchConfig {
    pub fetch_url: String,
    pub fetch_id: String,
    pub frequency: i32,
}