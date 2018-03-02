//! Module containing structs to work with languages and translations.
use std::fmt;
use std::str::FromStr;
use std::borrow::Cow;
use std::collections::HashMap;

use validator::ValidationError;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};

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

pub struct Translation {
    pub lang: Language,
    pub text: String,
}

impl Translation {
    pub fn new(lang: Language, text: String) -> Self {
        Self { lang, text }
    }
}

impl<'de> Deserialize<'de> for Translation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Lang,
            Text,
        }

        struct TranslationVisitor;

        impl<'de> Visitor<'de> for TranslationVisitor {
            type Value = Translation;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Translation")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Translation, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let lang = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let text = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(Translation::new(lang, text))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Translation, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut lang = None;
                let mut text = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Lang => {
                            if lang.is_some() {
                                return Err(de::Error::duplicate_field("lang"));
                            }
                            let val = map.next_value()?;
                            let language = Language::from_str(val).map_err(|_| de::Error::missing_field("lang"))?;
                            lang = Some(language);
                        }
                        Field::Text => {
                            if text.is_some() {
                                return Err(de::Error::duplicate_field("text"));
                            }
                            text = Some(map.next_value()?);
                        }
                    }
                }
                let lang = lang.ok_or_else(|| de::Error::missing_field("lang"))?;
                let text = text.ok_or_else(|| de::Error::missing_field("text"))?;
                Ok(Translation::new(lang, text))
            }
        }

        const FIELDS: &'static [&'static str] = &["lang", "text"];
        deserializer.deserialize_struct("Translation", FIELDS, TranslationVisitor)
    }
}
