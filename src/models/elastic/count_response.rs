use models::elastic::Shards;

#[derive(Deserialize, Debug)]
pub struct CountResponse {
    count: u64,
    #[serde(rename = "_shards")]
    shards: Shards,
}

impl CountResponse {
    pub fn get_count(&self) -> u64 {
        self.count
    }
}
