#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Filter {
    Equal(String),
    Lte(f32),
    Le(f32),
    Ge(f32),
    Gte(f32)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttributeFilter {
    pub name: String,
    pub filter: Filter
}