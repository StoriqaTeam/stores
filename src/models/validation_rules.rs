use std::borrow::Cow;
use std::collections::HashMap;

use serde_json;
use validator::ValidationError;
use regex::Regex;
use isolang::Language;
use super::Translation;

pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    lazy_static! {
        static ref PHONE_VALIDATION_RE: Regex = Regex::new(r"^\+?\d{7}\d*$").unwrap();
    }

    if PHONE_VALIDATION_RE.is_match(phone) {
        Ok(())
    } else {
        Err(ValidationError {
            code: Cow::from("phone"),
            message: Some(Cow::from("Incorrect phone format")),
            params: HashMap::new(),
        })
    }
}

pub fn validate_lang(lang: &str) -> Result<(), ValidationError> {
    match Language::from_639_1(lang) {
        None => Err(ValidationError {
            code: Cow::from("language"),
            message: Some(Cow::from("Value must be ISO 639-1 format.")),
            params: HashMap::new(),
        }),
        Some(_) => Ok(()),
    }
}

pub fn validate_non_negative<T: Into<f64>>(val: T) -> Result<(), ValidationError> {
    if val.into() > 0f64 {
        Ok(())
    } else {
        Err(ValidationError {
            code: Cow::from("value"),
            message: Some(Cow::from("Value must be non negative.")),
            params: HashMap::new(),
        })
    }
}

pub fn validate_translation(text: &serde_json::Value) -> Result<(), ValidationError> {
    serde_json::from_value::<Vec<Translation>>(text.clone()).map_err(|_| ValidationError {
        code: Cow::from("text"),
        message: Some(Cow::from("Invalid json format of text with translation.")),
        params: HashMap::new(),
    })?;

    Ok(())
}
