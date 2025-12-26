use figment::Figment;
use rbatis::{RBatis, table_sync::{self, ColumnMapper}};
use rocket_db_pools::{Connection, Database};

#[derive(Clone, Debug)]
pub struct DatabasePool(RBatis);

#[rocket::async_trait]
impl rocket_db_pools::Pool for DatabasePool {
    type Connection = RBatis;
    type Error = crate::Error;

    async fn init(figment: &Figment) -> crate::Result<Self> {
        let config: crate::types::config::DatabaseConfig = figment.extract()?;
        let instance = RBatis::new();
        let _mapper = match config.backend() {
            crate::types::config::DatabaseBackend::Sqlite => {instance.init(rbdc_sqlite::SqliteDriver {}, &config.url())?; &table_sync::SqliteTableMapper{} as &dyn ColumnMapper},
            crate::types::config::DatabaseBackend::Postgres => {instance.init(rbdc_pg::PostgresDriver {}, &config.url())?; &table_sync::PGTableMapper{} as &dyn ColumnMapper},
            crate::types::config::DatabaseBackend::Mysql => {instance.init(rbdc_mysql::MysqlDriver {}, &config.url())?; &table_sync::MssqlTableMapper{} as &dyn ColumnMapper},
        };

        // TODO: Automatic table mapping

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