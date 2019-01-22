use diesel::{pg::PgConnection, r2d2::ConnectionManager};
use failure::{Error as FailureError, Fail};
use futures::{future, Future, Stream};
use futures_cpupool::CpuPool;
use models::currency_exchange::NewCurrencyExchange;
use num_traits::{cast::ToPrimitive, Zero};
use r2d2::Pool;
use reqwest;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use stq_static_resources::currency::Currency;
use stq_types::newtypes::ExchangeRate;
use tokio::timer::Interval;

use repos::acl::legacy_acl::SystemACL;
use repos::currency_exchange::{CurrencyExchangeRepo, CurrencyExchangeRepoImpl};
use sentry::integrations::failure::capture_error;

#[derive(Clone)]
pub struct TickerContext {
    pub api_endpoint_url: String,
    pub http_client: reqwest::async::Client,
    pub db_pool: Pool<ConnectionManager<PgConnection>>,
    pub interval: Duration,
    pub thread_pool: CpuPool,
}

#[derive(Clone, Debug, Deserialize)]
struct ExmoCurrencyPairPayload {
    pub buy_price: Decimal,
    pub sell_price: Decimal,
    pub last_trade: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub avg: Decimal,
    pub vol: Decimal,
    pub vol_curr: Decimal,
    pub updated: u64,
}

#[derive(Clone, Debug)]
struct ExmoCurrencyPair {
    pub left_code: String,
    pub right_code: String,
    pub pair_info: ExmoCurrencyPairPayload,
}

#[derive(Clone, Debug, Default)]
struct ExmoCurrencyPairs(pub Vec<ExmoCurrencyPair>);

impl From<ExmoCurrencyPairs> for NewCurrencyExchange {
    fn from(pairs: ExmoCurrencyPairs) -> Self {
        let mut currency_exchange = pairs.0.iter().fold(NewCurrencyExchange::default(), |rates, pair| {
            let ExmoCurrencyPair {
                left_code,
                right_code,
                pair_info,
            } = pair;
            let ExmoCurrencyPairPayload { buy_price, sell_price, .. } = pair_info;

            // prevent division by 0
            if buy_price.is_zero() || sell_price.is_zero() {
                return rates;
            }

            // skip unused currency pairs
            let (left_code, right_code) = match (Currency::from_code(left_code), Currency::from_code(right_code)) {
                (Some(left_code), Some(right_code)) => (left_code, right_code),
                _ => {
                    return rates;
                }
            };

            let mut data = rates.data;
            {
                {
                    let left_currency_reversed_rates = data
                        .entry(left_code.clone())
                        .or_insert([(left_code.clone(), ExchangeRate(1.0))].iter().cloned().collect());

                    if left_code == right_code {
                        (*left_currency_reversed_rates).insert(right_code.clone(), ExchangeRate(1.0));
                    } else {
                        // buy price for the reversed exchange rate is calculated as an inverse of the sell price
                        if let Some(buy_price) = (Decimal::new(1, 0) / sell_price).to_f64() {
                            (*left_currency_reversed_rates).insert(right_code.clone(), ExchangeRate(buy_price));
                        }
                    }
                }
                {
                    let right_currency_reversed_rates = data
                        .entry(right_code.clone())
                        .or_insert([(right_code.clone(), ExchangeRate(1.0))].iter().cloned().collect());

                    if left_code == right_code {
                        (*right_currency_reversed_rates).insert(right_code.clone(), ExchangeRate(1.0));
                    } else {
                        if let Some(buy_price) = buy_price.to_f64() {
                            (*right_currency_reversed_rates).insert(left_code.clone(), ExchangeRate(buy_price));
                        }
                    }
                }
            }
            NewCurrencyExchange { data }
        });

        // Added ETH STQ pairs
        if let Some(usd) = currency_exchange.data.get(&Currency::USD).cloned() {
            let usd_eth = usd.get(&Currency::ETH).cloned().unwrap_or(ExchangeRate(1.0));
            let usd_stq = usd.get(&Currency::STQ).cloned().unwrap_or(ExchangeRate(1.0));
            {
                let mut stq = currency_exchange
                    .data
                    .entry(Currency::STQ)
                    .or_insert([(Currency::STQ, ExchangeRate(1.0))].iter().cloned().collect());
                stq.entry(Currency::ETH).or_insert(ExchangeRate(usd_eth.0 / usd_stq.0));
            }

            {
                let mut eth = currency_exchange
                    .data
                    .entry(Currency::ETH)
                    .or_insert([(Currency::ETH, ExchangeRate(1.0))].iter().cloned().collect());
                eth.entry(Currency::STQ).or_insert(ExchangeRate(usd_stq.0 / usd_eth.0));
            }
        }
        currency_exchange
    }
}

pub fn run(ctx: TickerContext) -> impl Future<Item = (), Error = FailureError> {
    Interval::new(Instant::now(), ctx.interval)
        .map_err(FailureError::from)
        .fold(ctx, |ctx, _| {
            info!("Started updating currency pairs");
            update_currency_pairs(ctx.clone()).then(|res| {
                match res {
                    Ok(_) => {
                        info!("Finished updating currency pairs");
                    }
                    Err(err) => {
                        let err = FailureError::from(err.context("An error occurred while updating currency pairs"));
                        error!("{:?}", &err);
                        capture_error(&err);
                    }
                };

                future::ok::<_, FailureError>(ctx)
            })
        })
        .map(|_| ())
}

fn update_currency_pairs(ctx: TickerContext) -> impl Future<Item = (), Error = FailureError> {
    let TickerContext {
        api_endpoint_url,
        http_client,
        ..
    } = ctx.clone();

    info!("Getting currency pairs from EXMO API...");
    http_client
        .get(api_endpoint_url.as_str())
        .send()
        .map_err(FailureError::from)
        .and_then(|mut res| {
            res.json::<serde_json::Value>()
                .map_err(|e| e.context("Received an invalid JSON from EXMO API").into())
                .and_then(|value| {
                    info!("Received a JSON response from EXMO API: {:?}", value);
                    serde_json::from_value::<HashMap<String, ExmoCurrencyPairPayload>>(value)
                        .map_err(|e| e.context("Unrecognized JSON response").into())
                })
        })
        .and_then(extract_rates)
        .map(NewCurrencyExchange::from)
        .and_then(|rates| update_rates_in_db(ctx, rates))
}

fn extract_rates(data: HashMap<String, ExmoCurrencyPairPayload>) -> Result<ExmoCurrencyPairs, FailureError> {
    data.iter()
        .map(|(pair_name, payload)| {
            let segments = pair_name.split("_").collect::<Vec<_>>();
            match segments.as_slice() {
                [left_code, right_code] => Ok(ExmoCurrencyPair {
                    left_code: left_code.to_string(),
                    right_code: right_code.to_string(),
                    pair_info: payload.clone(),
                }),
                _ => Err(format_err!("Failed to parse currency pair '{}'", pair_name)),
            }
        })
        .collect::<Result<Vec<_>, FailureError>>()
        .map(ExmoCurrencyPairs)
}

fn update_rates_in_db(ctx: TickerContext, rates: NewCurrencyExchange) -> impl Future<Item = (), Error = FailureError> {
    let TickerContext { db_pool, thread_pool, .. } = ctx;

    thread_pool.spawn(future::lazy(move || {
        let conn = db_pool.get().map_err(FailureError::from)?;
        let repo = CurrencyExchangeRepoImpl::new(&conn, Box::new(SystemACL::default()));
        repo.update(rates).map(|_| ())
    }))
}
