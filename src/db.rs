
#[derive(sqlx::FromRow, Debug)]
pub struct Favorite {
    pub id: i64,
    pub path: String,
    pub created_at: chrono::NaiveDateTime
}

#[derive(sqlx::FromRow, Debug)]
pub struct Alias {
    pub id: i64,
    pub favorite_id: i64,
    pub name: String,
    pub created_at: chrono::NaiveDateTime
}