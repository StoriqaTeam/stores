use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use failure::Error as FailureError;
use failure::Fail;
use futures::future;
use futures::future::Either;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use r2d2::{self, Pool};
use rusoto_core::Region;
use tokio::timer::Interval;

use stq_static_resources::Language;

use config::{self, Config};
use errors::Error;
use models::CatalogWithAttributes;
use repos::legacy_acl::SystemACL;
use repos::{BaseProductsRepo, BaseProductsRepoImpl};

use loaders::rocket_models::{RocketRetailProduct, ToXMLDocument};
use loaders::services::s3::S3;

#[derive(Clone)]
pub struct RocketRetailLoader {
    busy: Arc<Mutex<bool>>,
    duration: Duration,
    db_pool: Pool<ConnectionManager<PgConnection>>,
    thread_pool: CpuPool,
    config: Option<config::RocketRetail>,
    s3: Option<Arc<S3>>,
}

impl RocketRetailLoader {
    const DEFAULT_DURATION: u64 = 3600;
    const DEFAULT_THREAD_COUNT: usize = 1;

    pub fn new(env: RocketRetailEnvironment) -> Self {
        let duration = RocketRetailLoader::set_duration(env.config.rocket_retail.as_ref());
        let thread_pool = RocketRetailLoader::set_thread_pool(env.config.rocket_retail.as_ref());

        let s3 = RocketRetailLoader::create_service_s3(env.config.s3.clone());

        RocketRetailLoader {
            busy: Arc::new(Mutex::new(false)),
            duration,
            db_pool: env.db_pool.clone(),
            thread_pool,
            config: env.config.rocket_retail.clone(),
            s3,
        }
    }

    pub fn start(self) -> impl Stream<Item = (), Error = FailureError> {
        info!("Rocket retail loader started.");
        let interval = Interval::new_interval(self.duration).map_err(|e| e.context("timer creation error").into());

        interval.and_then(move |_| {
            if self.config.as_ref().is_some() {
                let busy = *self.busy.lock().expect("Rocket retail loader: poisoned mutex at fetch step");
                if busy {
                    warn!("Rocket retail loader: tried to ping rocket retail loader, but it was busy");
                    Either::A(future::ok(()))
                } else {
                    Either::B(self.clone().make_step())
                }
            } else {
                info!("Rocket retail loader: disabled. Config section [rocket_retail] not set.");
                Either::A(future::ok(()))
            }
        })
    }

    fn make_step(self) -> impl Future<Item = (), Error = FailureError> {
        {
            let mut busy = self.busy.lock().expect("Rocket retail loader: poisoned mutex at fetch step");
            *busy = true;
        }

        let config = self.config.clone().expect("Can't load rocket_retail config!");
        let service_s3 = self.s3.clone().expect("Can't load S3 client!");
        let service2 = self.clone();
        let cluster = config.cluster.clone();
        let file_name = self.create_file_name(&config.file_name, &RocketRetailEnvironment::DEFAULT_LANG);

        self.use_transactions_repo(|repo| repo.get_all_catalog())
            .and_then(move |catalog_products| {
                debug!("Select {} catalog products from DB", catalog_products.len());
                let all: Vec<RocketRetailProduct> = catalog_products
                    .into_iter()
                    .flat_map(|catalog_with_attributes| {
                        let CatalogWithAttributes { base_product, variants } = catalog_with_attributes;

                        variants
                            .into_iter()
                            .map(|variant| {
                                RocketRetailProduct::new(
                                    base_product.clone(),
                                    variant,
                                    Some(RocketRetailEnvironment::DEFAULT_LANG),
                                    &cluster,
                                )
                            }).collect::<Vec<RocketRetailProduct>>()
                    }).collect();

                let mut data: Vec<u8> = vec![];
                all.to_xml_document()
                    .write(&mut data)
                    .and_then(|_| Ok(data))
                    .map_err(|e| e.context("Can't create xml document for rocket retail.").into())
            }).and_then(move |data| {
                debug!("Initial xml data for upload s3 store");

                service_s3.upload(&file_name, data).map_err(From::from)
            }).then(move |res| {
                let mut busy = service2.busy.lock().expect("Rocket retail loader: poisoned mutex at fetch step");
                *busy = false;
                res
            })
    }

    fn use_transactions_repo<F, T>(&self, f: F) -> impl Future<Item = T, Error = FailureError>
    where
        T: Send + 'static,
        F: FnOnce(BaseProductsRepoImpl<PgConnection>) -> Result<T, FailureError> + Send + 'static,
    {
        let self_clone = self.clone();
        self.thread_pool.spawn_fn(move || {
            self_clone.get_connection_blocking().and_then(|conn| {
                let repo = BaseProductsRepoImpl::new(&*conn, Box::new(SystemACL::default()));
                f(repo).map_err(|e| e.context(Error::Connection).into())
            })
        })
    }

    fn get_connection_blocking(&self) -> impl Future<Item = r2d2::PooledConnection<ConnectionManager<PgConnection>>, Error = FailureError> {
        future::result(self.db_pool.get()).map_err(|e| e.context(Error::Connection).into())
    }

    fn create_file_name(&self, file_name: &str, lang: &Language) -> String {
        format!("{}_{}.{}", file_name, lang, RocketRetailEnvironment::DEFAULT_FILE_EXTENSION)
    }

    fn set_duration(rocket_retail: Option<&config::RocketRetail>) -> Duration {
        match rocket_retail {
            Some(config) => Duration::from_secs(config.interval_s as u64),
            None => Duration::from_secs(RocketRetailLoader::DEFAULT_DURATION),
        }
    }

    fn set_thread_pool(rocket_retail: Option<&config::RocketRetail>) -> CpuPool {
        match rocket_retail {
            Some(config) => CpuPool::new(config.thread_count as usize),
            None => CpuPool::new(RocketRetailLoader::DEFAULT_THREAD_COUNT),
        }
    }

    fn create_service_s3(config_s3: Option<config::S3>) -> Option<Arc<S3>> {
        match config_s3 {
            Some(config) => {
                let config::S3 {
                    region,
                    key,
                    secret,
                    bucket,
                } = config;
                let region = region.parse::<Region>().expect("Invalid region specified");
                Some(Arc::new(S3::create(key, secret, region, bucket).expect("Can't create S3 client!")))
            }
            None => None,
        }
    }
}

#[derive(Clone)]
pub struct RocketRetailEnvironment {
    pub config: Arc<Config>,
    pub db_pool: Pool<ConnectionManager<PgConnection>>,
}

impl RocketRetailEnvironment {
    pub const DEFAULT_LANG: Language = Language::En;
    pub const DEFAULT_FILE_EXTENSION: &'static str = "xml";

    pub fn new(config: Config) -> Self {
        // Prepare database pool
        let database_url: String = config.server.database.parse().expect("Database URL must be set in configuration");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");

        Self {
            config: Arc::new(config),
            db_pool,
        }
    }
}
