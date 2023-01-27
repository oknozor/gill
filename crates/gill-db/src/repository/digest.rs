use sqlx::{FromRow, PgPool};

#[derive(sqlx::FromRow, Debug)]
pub struct RepositoryDigest {
    pub id: i32,
    pub name: String,
    pub owner: String,
    pub domain: String,
    pub summary: Option<String>,
    pub star_count: Option<i64>,
    pub fork_count: Option<i64>,
    pub watch_count: Option<i64>,
    pub clone_url: String,
}

impl RepositoryDigest {
    pub async fn all_local(
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> sqlx::Result<Vec<RepositoryDigest>> {
        let repositories = sqlx::query_as!(
            RepositoryDigest,
            // language=PostgreSQL
            r#"
            SELECT r.id,
                   r.name,
                   u.username as owner,
                   r.domain,
                   r.summary,
                   COUNT(rs.repository_id) as star_count,
                   COUNT(rf.repository_id) as fork_count,
                   COUNT(rw.repository_id) as watch_count,
                   r.clone_uri as clone_url
            FROM repository r
                     RIGHT JOIN users u ON r.attributed_to = u.activity_pub_id
                     LEFT JOIN repository_star rs ON rs.repository_id = r.id
                     LEFT JOIN repository_fork rf ON rf.repository_id = r.id
                     LEFT JOIN repository_watch rw ON rw.repository_id = r.id
            WHERE NOT r.private AND r.is_local
            GROUP BY r.id, u.username, r.name, r.id, r.summary
            LIMIT $1 OFFSET $2;"#,
            limit,
            offset,
        )
        .fetch_all(db)
        .await?;

        Ok(repositories)
    }

    pub async fn all_federated(
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> sqlx::Result<Vec<RepositoryDigest>> {
        let repositories = sqlx::query_as!(
            RepositoryDigest,
            // language=PostgreSQL
            r#"
            SELECT r.id,
                   r.name,
                   u.username as owner,
                   r.domain,
                   r.summary,
                   COUNT(rs.repository_id) as star_count,
                   COUNT(rf.repository_id) as fork_count,
                   COUNT(rw.repository_id) as watch_count,
                   r.clone_uri as clone_url
            FROM repository r
                     RIGHT JOIN users u ON r.attributed_to = u.activity_pub_id
                     LEFT JOIN repository_star rs ON rs.repository_id = r.id
                     LEFT JOIN repository_fork rf ON rf.repository_id = r.id
                     LEFT JOIN repository_watch rw ON rw.repository_id = r.id
            WHERE NOT r.private AND NOT r.is_local
            GROUP BY r.id, u.username, r.name, r.id, r.summary
            LIMIT $1 OFFSET $2;"#,
            limit,
            offset,
        )
        .fetch_all(db)
        .await?;

        Ok(repositories)
    }
}

#[derive(FromRow)]
pub struct RepositoryLight {
    pub summary: Option<String>,
    pub star_count: Option<i64>,
    pub fork_count: Option<i64>,
    pub watch_count: Option<i64>,
    pub clone_url: String,
}

impl RepositoryLight {
    pub async fn stats_by_namespace(
        owner: &str,
        repository: &str,
        db: &PgPool,
    ) -> sqlx::Result<RepositoryLight> {
        let repositories = sqlx::query_as!(
            RepositoryLight,
            // language=PostgreSQL
            r#"
            SELECT r.summary,
                   COUNT(rs.repository_id) as star_count,
                   COUNT(rf.repository_id) as fork_count,
                   COUNT(rw.repository_id) as watch_count,
                   r.clone_uri as clone_url
            FROM repository r
                     RIGHT JOIN users u ON r.attributed_to = u.activity_pub_id
                     LEFT JOIN repository_watch rw ON rw.repository_id = r.id
                     LEFT JOIN repository_star rs ON rs.repository_id = r.id
                     LEFT JOIN repository_fork rf ON rf.repository_id = r.id
            WHERE NOT r.private AND u.username = $1 AND r.name = $2
            GROUP BY r.id, r.summary"#,
            owner,
            repository
        )
        .fetch_one(db)
        .await?;

        Ok(repositories)
    }
}
