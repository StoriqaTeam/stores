//! Module containing structs to work with languages and translations.
use std::fmt;
use std::str::FromStr;
use std::borrow::Cow;
use std::collections::HashMap;

use validator::ValidationError;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Ch,
    De,
    Ru,
    Es,
    Fr,
    Ko,
    Po,
    Ja,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lang = match *self {
            Language::En => "en",
            Language::Ch => "ch",
            Language::De => "de",
            Language::Ru => "ru",
            Language::Es => "es",
            Language::Fr => "fr",
            Language::Ko => "ko",
            Language::Po => "po",
            Language::Ja => "ja",
        };
        write!(f, "{}", lang)
    }
}

impl FromStr for Language {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "en" => Language::En,
            "ch" => Language::Ch,
            "de" => Language::De,
            "ru" => Language::Ru,
            "es" => Language::Es,
            "fr" => Language::Fr,
            "ko" => Language::Ko,
            "po" => Language::Po,
            "ja" => Language::Ja,
            _ => {
                return Err(ValidationError {
                    code: Cow::from("language"),
                    message: Some(Cow::from(
                        "Invalid language format. Language name must be ISO 639-1 format.",
                    )),
                    params: HashMap::new(),
                })
            }
        })
    }
}

#[derive(Deserialize)]
pub struct Translation {
    pub lang: Language,
    pub text: String,
}

impl Translation {
    pub fn new(lang: Language, text: String) -> Self {
        Self { lang, text }
    }
}
