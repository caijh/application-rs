use database_common::connection::DbConnection;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use tracing::log;

pub struct Dao {
    pub connection: DatabaseConnection,
}

impl Dao {
    pub async fn new(connection: DbConnection) -> Dao {
        let db_url = connection.to_string();
        let mut opt = ConnectOptions::new(db_url);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Debug);

        let db = Database::connect(opt)
            .await
            .expect("Could not connect to database");
        Dao { connection: db }
    }
}
