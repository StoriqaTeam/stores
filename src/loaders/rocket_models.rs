use std::fmt::{self, Display, Error as FormatError, Formatter};

use failure::Error as FailureError;
use failure::Fail;
use serde_json;

use treexml::{Document, Element, ElementBuilder, XmlVersion};

use stq_static_resources::{Language, Translation};
use stq_types::{BaseProductId, StoreId};

use errors::Error;
use models::{Attribute, BaseProduct, ProdAttr, ProductWithAttributes, RawCategory};

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

/// For details, see [https://yandex.ru/support/partnermarket/export/yml.html#yml-format](https://yandex.ru/support/partnermarket/export/yml.html#yml-format)
#[derive(Debug, Default)]
pub struct RocketRetailCatalog {
    pub date: String,
    pub shop: RocketRetailShop,
}

impl ToXMLDocument for RocketRetailCatalog {
    fn to_xml_document(self) -> Document {
        Document {
            encoding: "UTF-8".to_string(),
            root: Some(self.to_xml()),
            version: XmlVersion::Version10,
        }
    }
}

impl ToXMLElement for RocketRetailCatalog {
    fn to_xml(self) -> Element {
        let mut bld = ElementBuilder::new("yml_catalog");
        bld.attr("date", self.date);
        bld.children(vec![&mut ElementBuilder::from(self.shop.to_xml())]);
        bld.element()
    }
}

#[derive(Debug, Default)]
pub struct RocketRetailShop {
    pub categories: Vec<RocketRetailCategory>,
    pub offers: Vec<RocketRetailProduct>,
}

impl ToXMLElement for RocketRetailShop {
    fn to_xml(self) -> Element {
        let mut categories_bld = ElementBuilder::new("categories");
        let mut category_blds = self
            .categories
            .into_iter()
            .map(|v| ElementBuilder::from(v.to_xml()))
            .collect::<Vec<_>>();
        categories_bld.children(category_blds.iter_mut().collect());

        let mut offers_bld = ElementBuilder::new("offers");
        let mut offer_blds = self
            .offers
            .into_iter()
            .map(|v| ElementBuilder::from(v.to_xml()))
            .collect::<Vec<_>>();
        offers_bld.children(offer_blds.iter_mut().collect());

        ElementBuilder::new("shop")
            .children(vec![&mut categories_bld, &mut offers_bld])
            .element()
    }
}

#[derive(Debug, Default)]
pub struct RocketRetailCategory {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub title: String,
}

impl ToXMLElement for RocketRetailCategory {
    fn to_xml(self) -> Element {
        let mut elm = ElementBuilder::new("category");
        elm.attr("id", self.id);
        if let Some(parent_id) = self.parent_id {
            elm.attr("parentid", parent_id);
        };
        elm.text(self.title);
        elm.element()
    }
}

impl RocketRetailCategory {
    pub fn from_raw_category(raw_category: RawCategory, lang_arg: Option<Language>) -> RocketRetailCategory {
        let lang = lang_arg.unwrap_or(RocketRetailEnvironment::DEFAULT_LANG);

        let RawCategory { id, parent_id, name, .. } = raw_category;

        let parent_id = parent_id.and_then(|parent_id| {
            let parent_id = parent_id.0;
            if parent_id != 0 {
                Some(parent_id)
            } else {
                None
            }
        });

        let category_translations = get_translations(name).unwrap_or_default();
        let title = get_text_by_lang(&category_translations, lang.clone()).unwrap_or(format!("no category title for language: {}", lang));

        RocketRetailCategory {
            id: id.0,
            parent_id,
            title,
        }
    }
}

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
    pub category_id: i32,
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
}

impl RocketRetailProduct {
    pub fn new(
        base: BaseProduct,
        store_name: serde_json::Value,
        product_arg: ProductWithAttributes,
        lang_arg: Option<Language>,
        cluster: &str,
    ) -> Self {
        let lang = lang_arg.unwrap_or(RocketRetailEnvironment::DEFAULT_LANG);

        let store_translations = get_translations(store_name).unwrap_or_default();
        let store_name = get_text_by_lang(&store_translations, lang.clone()).unwrap_or(format!("no store name for language: {}", lang));

        let translation_names = get_translations(base.name.clone()).unwrap_or_default();
        let name = get_text_by_lang(&translation_names, lang.clone()).unwrap_or(format!("no name for language: {}", lang));

        let descriptions = base.long_description.unwrap_or(base.short_description);
        let description_translations = get_translations(descriptions).unwrap_or_default();
        let description =
            get_text_by_lang(&description_translations, lang.clone()).unwrap_or(format!("no description for language: {}", lang));

        let ProductWithAttributes { product, attributes } = product_arg;

        let params = attributes.into_iter().map(|v| Param::from_attribute(v, lang.clone())).collect();
        let picture = product
            .photo_main
            .as_ref()
            .and_then(|photo_main| create_photo_url_from_product(photo_main, ImageSize::Medium))
            .unwrap_or_default();
        let url = create_product_url(cluster, base.store_id, base.id);

        Self {
            id: product.id.to_string(),
            name,
            description: Some(description),
            vendor: Some(store_name),
            model: Some(product.vendor_code),
            url,
            price: product.price.into(),
            category_id: base.category_id.0,
            currency_id: product.currency.code().to_string(),
            picture,
            params,
            ..Default::default()
        }
    }
}

fn create_product_url(cluster: &str, store_id: StoreId, base_product_id: BaseProductId) -> String {
    format!("https://{}/store/{}/products/{}", cluster, store_id, base_product_id)
}

fn create_photo_url_from_product(photo: &str, image_size: ImageSize) -> Option<String> {
    match image_size {
        ImageSize::Original => Some(photo.to_string()),
        _ => create_photo_url(photo, &image_size.to_string()),
    }
}

fn create_photo_url(photo_url: &str, image_size: &str) -> Option<String> {
    let mut parts_url = photo_url.split('/').collect::<Vec<_>>();
    let photo_name = parts_url.pop()?;
    let parts_name = photo_name.split('.').collect::<Vec<_>>();

    if parts_name.len() != 2 {
        debug!("cannot get photo name from string {}", photo_url);

        None
    } else {
        let file_name = parts_name.first()?;
        let ext = parts_name.last()?;
        let new_name = format!("{}-{}.{}", file_name, image_size, ext);

        Some(photo_url.replace(photo_name, new_name.as_str()))
    }
}

impl ToXMLElement for RocketRetailProduct {
    fn to_xml(self) -> Element {
        let mut elm = ElementBuilder::new("offer");

        let RocketRetailProduct {
            id,
            name,
            model,
            vendor,
            vendor_code,
            available,
            url,
            price,
            oldprice,
            category_id,
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
            ..
        } = self;

        elm.attr("id", id);

        if let Some(available) = available {
            elm.attr("available", available);
        } else {
            elm.attr("available", true);
        }

        let mut elm = elm
            .element()
            .with_child_text("name", name)
            .with_child_option_text("model", model)
            .with_child_option_text("vendor", vendor)
            .with_child_option_text("vendorCode", vendor_code)
            .with_child_text("url", url)
            .with_child_text("price", price)
            .with_child_option_text("oldprice", oldprice)
            .with_child_text("categoryId", category_id)
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
            .with_child_option_text("age", age);

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

pub enum ImageSize {
    Small,
    Medium,
    Large,
    Original,
}

impl Display for ImageSize {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        match self {
            &ImageSize::Small => f.write_str("small"),
            &ImageSize::Medium => f.write_str("medium"),
            &ImageSize::Large => f.write_str("large"),
            &ImageSize::Original => f.write_str("original"),
        }
    }
}
