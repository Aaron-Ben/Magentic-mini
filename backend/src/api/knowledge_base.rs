use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::Row;

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub model: String,
    pub chunk_size: i32,
    pub overlap_size: i32,
    pub document_count: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub filename: String,
    pub filetype: String,
    pub filesize: i64,
    pub created_at: i64,
}

impl KnowledgeBase {
    pub async fn create(pool: &PgPool, name: String, description: String) -> Result<Self> {
        let rec = sqlx::query(
            r#"
            INSERT INTO knowledge_bases (id, name, description, document_count, created_at, updated_at)
            VALUES ($1, $2, $3, 0, EXTRACT(EPOCH FROM NOW())::BIGINT, EXTRACT(EPOCH FROM NOW())::BIGINT)
            RETURNING id, name, description, document_count, created_at, updated_at
            "#,
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await?;

        Ok(Self {
            id: rec.get("id"),
            name: rec.get("name"),
            description: rec.get("description"),
            model: rec.get("model"),
            chunk_size: rec.get("chunk_size"),
            overlap_size: rec.get("overlap_size"),
            document_count: rec.get("document_count"),
            created_at: rec.get("created_at"),
            updated_at: rec.get("updated_at"),
        })
    }

    pub async fn find_by_id(pool: &PgPool, id: &str) -> Result<Option<Self>> {
        let rec = sqlx::query(
            r#"SELECT id, name, description, document_count, created_at, updated_at FROM knowledge_bases WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(rec.map(|r| Self {
            id: r.get("id"),
            name: r.get("name"),
            description: r.get("description"),
            model: r.get("model"),
            chunk_size: r.get("chunk_size"),
            overlap_size: r.get("overlap_size"),
            document_count: r.get("document_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    pub async fn update(pool: &PgPool, id: &str, name: Option<&str>, description: Option<&str>) -> Result<Option<Self>> {
        let rec = sqlx::query(
            r#"
            UPDATE knowledge_bases
            SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                updated_at = EXTRACT(EPOCH FROM NOW())::BIGINT
            WHERE id = $1
            RETURNING id, name, description, document_count, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .fetch_optional(pool)
        .await?;

        Ok(rec.map(|r| Self {
            id: r.get("id"),
            name: r.get("name"),
            description: r.get("description"),
            model: r.get("model"),
            chunk_size: r.get("chunk_size"),
            overlap_size: r.get("overlap_size"),
            document_count: r.get("document_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    pub async fn delete(pool: &PgPool, id: &str) -> Result<bool> {
        let rec = sqlx::query(
            r#"
            DELETE FROM knowledge_bases WHERE id = $1
            "#,
        )
            .bind(id)
            .execute(pool)
            .await?;
        Ok(rec.rows_affected() > 0)
    }

    pub async fn list(pool: &PgPool) -> Result<Vec<Self>> {
        let recs = sqlx::query(
            r#"SELECT id, name, description, document_count, created_at, updated_at FROM knowledge_bases ORDER BY created_at DESC"#,
        )
        .fetch_all(pool)
        .await?;
        Ok(recs.into_iter().map(|r| Self {
            id: r.get("id"),
            name: r.get("name"),
            description: r.get("description"),
            
            model: r.get("model"),
            chunk_size: r.get("chunk_size"),
            overlap_size: r.get("overlap_size"),
            document_count: r.get("document_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }
}

impl Document {
    pub async fn create(pool: &PgPool, knowledge_base_id: &str, filename: String, filetype: String, filesize: i64) -> Result<Self> {
        let rec = sqlx::query(
            r#"
            INSERT INTO documents (id, knowledge_base_id, filename, filetype, filesize, created_at)
            VALUES ($1, $2, $3, $4, $5, EXTRACT(EPOCH FROM NOW())::BIGINT)
            RETURNING id, filename, filetype, filesize, created_at
            "#,
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(knowledge_base_id)
        .bind(filename)
        .bind(filetype)
        .bind(filesize)
        .fetch_one(pool)
        .await?;

        Ok(Self {
            id: rec.get("id"),
            filename: rec.get("filename"),
            filetype: rec.get("filetype"),
            filesize: rec.get("filesize"),
            created_at: rec.get("created_at"),
        })
    }

    pub async fn find_by_id(pool: &PgPool, id: &str) -> Result<Option<Self>> {
        let rec = sqlx::query(
            r#"SELECT id, filename, filetype, filesize, created_at FROM documents WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(rec.map(|r| Self {
            id: r.get("id"),
            filename: r.get("filename"),
            filetype: r.get("filetype"),
            filesize: r.get("filesize"),
            created_at: r.get("created_at"),
        }))
    }

    pub async fn update(pool: &PgPool, id: &str, filename: Option<&str>, filetype: Option<&str>, filesize: Option<i64>) -> Result<Option<Self>> {
        let rec = sqlx::query(
            r#"
            UPDATE documents
            SET
                filename = COALESCE($2, filename),
                filetype = COALESCE($3, filetype),
                filesize = COALESCE($4, filesize),
                updated_at = EXTRACT(EPOCH FROM NOW())::BIGINT
            WHERE id = $1
            RETURNING id, filename, filetype, filesize, created_at
            "#,
        )
        .bind(id)
        .bind(filename)
        .bind(filetype)
        .bind(filesize)
        .fetch_optional(pool)
        .await?;

        Ok(rec.map(|r| Self {
            id: r.get("id"),
            filename: r.get("filename"),
            filetype: r.get("filetype"),
            filesize: r.get("filesize"),
            created_at: r.get("created_at"),
        }))
    }

    pub async fn delete(pool: &PgPool, id: &str) -> Result<bool> {
        let rec = sqlx::query(
            r#"
            DELETE FROM documents WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(rec.rows_affected() > 0)
    }

    pub async fn list(pool: &PgPool, knowledge_base_id: &str) -> Result<Vec<Self>> {
        let recs = sqlx::query(
            r#"SELECT id, filename, filetype, filesize, created_at FROM documents WHERE knowledge_base_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(knowledge_base_id)
        .fetch_all(pool)
        .await?;    
        Ok(recs.into_iter().map(|r| Self {
            id: r.get("id"),
            filename: r.get("filename"),
            filetype: r.get("filetype"),
            filesize: r.get("filesize"),
            created_at: r.get("created_at"),
        }).collect())
    }
}