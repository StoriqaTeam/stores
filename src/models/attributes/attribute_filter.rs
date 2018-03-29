#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttributeFilter {
    pub id: i32,
    pub equal: Option<EqualFilter>,
    pub range: Option<RangeFilter>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EqualFilter {
    pub values: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RangeFilter {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}
