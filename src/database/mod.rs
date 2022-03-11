extern crate sqlx;

use lazy_static;
use async_once::AsyncOnce;

use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct Database {
    db: sqlx::sqlite::SqlitePool,
}

impl Database {
    pub fn foo(&self) {
        println!("{:?}", self.db);
    }
}

fn connect_sync() -> Database {
    Runtime::new().unwrap().block_on(async {
        let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
           .filename("database.sqlite")
           .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");
        Database {
            db: database
        }
    })
}

lazy_static! {
    pub static ref DATABASE : Database = connect_sync();
}
