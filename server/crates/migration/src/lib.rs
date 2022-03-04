use sea_orm::Database;
pub use sea_schema::migration::*;

mod m20220101_000001_create_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_create_table::Migration)]
    }
}

pub async fn run_db_migration(url: &str) -> Result<(), sea_orm::DbErr> {
    let conn = Database::connect(url).await?;
    Migrator::up(&conn, None).await?;

    Ok(())
}
