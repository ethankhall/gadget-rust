use sea_orm::Statement;
use sea_schema::migration::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = "
        CREATE TABLE external_user(
            user_id serial primary key,
            prefered_name VARCHAR (512) NOT NULL,
            external_user_id VARCHAR (512) NOT NULL,
            UNIQUE(external_user_id)
        );";
        let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
        manager.get_connection().execute(stmt).await.map(|_| ())?;

        let sql = "
        CREATE TABLE global_redirects(
            global_redirects_id serial primary key,
            public_ref VARCHAR (10) NOT NULL,
            alias VARCHAR (512) NOT NULL,
            destination VARCHAR(2048) NOT NULL,
            created_on TIMESTAMP NOT NULL,
            created_by_user_id int REFERENCES external_user(user_id) NOT NULL,
            UNIQUE(alias),
            UNIQUE(public_ref)
        );";
        let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
        manager.get_connection().execute(stmt).await.map(|_| ())?;

        let sql = "
        CREATE TABLE global_usage(
            usage_id serial primary key,
            global_redirects_id int REFERENCES global_redirects(global_redirects_id) NOT NULL,
            clicks int not null
        );";
        let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
        manager.get_connection().execute(stmt).await.map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = "DROP TABLE `global_usage`";
        let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
        manager.get_connection().execute(stmt).await.map(|_| ())?;

        let sql = "DROP TABLE `global_redirects`";
        let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
        manager.get_connection().execute(stmt).await.map(|_| ())
    }
}
