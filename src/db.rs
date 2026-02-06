use anyhow::Result;
use libsql::{params, Builder, Connection, Database};

use crate::{chunker::Chunk, paths};

pub struct SearchResult {
    pub file_path: String,
    pub line: usize,
    pub text: String,
    pub distance: f32,
}

fn vec_to_json(v: &[f32]) -> String {
    format!(
        "[{}]",
        v.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub async fn open_or_create(model_name: &str, dim: usize) -> Result<Database> {
    let db_path = paths::embeddings_db_path(model_name);
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let db = Builder::new_local(db_path).build().await?;
    let conn = db.connect()?;

    conn.execute_batch(&format!(
        "CREATE TABLE IF NOT EXISTS chunks (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            line INTEGER NOT NULL,
            text TEXT NOT NULL,
            embedding F32_BLOB({dim})
        );
        CREATE INDEX IF NOT EXISTS chunks_file_path ON chunks(file_path);
        CREATE INDEX IF NOT EXISTS chunks_hash ON chunks(content_hash);"
    ))
    .await?;

    conn.execute(
        &format!("CREATE INDEX IF NOT EXISTS chunks_vec_idx ON chunks(libsql_vector_idx(embedding, 'metric=cosine'))"),
        (),
    )
    .await?;

    Ok(db)
}

pub async fn get_cached_chunks(
    conn: &Connection,
    content_hash: &str,
) -> Result<Option<Vec<Chunk>>> {
    let mut rows = conn
        .query(
            "SELECT line, text, vector_extract(embedding) FROM chunks WHERE content_hash = ?1",
            [content_hash],
        )
        .await?;

    let mut chunks = Vec::new();
    while let Some(row) = rows.next().await? {
        let line: i64 = row.get(0)?;
        let text: String = row.get(1)?;
        let embedding_str: String = row.get(2)?;
        let embeddings = parse_vector_str(&embedding_str)?;
        chunks.push(Chunk {
            line: line as usize,
            text,
            embeddings,
        });
    }

    if chunks.is_empty() {
        Ok(None)
    } else {
        Ok(Some(chunks))
    }
}

fn parse_vector_str(s: &str) -> Result<Vec<f32>> {
    let trimmed = s.trim().trim_start_matches('[').trim_end_matches(']');
    let values: Vec<f32> = trimmed
        .split(',')
        .map(|v| v.trim().parse::<f32>())
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(values)
}

pub async fn store_file_chunks(
    conn: &Connection,
    file_path: &str,
    content_hash: &str,
    chunks: &[Chunk],
) -> Result<()> {
    let tx = conn.transaction().await?;

    tx.execute("DELETE FROM chunks WHERE file_path = ?1", [file_path])
        .await?;

    for chunk in chunks {
        let embedding_json = vec_to_json(&chunk.embeddings);
        tx.execute(
            "INSERT INTO chunks (id, file_path, content_hash, line, text, embedding) VALUES (NULL, ?1, ?2, ?3, ?4, vector32(?5))",
            params![file_path, content_hash, chunk.line as i64, chunk.text.as_str(), embedding_json],
        )
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn vector_search(
    conn: &Connection,
    query_embedding: &[f32],
    top_k: usize,
    dir_prefix: &str,
) -> Result<Vec<SearchResult>> {
    let query_json = vec_to_json(query_embedding);
    let fetch_k = top_k * 5;

    let mut rows = conn
        .query(
            "SELECT chunks.file_path, chunks.line, chunks.text, vector_distance_cos(chunks.embedding, vector32(?1)) AS distance
             FROM vector_top_k('chunks_vec_idx', vector32(?1), ?2) AS vt
             JOIN chunks ON chunks.rowid = vt.id
             WHERE chunks.file_path LIKE ?3",
            params![query_json, fetch_k as i64, format!("{}%", dir_prefix)],
        )
        .await?;

    let mut results = Vec::new();
    while let Some(row) = rows.next().await? {
        let file_path: String = row.get(0)?;
        let line: i64 = row.get(1)?;
        let text: String = row.get(2)?;
        let distance: f64 = row.get(3)?;
        results.push(SearchResult {
            file_path,
            line: line as usize,
            text,
            distance: distance as f32,
        });
        if results.len() >= top_k {
            break;
        }
    }

    Ok(results)
}

pub async fn remove_missing_files(
    conn: &Connection,
    existing_paths: &[String],
) -> Result<usize> {
    if existing_paths.is_empty() {
        let deleted = conn
            .execute("DELETE FROM chunks", ())
            .await?;
        return Ok(deleted as usize);
    }

    let placeholders: Vec<String> = (1..=existing_paths.len())
        .map(|i| format!("?{}", i))
        .collect();
    let sql = format!(
        "DELETE FROM chunks WHERE file_path NOT IN ({})",
        placeholders.join(",")
    );

    let params: Vec<libsql::Value> = existing_paths
        .iter()
        .map(|p| libsql::Value::Text(p.clone()))
        .collect();

    let deleted = conn.execute(&sql, params).await?;
    Ok(deleted as usize)
}

pub async fn get_all_file_paths(conn: &Connection) -> Result<Vec<String>> {
    let mut rows = conn
        .query("SELECT DISTINCT file_path FROM chunks", ())
        .await?;

    let mut paths = Vec::new();
    while let Some(row) = rows.next().await? {
        let path: String = row.get(0)?;
        paths.push(path);
    }
    Ok(paths)
}

pub async fn get_stats(conn: &Connection) -> Result<(usize, usize)> {
    let mut rows = conn
        .query(
            "SELECT COUNT(DISTINCT file_path), COUNT(*) FROM chunks",
            (),
        )
        .await?;

    if let Some(row) = rows.next().await? {
        let files: i64 = row.get(0)?;
        let chunks: i64 = row.get(1)?;
        Ok((files as usize, chunks as usize))
    } else {
        Ok((0, 0))
    }
}
