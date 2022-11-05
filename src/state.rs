use std::path::Path;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use crate::config;

pub struct State {
    pub db_pool: sqlx::SqlitePool,
}

impl State {

    pub async fn init() -> Result<Self, sqlx::Error> {
        Self::init_with(&config::db_path()).await
    }

    pub async fn init_with(db_path: &impl AsRef<Path>) -> Result<Self, sqlx::Error> {
        let db_path = db_path.as_ref();
        
        let state = State {
            db_pool: {
                let connect = SqliteConnectOptions::new()
                .foreign_keys(true)
                .filename(db_path)
                
                .create_if_missing(true);

                SqlitePoolOptions::new()
                    .connect_with(connect)
                    .await?                
            }
        };

        sqlx::migrate!().run(&state.db_pool).await?;
        
        Ok(state)
    }
}