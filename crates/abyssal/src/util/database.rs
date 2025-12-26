use std::ops::{Deref, DerefMut};

use figment::Figment;
use rocket_db_pools::{Database, Pool};

#[derive(Clone, Debug)]
pub struct SeaPool(sea_orm::DatabaseConnection);

impl Deref for SeaPool {
    type Target = sea_orm::DatabaseConnection;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SeaPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Database)]
#[database("sea_orm")]
pub struct ORM(SeaPool);

#[rocket::async_trait]
impl Pool for SeaPool {
    type Connection = sea_orm::DatabaseConnection;
    type Error = crate::Error;

    async fn init(figment: &Figment) -> crate::Result<Self> {
        let conf: rocket_db_pools::Config = figment.extract()?;
        let pool = SeaPool(sea_orm::Database::connect(conf.url).await?);
        Ok(pool)
    }

    async fn get(&self) -> crate::Result<Self::Connection> {
        self.0.clone()
    }

    async fn close(&self) -> () {
        self.0.close().await.unwrap();
    }
}
