use std::str::FromStr;

use chrono::{DateTime, Utc};
use sqlitemapper::query;
use rusqlite::Connection;

pub struct Timestamp(pub DateTime<Utc>);

impl FromStr for Timestamp {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Timestamp(DateTime::parse_from_rfc3339(s)?
            .with_timezone(&Utc)))
    }
}

impl ToString for Timestamp {
    fn to_string(&self) -> String {
        self.0.to_rfc3339()
    }
}

sqlitemapper::schema!{
    pub mod schema {
    }
}

fn main() -> Result<(), rusqlite::Error> {
    let mut conn = Connection::open("database.db")?;

    // let users = __query!(nested::schema, "SELECT * FROM users")
    // let users = query!("SELECT * FROM users")
    // let users = query!(schema, "SELECT * FROM users");
    //     .bind([])
    //     .query_all(&mut conn)?;

    // for user in users {
    //     println!("{:?}", user);
    // }

    Ok(())
}
