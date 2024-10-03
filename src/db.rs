use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_query;
use dotenv::dotenv;
use std::env;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

pub fn create_database(database_name: &str) {
    dotenv().ok();
    let database_url = env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL must be set");

    let mut connection =
        PgConnection::establish(&database_url).expect("Failed to connect to Postgres");

    let create_db_query = format!(r#"CREATE DATABASE "{}";"#, database_name);
    sql_query(&create_db_query)
        .execute(&mut connection)
        .expect("Failed to create database");
    println!("Database '{}' created", database_name);
}
