use std::sync::Arc;
use cli::{Cli, Action};
use config::init_config;
use sqlx::sqlite::{self, SqliteConnectOptions};
use clap::Parser;

mod db;
mod cli;
mod config;

pub struct State {
    pub db_pool: sqlx::SqlitePool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_config()?;

    let state = Arc::new(State {
        db_pool: {
            let connect = SqliteConnectOptions::new()
                .foreign_keys(true)
                .filename(config::db_path())
                .create_if_missing(true);

            sqlite::SqlitePoolOptions::new()
                .connect_with(connect)
                .await?
        },
    });
    sqlx::migrate!().run(&state.db_pool).await?;

    let cli: Cli = Cli::parse();
    if let Some(cmd) = cli.cmd {
        cmd.run(Arc::clone(&state)).await?;
    }

    Ok(())
}
