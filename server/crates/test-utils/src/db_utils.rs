pub use sea_orm::{entity::*, query::*, Database, DatabaseConnection, DbBackend};
use sqlx::postgres::PgPoolOptions;

pub async fn setup_schema() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&get_db_host_url())
        .await
        .unwrap();

    let mut conn = pool.acquire().await.unwrap();

    sqlx::query!("DROP DATABASE IF EXISTS postgres_test")
        .execute(&mut conn)
        .await
        .unwrap();

    sqlx::query!("CREATE DATABASE postgres_test")
        .execute(&mut conn)
        .await
        .unwrap();

    gadget_migration::run_db_migration(&get_db_url_with_test_db())
        .await
        .unwrap();
}

pub fn get_db_host_url() -> String {
    let host = std::env::var("TEST_DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    format!("postgresql://postgres:password@{}:5432", host)
}

pub fn get_db_url_with_test_db() -> String {
    format!("{}/postgres_test", get_db_host_url())
}
