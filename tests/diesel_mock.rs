extern crate diesel;
extern crate stores_lib;

use diesel::prelude::*;
use diesel::sql_query;
use stores_lib::config::Config;

pub type TestConnection = PgConnection;

pub fn connection() -> TestConnection {
    let config = Config::new().unwrap();
    let database_url = config.server.database;
    let conn = PgConnection::establish(&database_url).unwrap();
    conn.begin_test_transaction().unwrap();
    conn
}

pub fn connection_with_stores_db_with_stores_table() -> TestConnection {
    let connection = connection();
    sql_query("DROP TABLE IF EXISTS stores CASCADE")
        .execute(&connection)
        .unwrap();
    sql_query(
        "CREATE TABLE stores ( \
         id SERIAL PRIMARY KEY, \
         user_id INTEGER NOT NULL, \
         is_active BOOLEAN NOT NULL DEFAULT 't', \
         name JSONB NOT NULL, \
         short_description JSONB NOT NULL, \
         long_description VARCHAR, \
         slug VARCHAR UNIQUE NOT NULL, \
         cover VARCHAR, \
         logo VARCHAR, \
         phone VARCHAR, \
         email VARCHAR , \
         address VARCHAR , \
         facebook_url VARCHAR, \
         twitter_url VARCHAR, \
         instagram_url VARCHAR, \
         default_language VARCHAR NOT NULL, \
         slogan VARCHAR, \
         created_at TIMESTAMP NOT NULL DEFAULT current_timestamp, \
         updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp \
         )",
    ).execute(&connection)
        .unwrap();
    //use schema::users::dsl::*;
    // ::diesel::insert_into(users)
    //     .values(&vec![
    //         (id.eq(1), name.eq("Sean"), hair_color.eq("black")),
    //         (id.eq(2), name.eq("Tess"), hair_color.eq("brown")),
    //     ])
    //     .execute(&connection)
    //     .unwrap();
    connection
}
