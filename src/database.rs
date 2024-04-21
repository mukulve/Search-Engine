use anyhow::{Ok, Result};
use rusqlite::{params, Connection};
use std::collections::HashMap;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Self {
        let path = "search.db";
        let db = Connection::open(path).unwrap();
        db.execute("CREATE TABLE IF NOT EXISTS Website (WEBSITEID INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, URL VARCHAR(64) NOT NULL);", params![]).unwrap();
        db.execute("CREATE TABLE IF NOT EXISTS Word ( WORDID INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, COUNT INTEGER, WORD VARCHAR(64) NOT NULL, WEBSITEID INTEGER NOT NULL, FOREIGN KEY(WEBSITEID) REFERENCES Website(WEBSITEID));",params![],).unwrap();
        Self { conn: db }
    }

    pub fn insert_website(&self, url: &str) -> Result<i32> {
        self.conn
            .execute("INSERT INTO Website (URL) VALUES (?1)", params![url])?;
        //get the last inserted row id
        let website_id = self.conn.last_insert_rowid() as i32;
        Ok(website_id)
    }

    pub fn get_website_id(&self, url: &str) -> Result<i32> {
        let mut stmt = self
            .conn
            .prepare("SELECT WEBSITEID FROM Website WHERE URL = ?1")?;
        let mut rows = stmt.query(params![url])?;
        let website_id: i32 = rows.next()?.unwrap().get(0)?;
        Ok(website_id)
    }

    pub fn insert_word(&self, word: &str, website_id: i32, count: i32) -> Result<()> {
        self.conn.execute(
            "INSERT INTO Word (WORD, WEBSITEID, COUNT) VALUES (?1, ?2, ?3)",
            params![word, website_id, count],
        )?;
        Ok(())
    }

    pub fn search(&self, word: &str) -> Result<HashMap<String, i32>> {
        let mut results: HashMap<String, i32> = HashMap::new();

        let words = word
            .split_whitespace()
            .map(|f| f.to_lowercase())
            .collect::<Vec<String>>();

        let mut stmt = self
            .conn
            .prepare("SELECT URL, COUNT, WORD FROM Website INNER JOIN Word ON Website.WEBSITEID = Word.WEBSITEID WHERE Word.WORD LIKE ?1")?;

        for word in words {
            let fuzzy_word = format!("%{}%", word);
            let mut rows = stmt.query(params![fuzzy_word])?;

            while let Some(row) = rows.next()? {
                let url: String = row.get(0)?;
                let db_count: i32 = row.get(1)?;
                // let db_word: String = row.get(2)?;

                let count = results.entry(url).or_insert(0);
                *count += db_count;
            }
        }

        Ok(results)
    }
}
