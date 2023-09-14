use std::str::FromStr;

use chrono::{DateTime, Utc, NaiveDateTime};
use sqlitemapper::query;
use rusqlite::Connection;

#[derive(Debug)]
pub struct Timestamp(pub DateTime<Utc>);

impl FromStr for Timestamp {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
            .map(|dt| dt.and_utc())
            .map(Timestamp)
    }
}

impl ToString for Timestamp {
    fn to_string(&self) -> String {
        self.0.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

sqlitemapper::schema!{
    pub mod schema {
        mod users {
            type created_at = crate::Timestamp;
        }
    }
}

fn main() -> Result<(), rusqlite::Error> {
    let mut conn = Connection::open("database.db")?;

    let users = query!(schema, "SELECT * FROM users")
        .bind([])
        .query_all::<(_, _, _, _)>(&mut conn)?;

    for user in users {
        println!("{:?}", user);
    }

    Ok(())
}
