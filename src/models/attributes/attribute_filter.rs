#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttributeFilter {
    pub id: i32,
    pub equal: Option<EqualFilter>,
    pub range: Option<RangeFilter>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Hash, PartialEq)]
pub struct EqualFilter {
    pub values: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RangeFilter {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl RangeFilter {
    pub fn add_value(&mut self, value: f64) {
        if let Some(min) = self.min_value {
            if value < min {
                self.min_value = Some(value);
            }
        } else {
            self.min_value = Some(value)
        }

        if let Some(max) = self.max_value {
            if value > max {
                self.max_value = Some(value);
            }
        } else {
            self.max_value = Some(value)
        }
    }
}
