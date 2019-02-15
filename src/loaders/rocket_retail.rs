use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use chrono::Utc;
use diesel::{pg::PgConnection, r2d2::ConnectionManager, Connection};
use failure::{Error as FailureError, Fail};
use futures::future;
    use futures::future::Either;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use r2d2::{self, Pool};
use rusoto_core::Region;
use tokio::timer::Interval;

use stq_cache::cache::NullCache;
use stq_static_resources::Language;
use stq_types::StoreId;

use config::{self, Config};
use errors::Error;
use models::{CatalogWithAttributes, Category, Visibility};
use repos::legacy_acl::SystemACL;
use repos::{
    categories::{category_cache::CategoryCacheImpl, CategoriesRepo},
    BaseProductsRepo, BaseProductsRepoImpl, CategoriesRepoImpl, StoresRepo, StoresRepoImpl,
};

use loaders::rocket_models::{RocketRetailCatalog, RocketRetailCategory, RocketRetailProduct, RocketRetailShop, ToXMLDocument};
use loaders::services::s3::S3;

#[derive(Clone, Debug, Fail)]
#[fail(display = "An impossible error occurred")]
pub struct ImpossibleError;

#[derive(Clone)]
pub struct RocketRetailLoader {
    busy: Arc<Mutex<bool>>,
    category_cache: NullCache<Category, ImpossibleError>,
    duration: Duration,
    db_pool: Pool<ConnectionManager<PgConnection>>,
    thread_pool: CpuPool,
    config: Option<config::RocketRetail>,
    s3: Option<Arc<S3>>,
}

struct RepoContext<'a> {
    pub base_products_repo: BaseProductsRepoImpl<'a, PgConnection>,
    pub categories_repo: CategoriesRepoImpl<'a, NullCache<Category, ImpossibleError>, PgConnection>,
    pub stores_repo: StoresRepoImpl<'a, PgConnection>,
}

impl RocketRetailLoader {
    const DEFAULT_DURATION: u64 = 3600;
    const DEFAULT_THREAD_COUNT: usize = 1;

    pub fn new(env: RocketRetailEnvironment) -> Self {
        let RocketRetailEnvironment {
            category_cache,
            config,
            db_pool,
        } = env;

        let Config { rocket_retail, s3, .. } = config.as_ref();

        let duration = RocketRetailLoader::set_duration(rocket_retail.as_ref());
        let thread_pool = RocketRetailLoader::set_thread_pool(rocket_retail.as_ref());
        let s3 = RocketRetailLoader::create_service_s3(s3.clone());

        RocketRetailLoader {
            busy: Arc::new(Mutex::new(false)),
            category_cache,
            duration,
            db_pool,
            thread_pool,
            config: rocket_retail.clone(),
            s3,
        }
    }

    pub fn start(self) -> impl Stream<Item = (), Error = FailureError> {
        info!("Rocket retail loader started.");
        let interval = Interval::new(Instant::now(), self.duration).map_err(|e| e.context("timer creation error").into());

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
                info!("Rocket retail loader: disabled. Config section [catalogs] not set.");
                Either::A(future::ok(()))
            }
        })
    }

    fn make_step(self) -> impl Future<Item = (), Error = FailureError> {
        {
            let mut busy = self.busy.lock().expect("Rocket retail loader: poisoned mutex at fetch step");
            *busy = true;
        }

        let config = self.config.clone().expect("Can't load catalogs config!");
        let service_s3 = self.s3.clone().expect("Can't load S3 client!");
        let service2 = self.clone();
        let cluster = config.cluster.clone();
        let file_name = self.create_file_name(&config.file_name, &RocketRetailEnvironment::DEFAULT_LANG);

        self.use_transactions_repo(move |ctx| {
            let RepoContext {
                base_products_repo,
                categories_repo,
                stores_repo,
            } = ctx;

            let catalog = base_products_repo.get_all_catalog()?;
            let stores_count = stores_repo.count(Visibility::Published)? as i32;
            let stores = stores_repo.list(StoreId(0), stores_count, Visibility::Published)?;

            let raw_categories = categories_repo.get_raw_categories()?;
            let categories = raw_categories
                .into_iter()
                .map(|raw_cat| RocketRetailCategory::from_raw_category(raw_cat, Some(RocketRetailEnvironment::DEFAULT_LANG)))
                .collect::<Vec<_>>();

            let catalog_products = catalog
                .into_iter()
                .filter_map(|cp| {
                    let CatalogWithAttributes { base_product, variants } = cp.clone();
                    stores.iter().find(|s| cp.base_product.store_id == s.id).map(|s| {
                        variants
                            .into_iter()
                            .map(|variant| {
                                RocketRetailProduct::new(
                                    base_product.clone(),
                                    s.name.clone(),
                                    variant,
                                    Some(RocketRetailEnvironment::DEFAULT_LANG),
                                    &cluster,
                                )
                            })
                            .collect::<Vec<RocketRetailProduct>>()
                    })
                })
                .flatten()
                .collect::<Vec<_>>();

            let date = Utc::now().format("%Y-%m-%d %H:%M").to_string();

            Ok(RocketRetailCatalog {
                date,
                shop: RocketRetailShop {
                    categories,
                    offers: catalog_products,
                },
            })
        })
        .and_then(move |catalog| {
            debug!("Creating XML document");
            let mut data: Vec<u8> = vec![];
            catalog
                .to_xml_document()
                .write(&mut data)
                .map(|_| data)
                .map_err(|e| e.context("Failed to create xml document for rocket retail.").into())
        })
        .and_then(move |data| {
            debug!("Initiated S3 upload");
            service_s3.upload(&file_name, data).map_err(From::from)
        })
        .then(move |res| {
            let mut busy = service2.busy.lock().expect("Rocket retail loader: poisoned mutex at fetch step");
            *busy = false;
            res
        })
    }

    fn use_transactions_repo<F, T>(&self, f: F) -> impl Future<Item = T, Error = FailureError>
    where
        T: Send + 'static,
        F: FnOnce(RepoContext) -> Result<T, FailureError> + Send + 'static,
    {
        let self_clone = self.clone();
        self.thread_pool.spawn_fn(move || {
            let conn = self_clone.db_pool.get().map_err(|e| e.context(Error::Connection))?;
            let category_cache = CategoryCacheImpl::new(self_clone.category_cache);

            let base_products_repo = BaseProductsRepoImpl::new(&*conn, Box::new(SystemACL::default()));
            let categories_repo = CategoriesRepoImpl::new(&*conn, Box::new(SystemACL::default()), Arc::new(category_cache));
            let stores_repo = StoresRepoImpl::new(&*conn, Box::new(SystemACL::default()));

            let repo_context = RepoContext {
                base_products_repo,
                categories_repo,
                stores_repo,
            };

            conn.transaction::<T, FailureError, _>(move || f(repo_context).map_err(|e| e.context(Error::Connection).into()))
        })
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
    pub category_cache: NullCache<Category, ImpossibleError>,
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
            category_cache: NullCache::new(),
            db_pool,
        }
    }
}
