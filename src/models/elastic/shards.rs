#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Shards {
    total: u32,
    successful: u32,
    failed: u32,
}
