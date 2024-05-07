use rusqlite::Error;

pub struct Insert<Record> {
    sql: &'static str,
    record: Vec<Record>,
}

impl<Record> Insert<Record> {
    pub fn new_unchecked(sql: &'static str, record: Record) -> Self {
        Insert { sql, record }
    }

    pub fn execute(self, conn: &mut Connection) -> Result<(), Error> {}
}

// impl<Record> Insert<Record> where Record::Table: Rowid
