use std::fmt;

use failure::Error as FailureError;
use failure::Fail;
use serde_json;

use treexml::{Document, Element, ElementBuilder};

use stq_static_resources::{Language, Translation};
use stq_types::{ProductId, StoreId};

use errors::Error;
use models::{Attribute, BaseProduct, ProdAttr, ProductWithAttributes};

use loaders::RocketRetailEnvironment;

pub trait ToXMLElement {
    fn to_xml(self) -> Element;
}

pub trait ToXMLDocument {
    fn to_xml_document(self) -> Document;
}

trait BuildElement {
    /// Add child.
    fn with_child(self, child: Self) -> Self;

    fn with_child_text<S>(self, name: &str, value: S) -> Self
    where
        S: ToString + fmt::Display;

    fn with_child_option_text<S>(self, name: &str, value: Option<S>) -> Self
    where
        S: ToString + fmt::Display + Clone;
}

#[derive(Debug)]
pub struct DeliveryOption {
    pub cost: i32,
    pub days: u32,
    pub order_before: Option<u32>, // values 0-24
}

#[derive(Debug)]
pub struct DeliveryOptions {
    pub options: Vec<DeliveryOption>,
}

#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub unit: Option<String>,
    pub value: String,
}

impl Param {
    pub fn from_attribute(other: (ProdAttr, Attribute), lang: Language) -> Self {
        let (attribute_value, attribute) = other;

        let translation_names = get_translations(attribute.name.clone()).unwrap_or_default();
        let name = get_text_by_lang(&translation_names, lang.clone()).unwrap_or(format!("no name for language: {}", lang));

        Self {
            name,
            unit: None,
            value: attribute_value.value.into(),
        }
    }
}

impl ToXMLElement for Param {
    fn to_xml(self) -> Element {
        let mut elm = ElementBuilder::new("param").attr("name", self.name).text(self.value).element();

        if let Some(unit) = self.unit {
            elm.attributes.insert("unit".to_string(), unit);
        }

        elm
    }
}

/// Details a see [https://yandex.ru/support/partnermarket/offers.html](https://yandex.ru/support/partnermarket/offers.html)
#[derive(Debug, Default)]
pub struct RocketRetailProduct {
    pub id: String, // attribute
    pub name: String,
    pub model: Option<String>,
    pub vendor: Option<String>,
    pub vendor_code: Option<String>,
    pub available: Option<bool>, // attribute
    pub url: String,             // max 512 characters
    pub price: f64,
    pub oldprice: Option<f64>,
    pub currency_id: String,
    pub picture: String, // max 512 characters
    pub delivery: Option<bool>,
    pub delivery_options: Option<DeliveryOptions>,
    pub pickup: Option<bool>,
    pub store: Option<bool>,
    pub description: Option<String>, // max 300 characters
    pub sales_notes: Option<String>,
    pub manufacturer_warranty: Option<bool>,
    pub country_of_origin: Option<String>, // country name
    pub adult: Option<bool>,
    pub barcode: Option<String>,
    pub params: Vec<Param>,
    pub expiry: Option<String>, // format ISO8601
    pub weight: Option<f64>,
    pub dimensions: Option<f64>,
    pub downloadable: Option<bool>,
    pub age: Option<u32>,
    pub group_id: Option<String>,
}

impl RocketRetailProduct {
    pub fn new(base: BaseProduct, product_arg: ProductWithAttributes, lang_arg: Option<Language>, cluster: &str) -> Self {
        let lang = lang_arg.unwrap_or(RocketRetailEnvironment::DEFAULT_LANG);

        let translation_names = get_translations(base.name.clone()).unwrap_or_default();
        let name = get_text_by_lang(&translation_names, lang.clone()).unwrap_or(format!("no name for language: {}", lang));
        let ProductWithAttributes { product, attributes } = product_arg;

        let params = attributes.into_iter().map(|v| Param::from_attribute(v, lang.clone())).collect();

        Self {
            id: product.id.to_string(),
            name,
            url: create_product_url(cluster, base.store_id, product.id),
            price: product.price.into(),
            currency_id: product.currency.code().to_string(),
            picture: product.photo_main.unwrap_or("".to_string()),
            params,
            group_id: Some(base.id.to_string()),
            ..Default::default()
        }
    }
}

fn create_product_url(cluster: &str, store_id: StoreId, product_id: ProductId) -> String {
    format!("https://{}/store/{}/products/{}", cluster, store_id, product_id)
}

impl ToXMLDocument for Vec<RocketRetailProduct> {
    fn to_xml_document(self) -> Document {
        let mut root = Element::new("offers");

        for item in self {
            let offer = item.to_xml();
            root.children.push(offer);
        }

        Document {
            root: Some(root),
            ..Document::default()
        }
    }
}

impl ToXMLElement for RocketRetailProduct {
    fn to_xml(self) -> Element {
        let mut elm = ElementBuilder::new("offer");

        let RocketRetailProduct {
            id,
            name,
            vendor,
            vendor_code,
            available,
            url,
            price,
            oldprice,
            currency_id,
            picture,
            delivery,
            pickup,
            store,
            description,
            sales_notes,
            manufacturer_warranty,
            country_of_origin,
            adult,
            expiry,
            weight,
            dimensions,
            downloadable,
            age,
            params,
            group_id,
            ..
        } = self;

        elm.attr("id", id);

        if let Some(available) = available {
            elm.attr("available", available);
        }

        let mut elm = elm
            .element()
            .with_child_text("name", name)
            .with_child_option_text("vendor", vendor)
            .with_child_option_text("vendorCode", vendor_code)
            .with_child_text("url", url)
            .with_child_text("price", price)
            .with_child_option_text("oldprice", oldprice)
            .with_child_text("currencyId", currency_id)
            .with_child_text("picture", picture)
            .with_child_option_text("delivery", delivery)
            .with_child_option_text("pickup", pickup)
            .with_child_option_text("store", store)
            .with_child_option_text("description", description)
            .with_child_option_text("sales_notes", sales_notes)
            .with_child_option_text("manufacturer_warranty", manufacturer_warranty)
            .with_child_option_text("country_of_origin", country_of_origin)
            .with_child_option_text("adult", adult)
            .with_child_option_text("expiry", expiry)
            .with_child_option_text("weight", weight)
            .with_child_option_text("dimensions", dimensions)
            .with_child_option_text("downloadable", downloadable)
            .with_child_option_text("age", age)
            .with_child_option_text("group_id", group_id);

        for param in params.into_iter() {
            elm = elm.with_child(param.to_xml());
        }

        elm
    }
}

fn get_translations(value: serde_json::Value) -> Result<Vec<Translation>, FailureError> {
    let result = serde_json::from_value::<Vec<Translation>>(value)
        .map_err(|e| e.context("Can not parse Translation from value").context(Error::Parse))?;

    Ok(result)
}

fn get_text_by_lang(values: &[Translation], lang: Language) -> Option<String> {
    for item in values {
        if item.lang == lang {
            return Some(item.text.clone());
        }
    }

    None
}

impl BuildElement for Element {
    fn with_child(mut self, child: Self) -> Self {
        self.children.push(child);
        self
    }

    fn with_child_text<S>(self, name: &str, value: S) -> Self
    where
        S: ToString + fmt::Display,
    {
        self.with_child(ElementBuilder::new(name).text(value).element())
    }

    fn with_child_option_text<S>(self, name: &str, value: Option<S>) -> Self
    where
        S: ToString + fmt::Display + Clone,
    {
        if let Some(value) = value {
            return self.with_child(ElementBuilder::new(name).text(value.clone()).element());
        }

        self
    }
}
