use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
pub enum Visibility {
    Active,
    Published,
}

impl FromStr for Visibility {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "active" => Ok(Visibility::Active),
            "published" => Ok(Visibility::Published),
            _ => Err(()),
        }
    }
}
