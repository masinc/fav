use async_trait::async_trait;
use clap::{Args, Parser, Subcommand};
use normpath::PathExt;
use std::{io, path::PathBuf, sync::Arc};

use crate::State;

#[inline]
fn normalize_path(path: &str) -> io::Result<String> {
    Ok(PathBuf::from(path)
        .normalize_virtually()?
        .as_path()
        .display()
        .to_string())
}

#[inline]
async fn get_path(alias: &str, state: Arc<State>) -> Result<Option<String>, sqlx::Error> {
    let id: Option<(i64,)> = sqlx::query_as("SELECT id FROM aliases WHERE name = ?")
        .bind(alias)
        .fetch_optional(&state.db_pool)
        .await?;

    if let Some((id,)) = id {
        let (path,): (String,) = sqlx::query_as("SELECT path FROM favorites WHERE id = ?")
            .bind(id)
            .fetch_one(&state.db_pool)
            .await?;

        Ok(Some(path))
    } else {
        Ok(None)
    }
}

#[async_trait]
pub trait Action {
    async fn run(&self, state: Arc<crate::State>) -> anyhow::Result<()>;
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Add the path.
    #[command(visible_aliases = ["a"])]
    Add(CommandAdd),
    /// Get the path alias.
    #[command(visible_aliases = ["g"])]
    Get(CommandGet),
    /// Set the path alias.
    #[command(visible_aliases = ["s"])]
    Set(CommandSet),
    /// Remove the alias or path.
    #[command(visible_aliases = ["rm"])]
    Remove(CommandRemove),
    /// Resolve aliases
    Resolve(CommandResolve),
    /// Lists paths.
    #[command(visible_aliases = ["ls"])]
    List(CommandList),
}

#[async_trait]
impl Action for Commands {
    async fn run(&self, state: Arc<crate::State>) -> anyhow::Result<()> {
        use Commands::*;
        match self {
            Add(cmd) => cmd.run(state).await,
            Get(cmd) => cmd.run(state).await,
            Set(cmd) => cmd.run(state).await,
            Remove(cmd) => cmd.run(state).await,
            Resolve(cmd) => cmd.run(state).await,
            List(cmd) => cmd.run(state).await,
        }
    }
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum CommandType {
    /// Targeting Alias
    Alias,
    /// Targeting Path
    Path,
}

#[derive(Debug, Args)]
pub struct CommandAdd {
    #[clap(short = 'a', long = "alias")]
    pub aliases: Vec<String>,
    pub path: String,
}

#[async_trait]
impl Action for CommandAdd {
    async fn run(&self, state: Arc<crate::State>) -> anyhow::Result<()> {
        let path = normalize_path(&self.path)?;
        sqlx::query("INSERT INTO favorites (path) VALUES (?)")
            .bind(&path)
            .execute(&state.db_pool)
            .await?;

        CommandSet {
            aliases: self.aliases.clone(),
            path,
        }
        .run(state)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct CommandGet {
    /// Specify whether to get "Alias" or "Path".
    #[arg(name="type", short='t', long = "type", value_enum, default_value_t = CommandType::Alias)]
    pub _type: CommandType,

    pub value: String,
}

#[async_trait]
impl Action for CommandGet {
    async fn run(&self, state: Arc<crate::State>) -> anyhow::Result<()> {
        use CommandType::*;
        match self._type {
            Alias => {
                let path = normalize_path(&self.value)?;
                let aliases: Vec<(String,)> = sqlx::query_as("SELECT name FROM aliases WHERE favorite_id in (SELECT id FROM favorites WHERE path = ?)")
                    .bind(&path)
                    .fetch_all(&state.db_pool).await?;
                aliases.iter().for_each(|(name,)| println!("{name}"));
            }
            Path => match get_path(&self.value, Arc::clone(&state)).await? {
                Some(path) => println!("{}", path),
                None => eprintln!("{} was not found", self.value),
            },
        }
        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct CommandSet {
    #[arg(short = 'a', long = "alias")]
    pub aliases: Vec<String>,
    pub path: String,
}

#[async_trait]
impl Action for CommandSet {
    async fn run(&self, state: Arc<crate::State>) -> anyhow::Result<()> {
        let path = normalize_path(&self.path)?;

        if !self.aliases.is_empty() {
            let (id,): (i64,) = sqlx::query_as("SELECT id FROM favorites WHERE path = ?")
                .bind(&path)
                .fetch_one(&state.db_pool)
                .await?;
            for alias in self.aliases.iter() {
                sqlx::query("INSERT INTO aliases (favorite_id, name) VALUES (?, ?)")
                    .bind(id)
                    .bind(alias)
                    .execute(&state.db_pool)
                    .await?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct CommandRemove {
    /// Specify whether to delete "Alias" or "Path".
    #[arg(name="type", short='t', long = "type", value_enum, default_value_t = CommandType::Alias)]
    pub _type: CommandType,
    /// Deletes data according to the value of the "Type" argument
    pub values: Vec<String>,
}

#[async_trait]
impl Action for CommandRemove {
    async fn run(&self, state: Arc<crate::State>) -> anyhow::Result<()> {
        use CommandType::*;
        match self._type {
            Alias => {
                for alias in self.values.iter() {
                    sqlx::query("DELETE FROM aliases WHERE name = ?")
                        .bind(alias)
                        .execute(&state.db_pool)
                        .await?;
                }
            }
            Path => {
                for path in self.values.iter() {
                    let path = normalize_path(path)?;
                    sqlx::query("DELETE FROM paths WHERE path = ?")
                        .bind(path)
                        .execute(&state.db_pool)
                        .await?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct CommandResolve {
    pub names: Vec<String>,
}

#[async_trait]
impl Action for CommandResolve {
    async fn run(&self, state: Arc<crate::State>) -> anyhow::Result<()> {
        for name in self.names.iter() {
            match get_path(name, Arc::clone(&state)).await? {
                Some(path) => println!("{}", path),
                None => eprintln!("{} was not found", name),
            }
        }
        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct CommandList;

#[async_trait]
impl Action for CommandList {
    async fn run(&self, state: Arc<crate::State>) -> anyhow::Result<()> {
        let pathes: Vec<(String,)> = sqlx::query_as("SELECT path FROM favorites")
            .fetch_all(&state.db_pool)
            .await?;
        pathes.iter().for_each(|(path,)| println!("{path}"));
        Ok(())
    }
}
