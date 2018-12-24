use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::AsRef;

use isolang::Language;
use regex::Regex;
use serde_json;
use validator::validate_length;
use validator::ValidationError;
use validator::Validator;

use models::{Coupon, Store};
use stq_static_resources::Translation;
use stq_types::{CouponCode, ProductPrice};

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

pub fn validate_slug<T: AsRef<str>>(val: T) -> Result<(), ValidationError> {
    let val = val.as_ref();
    lazy_static! {
        static ref SLUG_VALIDATION_RE: Regex = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    }

    if SLUG_VALIDATION_RE.is_match(val) {
        Ok(())
    } else {
        Err(ValidationError {
            code: Cow::from("slug"),
            message: Some(Cow::from("Incorrect slug format")),
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

pub fn validate_not_empty<T: AsRef<str>>(val: T) -> Result<(), ValidationError> {
    if val.as_ref().trim().is_empty() {
        Err(ValidationError {
            code: Cow::from("value"),
            message: Some(Cow::from("Value must not be empty.")),
            params: HashMap::new(),
        })
    } else {
        Ok(())
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

pub fn validate_non_negative_price(price: &ProductPrice) -> Result<(), ValidationError> {
    validate_non_negative(price.0)
}

pub fn validate_non_negative_coupon_quantity(value: i32) -> Result<(), ValidationError> {
    validate_non_negative(value)
}

pub fn validate_coupon_code(val: &CouponCode) -> Result<(), ValidationError> {
    lazy_static! {
        static ref CODE_VALIDATION_RE: Regex = Regex::new(r"^[a-zA-Z0-9]*$").unwrap();
    }

    let validator_code = Validator::Length {
        min: Some(Coupon::MIN_LENGTH_CODE),
        max: Some(Coupon::MAX_LENGTH_CODE),
        equal: None,
    };

    let check_result = if validate_length(validator_code, &val.0) {
        if CODE_VALIDATION_RE.is_match(&val.0) {
            Ok(())
        } else {
            Err(ValidationError {
                code: Cow::from("code"),
                message: Some(Cow::from("Incorrect code format. Must be only (a-z,A-Z,0-9)")),
                params: HashMap::new(),
            })
        }
    } else {
        Err(ValidationError {
            code: Cow::from("code"),
            message: Some(Cow::from(format!(
                "Value must be >= {} and <= {} characters.",
                Coupon::MIN_LENGTH_CODE,
                Coupon::MAX_LENGTH_CODE
            ))),
            params: HashMap::new(),
        })
    };

    check_result
}

fn get_translations(text: &serde_json::Value) -> Result<Vec<Translation>, ValidationError> {
    serde_json::from_value::<Vec<Translation>>(text.clone()).map_err(|_| ValidationError {
        code: Cow::from("text"),
        message: Some(Cow::from("Invalid json format of text with translation.")),
        params: HashMap::new(),
    })
}

pub fn validate_store_short_description(text: &serde_json::Value) -> Result<(), ValidationError> {
    validate_translation(text)?;

    let translations = get_translations(text)?;

    for t in translations {
        if t.text.len() > Store::MAX_LENGTH_SHORT_DESCRIPTION {
            return Err(ValidationError {
                code: Cow::from("text"),
                message: Some(Cow::from(format!(
                    "Text inside translation must be <= {} characters.",
                    Store::MAX_LENGTH_SHORT_DESCRIPTION
                ))),
                params: HashMap::new(),
            });
        }
    }

    Ok(())
}

pub fn validate_translation(text: &serde_json::Value) -> Result<(), ValidationError> {
    let translations = get_translations(text)?;

    for t in translations {
        if t.text.is_empty() {
            return Err(ValidationError {
                code: Cow::from("text"),
                message: Some(Cow::from("Text inside translation must not be empty.")),
                params: HashMap::new(),
            });
        }
    }

    Ok(())
}

pub fn validate_urls(text: &serde_json::Value) -> Result<(), ValidationError> {
    serde_json::from_value::<Vec<String>>(text.clone()).map_err(|_| ValidationError {
        code: Cow::from("urls"),
        message: Some(Cow::from("Invalid format of urls. Must be json array of strings.")),
        params: HashMap::new(),
    })?;

    Ok(())
}
