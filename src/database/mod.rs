extern crate sqlx;
use lazy_static;
use once_cell::sync::OnceCell;

use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct Database {
    db: sqlx::sqlite::SqlitePool,
}

pub struct RequestUser {
    pub name: String,
    pub user_id: i64,
}

impl Database {
    pub fn foo(&self) {
        println!("{:?}", self.db);
    }

    pub async fn add_user(&self, user: RequestUser) {
        let user_query = sqlx::query!(
            "INSERT into users (user_id, user_name) VALUES(?, ?)",
            user.user_id,
            user.name
        ).execute(&self.db)
        .await;
    }

    pub async fn add_gold_request(&self, amount: i64, user: RequestUser) {
        let user_query = sqlx::query!(
            "SELECT user_id, user_name FROM users WHERE user_id = ? ORDER BY user_id LIMIT 1",
            user.user_id
        )
        .fetch_one(&self.db)
        .await;
        match user_query {
            Err(sqlx::Error::RowNotFound) => {
                println!("Adding user {:?}", &user.name);
                self.add_user(user).await;
            },
            Ok(r) => {
                println!("Adding gold request for {} gold to user {}", amount, &r.user_id);
                let gold_add_query = sqlx::query!(
                    "INSERT into gold_requests (user_id, amount) VALUES(?, ?)",
                    r.user_id,
                    amount
                ).execute(&self.db)
                .await;
                match gold_add_query {
                    Err(why) => {
                        println!("Failed to add gold: {:?}", why);
                    },
                    Ok(r) => {
                        println!("Successfully added gold!");
                    }
                }
            },
            _ => ()
        };
        // println!("{:?}", user_query);
    }
}

pub async fn connect() -> Database {
    println!("Spinning up database connection");
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");
    Database { db: database }
}

pub static DATABASE: OnceCell<Database> = OnceCell::new();
