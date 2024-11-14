use crate::error::Result;
use chrono::{DateTime, Local};
use rusqlite::{params, Connection};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

#[derive(Debug)]
pub struct NoteRecord {
    pub id: i64,
    pub title: String,
    pub filename: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                filename TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Database { conn })
    }

    pub fn insert_note(&self, title: &str, filename: &str) -> Result<i64> {
        let now = Local::now();
        self.conn.execute(
            "INSERT INTO notes (title, filename, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![title, filename, now.to_rfc3339(), now.to_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all_notes(&self) -> Result<Vec<NoteRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, filename, created_at, updated_at FROM notes ORDER BY created_at DESC",
        )?;
        let notes = stmt
            .query_map([], |row| {
                Ok(NoteRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    filename: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Local),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Local),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(notes)
    }

    pub fn search_notes(&self, query: &str) -> Result<Vec<NoteRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, filename, created_at, updated_at 
             FROM notes 
             WHERE title LIKE ?1 OR filename LIKE ?1
             ORDER BY created_at DESC",
        )?;
        let search_pattern = format!("%{}%", query);
        let notes = stmt
            .query_map([search_pattern], |row| {
                Ok(NoteRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    filename: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Local),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Local),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(notes)
    }

    pub fn get_note(&self, id: i64) -> Result<Option<NoteRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, filename, created_at, updated_at FROM notes WHERE id = ?1",
        )?;
        let mut notes = stmt
            .query_map([id], |row| {
                Ok(NoteRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    filename: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Local),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Local),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(notes.pop())
    }

    pub fn delete_note(&self, id: i64) -> Result<bool> {
        let rows_affected = self.conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
        Ok(rows_affected > 0)
    }
}
