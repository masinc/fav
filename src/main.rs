use std::sync::Arc;
use cli::{Cli, Action};
use clap::Parser;
use state::State;

mod db;
mod cli;
mod config;
mod state;


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let cli: Cli = Cli::parse();
    if let Some(cli::Commands::Init(cmd)) = cli.cmd {
        return cmd.init().await;
    }

    let state = Arc::new(State::init().await?);

    if let Some(cmd) = cli.cmd {
        cmd.run(Arc::clone(&state)).await?;
    }

    Ok(())
}
