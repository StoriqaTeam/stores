#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Filter {
    Equal(String),
    Lte(f32),
    Gte(f32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttributeFilter {
    pub id: i32,
    pub filter: Filter,
}
