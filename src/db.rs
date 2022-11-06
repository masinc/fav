use crate::state::State;
use std::sync::Arc;

pub type FavoriteId = i64;
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Favorite {
    pub id: FavoriteId,
    pub path: String,
    pub created_at: chrono::NaiveDateTime,
}

impl Favorite {
    #[inline]
    pub async fn list_path(verbose: bool, state: Arc<State>) -> Result<Vec<String>, sqlx::Error> {
        if !verbose {
            let pathes: Vec<(String,)> = sqlx::query_as("SELECT path FROM favorites")
                .fetch_all(&state.db_pool)
                .await?;
            Ok(pathes.into_iter().map(|(path,)| path).collect())
        } else {
            todo!()
        }
    }
}

pub type AliasId = i64;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Alias {
    pub id: AliasId,
    pub favorite_id: FavoriteId,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug)]
pub enum AliasArg<'a> {
    Id(&'a AliasId),
    Name(&'a str),
    Alias(&'a Alias),
}

impl Alias {
    #[inline]
    pub async fn list_name(verbose: bool, state: Arc<State>) -> Result<Vec<String>, sqlx::Error> {
        if !verbose {
            let aliases: Vec<(String,)> = sqlx::query_as("SELECT name FROM aliases")
                .fetch_all(&state.db_pool)
                .await?;
            Ok(aliases.into_iter().map(|(alias,)| alias).collect())
        } else {
            todo!()
        }
    }

    pub async fn from_id(id: i64, state: Arc<State>) -> Result<Option<Self>, sqlx::Error> {
        let alias = sqlx::query_as::<_, Alias>("SELECT * FROM aliases WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.db_pool)
            .await?;
        Ok(alias)
    }

    pub async fn from_name(
        name: impl AsRef<str>,
        state: Arc<State>,
    ) -> Result<Option<Self>, sqlx::Error> {
        let alias = sqlx::query_as::<_, Alias>("SELECT * FROM aliases WHERE name = ?")
            .bind(name.as_ref())
            .fetch_optional(&state.db_pool)
            .await?;
        Ok(alias)
    }

    pub async fn get_favorite_path(
        arg: AliasArg<'_>,
        state: Arc<State>,
    ) -> Result<Option<String>, sqlx::Error> {
        let path = match arg {
            AliasArg::Id(id) => {
                let Some((favorite_id, )) =
                    sqlx::query_as::<_, (FavoriteId, )>("SELECT favorite_id FROM aliases WHERE id = ?")
                        .bind(id)
                        .fetch_optional(&state.db_pool)
                        .await?
                    else {
                        return Ok(None);
                    };

                sqlx::query_as::<_, (String,)>("SELECT path FROM favorites WHERE id = ?")
                    .bind(favorite_id)
                    .fetch_optional(&state.db_pool)
                    .await?
                    .map(|x| x.0)
            }
            AliasArg::Name(name) => {
                let Some((favorite_id,)) =
                    sqlx::query_as::<_, (FavoriteId, )>("SELECT favorite_id FROM aliases WHERE name = ?")
                        .bind(name)
                        .fetch_optional(&state.db_pool)
                        .await?
                    else {
                        return Ok(None);
                    };

                sqlx::query_as::<_, (String,)>("SELECT path FROM favorites WHERE id = ?")
                    .bind(favorite_id)
                    .fetch_optional(&state.db_pool)
                    .await?
                    .map(|x| x.0)
            }
            AliasArg::Alias(alias) => {
                sqlx::query_as::<_, (String,)>("SELECT path FROM favorites WHERE id = ?")
                    .bind(alias.favorite_id)
                    .fetch_optional(&state.db_pool)
                    .await?
                    .map(|x| x.0)
            }
        };

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_favorite_list_path(pool: sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        let state = Arc::new(State { db_pool: pool });

        let list = Favorite::list_path(false, Arc::clone(&state)).await?;
        assert!(list.is_empty());

        sqlx::query("INSERT INTO favorites (path) VALUES (?), (?), (?)")
            .bind("a")
            .bind("b")
            .bind("c")
            .execute(&state.db_pool)
            .await?;

        let list = Favorite::list_path(false, Arc::clone(&state)).await?;
        assert_eq!(&list[..], &["a", "b", "c"]);

        Ok(())
    }

    #[sqlx::test]
    async fn test_alias_list_name(pool: sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        let state = Arc::new(State { db_pool: pool });

        let list = Alias::list_name(false, Arc::clone(&state)).await?;
        assert!(list.is_empty());

        sqlx::query("INSERT INTO favorites (id, path) VALUES (1, 'xyz')")
            .execute(&state.db_pool)
            .await?;
        sqlx::query("INSERT INTO aliases (favorite_id, name) VALUES (1, 'a'), (1, 'b'), (1, 'c')")
            .execute(&state.db_pool)
            .await?;

        let list = Alias::list_name(false, Arc::clone(&state)).await?;
        assert_eq!(&list[..], &["a", "b", "c"]);

        Ok(())
    }

    #[sqlx::test]
    async fn test_alias_from_id(pool: sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        let state = Arc::new(State { db_pool: pool });

        let alias = Alias::from_id(10, Arc::clone(&state)).await?;
        assert!(alias.is_none());

        sqlx::query("INSERT INTO favorites (id, path) VALUES (99, 'xyz')")
            .execute(&state.db_pool)
            .await?;
        sqlx::query("INSERT INTO aliases (id, favorite_id, name) VALUES (10, 99, 'a')")
            .execute(&state.db_pool)
            .await?;

        let alias = Alias::from_id(10, Arc::clone(&state)).await?.unwrap();
        let right = Alias {
            id: 10,
            favorite_id: 99,
            name: "a".into(),
            // not check
            created_at: chrono::NaiveDateTime::default(),
        };

        assert_eq!(alias.id, right.id);
        assert_eq!(alias.favorite_id, right.favorite_id);
        assert_eq!(alias.name, right.name);

        Ok(())
    }

    #[sqlx::test]
    async fn test_alias_from_name(pool: sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        let state = Arc::new(State { db_pool: pool });

        let alias = Alias::from_id(10, Arc::clone(&state)).await?;
        assert!(alias.is_none());

        sqlx::query("INSERT INTO favorites (id, path) VALUES (99, 'xyz')")
            .execute(&state.db_pool)
            .await?;
        sqlx::query("INSERT INTO aliases (id, favorite_id, name) VALUES (10, 99, 'a')")
            .execute(&state.db_pool)
            .await?;

        let alias = Alias::from_name("a", Arc::clone(&state)).await?.unwrap();
        let right = Alias {
            id: 10,
            favorite_id: 99,
            name: "a".into(),
            // not check
            created_at: chrono::NaiveDateTime::default(),
        };

        assert_eq!(alias.id, right.id);
        assert_eq!(alias.favorite_id, right.favorite_id);
        assert_eq!(alias.name, right.name);

        Ok(())
    }

    #[sqlx::test]
    async fn test_alias_get_favorite_path_id(pool: sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        let state = Arc::new(State { db_pool: pool });

        let path = Alias::get_favorite_path(AliasArg::Id(&10), Arc::clone(&state)).await?;
        assert!(path.is_none());

        sqlx::query("INSERT INTO favorites (id, path) VALUES (99, 'xyz')")
            .execute(&state.db_pool)
            .await?;
        sqlx::query("INSERT INTO aliases (id, favorite_id, name) VALUES (10, 99, 'a')")
            .execute(&state.db_pool)
            .await?;

        let path = Alias::get_favorite_path(AliasArg::Id(&10), Arc::clone(&state)).await?;
        assert_eq!(path, Some("xyz".into()));

        Ok(())
    }

    #[sqlx::test]
    async fn test_alias_get_favorite_path_name(pool: sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        let state = Arc::new(State { db_pool: pool });

        let path = Alias::get_favorite_path(AliasArg::Name("a"), Arc::clone(&state)).await?;
        assert!(path.is_none());

        sqlx::query("INSERT INTO favorites (id, path) VALUES (99, 'xyz')")
            .execute(&state.db_pool)
            .await?;
        sqlx::query("INSERT INTO aliases (id, favorite_id, name) VALUES (10, 99, 'a')")
            .execute(&state.db_pool)
            .await?;

        let path = Alias::get_favorite_path(AliasArg::Name("a"), Arc::clone(&state)).await?;
        assert_eq!(path, Some("xyz".into()));

        Ok(())
    }

    #[sqlx::test]
    async fn test_alias_get_favorite_path_alias(pool: sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        let state = Arc::new(State { db_pool: pool });

        let alias = Alias {
            id: 10,
            favorite_id: 99,
            name: "a".into(),
            created_at: chrono::NaiveDateTime::default(),
        };

        let path = Alias::get_favorite_path(AliasArg::Alias(&alias), Arc::clone(&state)).await?;
        assert!(path.is_none());

        sqlx::query("INSERT INTO favorites (id, path) VALUES (99, 'xyz')")
            .execute(&state.db_pool)
            .await?;
        sqlx::query("INSERT INTO aliases (id, favorite_id, name) VALUES (10, 99, 'a')")
            .execute(&state.db_pool)
            .await?;

        let path = Alias::get_favorite_path(AliasArg::Alias(&alias), Arc::clone(&state)).await?;
        assert_eq!(path, Some("xyz".into()));

        Ok(())
    }
}
