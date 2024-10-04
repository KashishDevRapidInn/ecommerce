use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_query;
use dotenv::dotenv;
use std::env;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

/******************************************/
// Establishing Db Connection
/******************************************/
pub fn establish_connection() -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

/******************************************/
// Creating new db for tests
/******************************************/
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

/******************************************/
// Dropping db code
/******************************************/
pub fn drop_database(database_name: &str) {
    dotenv().ok();

    let default_db_url = env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL must be set");

    // Here I'm connecting to Postgres
    let mut connection = PgConnection::establish(&default_db_url)
        .expect("Failed to connect to the maintenance database");

    // My drop db logic wasn't working because I was trying to drop db which had active connection, so i need to dekete my active connections
    let terminate_query = format!(
        r#"
        SELECT pg_terminate_backend(pid) 
        FROM pg_stat_activity 
        WHERE datname = '{}';
    "#,
        database_name
    );

    if let Err(e) = sql_query(&terminate_query).execute(&mut connection) {
        eprintln!("Failed to terminate connections: {}", e);
        return;
    }

    // Dropping db
    let drop_query = format!(r#"DROP DATABASE IF EXISTS "{}";"#, database_name);

    if let Err(e) = sql_query(&drop_query).execute(&mut connection) {
        eprintln!("Failed to drop database: {}", e);
    } else {
        println!("Database '{}' dropped successfully.", database_name);
    }
}
