use chrono::Utc;
use failure::Error as FailureError;

use crate::config::Config;
use crate::models::*;
use crate::stores_responses::*;
use crate::warehouses_responses::*;

pub struct CatalogProvider {
    config: Config,
    client: reqwest::Client,
}

impl CatalogProvider {
    pub fn with_config(config: Config) -> Result<CatalogProvider, FailureError> {
        Ok(CatalogProvider {
            config,
            client: reqwest::Client::new(),
        })
    }

    pub fn get_catalog_from_stores(&self) -> Result<CatalogResponse, FailureError> {
        let client = self.client.clone();
        let url = format!("{}/catalog", self.config.stores_microservice.url.clone());
        let mut response = client
            .get(url.as_str())
            .header("Currency", "STQ")
            .header("FiatCurrency", "USD")
            .send()?;
        let catalog: CatalogResponse = response.json()?;
        Ok(catalog)
    }

    pub fn get_stocks_from_warehouses(&self) -> Result<Vec<CatalogStocksResponse>, FailureError> {
        let client = self.client.clone();
        let url = format!("{}/stocks", self.config.warehouses_microservice.url.clone());
        let mut response = client.get(url.as_str()).send()?;
        let catalog: Vec<CatalogStocksResponse> = response.json()?;
        Ok(catalog)
    }

    pub fn get_rocket_retail_catalog(&self) -> Result<RocketRetailCatalog, FailureError> {
        let catalog = self.get_catalog_from_stores()?;
        let stocks = self.get_stocks_from_warehouses()?;
        let cluster = self.config.cluster.clone();

        let categories: Vec<_> = catalog
            .categories
            .clone()
            .into_iter()
            .map(|cat| RocketRetailCategory::new(cat, Some(DEFAULT_LANG)))
            .collect();

        let offers: Vec<_> = catalog
            .products
            .clone()
            .into_iter()
            .filter_map(|p| {
                let bp = catalog.find_base_product_by_id(p.base_product_id)?;
                let s = catalog.find_store_by_id(bp.store_id)?;
                let stock_quantity = stocks
                    .iter()
                    .filter(|st| st.product_id == p.id)
                    .map(|st| st.quantity.0)
                    .next()
                    .unwrap_or_default();

                Some(RocketRetailProduct::new(
                    bp.clone(),
                    s.clone(),
                    p.clone(),
                    catalog.find_prod_attrs_by_product_id(p.id),
                    Some(DEFAULT_LANG),
                    &cluster,
                    stock_quantity,
                ))
            })
            .collect();

        let date = Utc::now().format("%Y-%m-%d %H:%M").to_string();

        Ok(RocketRetailCatalog {
            date,
            shop: RocketRetailShop { categories, offers },
        })
    }
}
