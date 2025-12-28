use std::ops::{Deref, DerefMut};

use figment::Figment;
use rbatis::{RBatis, table_sync::{self, ColumnMapper}};
use rocket_db_pools::{Connection, Database};
use crate::models;

#[derive(Clone, Debug)]
pub struct DatabasePool(pub(self) RBatis);

impl Deref for DatabasePool {
    type Target = RBatis;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DatabasePool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<DatabasePool> for RBatis {
    fn from(value: DatabasePool) -> Self {
        value.0
    }
}

impl DatabasePool {
    pub fn db(&self) -> RBatis {
        self.0.clone()
    }
}

#[rocket::async_trait]
impl rocket_db_pools::Pool for DatabasePool {
    type Connection = RBatis;
    type Error = crate::Error;

    async fn init(figment: &Figment) -> crate::Result<Self> {
        let config: crate::types::config::DatabaseConfig = figment.extract()?;
        let instance = RBatis::new();
        let mapper = match config.backend() {
            crate::types::config::DatabaseBackend::Sqlite => {instance.init(rbdc_sqlite::SqliteDriver {}, &config.url())?; &table_sync::SqliteTableMapper{} as &dyn ColumnMapper},
            crate::types::config::DatabaseBackend::Postgres => {instance.init(rbdc_pg::PostgresDriver {}, &config.url())?; &table_sync::PGTableMapper{} as &dyn ColumnMapper},
            crate::types::config::DatabaseBackend::Mysql => {instance.init(rbdc_mysql::MysqlDriver {}, &config.url())?; &table_sync::MssqlTableMapper{} as &dyn ColumnMapper},
        };

        models::LocalUser::sync(&instance, mapper).await?;
        models::Session::sync(&instance, mapper).await?;

        Ok(DatabasePool(instance))
    }

    async fn get(&self) -> crate::Result<Self::Connection> {
        Ok(self.0.clone())
    }

    async fn close(&self) {}
}

#[derive(Database)]
#[database("rbatis")]
pub struct AbyssalDb(DatabasePool);

pub type ORM = Connection<AbyssalDb>;