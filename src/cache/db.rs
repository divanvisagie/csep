use std::path::{Path, PathBuf};
use anyhow::Result;
use rusqlite::{params, Connection};

use crate::chunker::Chunk;

pub struct CacheDB {
    conn: Connection,
}

pub fn get_db_path() -> PathBuf {
    dirs::cache_dir().unwrap().join("csep").join("embeddings.sqlite")
}

impl CacheDB {
    pub fn new() -> Result<Self> {
        let path = get_db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS embeddings (
                path TEXT,
                hash TEXT,
                model TEXT,
                line INTEGER,
                chunk TEXT,
                embedding BLOB
            )",
            [],
        )?;
        let db = CacheDB { conn };
        db.clean_removed_files()?;
        Ok(db)
    }

    pub fn get_chunks(&self, path: &str, hash: &str, model: &str) -> Result<Option<Vec<Chunk>>> {
        let mut stmt = self.conn.prepare(
            "SELECT line, chunk, embedding FROM embeddings WHERE path=?1 AND hash=?2 AND model=?3 ORDER BY line",
        )?;
        let rows = stmt.query_map(params![path, hash, model], |row| {
            let line: i64 = row.get(0)?;
            let text: String = row.get(1)?;
            let blob: Vec<u8> = row.get(2)?;
            let embeddings: Vec<f32> = bincode::deserialize(&blob).unwrap_or_default();
            Ok(Chunk { line: line as usize, text, embeddings })
        })?;
        let mut chunks = Vec::new();
        for row in rows {
            chunks.push(row?);
        }
        if chunks.is_empty() {
            Ok(None)
        } else {
            Ok(Some(chunks))
        }
    }

    pub fn upsert_chunks(&self, path: &str, hash: &str, model: &str, chunks: &[Chunk]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM embeddings WHERE path=?1 AND model=?2", params![path, model])?;
        let mut stmt = tx.prepare(
            "INSERT INTO embeddings (path, hash, model, line, chunk, embedding) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for chunk in chunks {
            let blob = bincode::serialize(&chunk.embeddings)?;
            stmt.execute(params![path, hash, model, chunk.line as i64, chunk.text, blob])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn delete_file(&self, path: &str) -> Result<()> {
        self.conn.execute("DELETE FROM embeddings WHERE path=?1", params![path])?;
        Ok(())
    }

    pub fn clean_removed_files(&self) -> Result<()> {
        let mut stmt = self.conn.prepare("SELECT DISTINCT path FROM embeddings")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        for row in rows {
            let path: String = row?;
            if !Path::new(&path).exists() {
                self.conn.execute("DELETE FROM embeddings WHERE path=?1", params![path])?;
            }
        }
        Ok(())
    }
}
