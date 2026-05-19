use anyhow::Result;
use sqlx::{SqlitePool, Row};
use chrono::Utc;
use crate::database::{Database, UrlRecord, SubdomainRecord, S3BucketRecord, SecurityFindingRecord, DatabaseStats};

pub struct SQLiteDatabase {
    pool: SqlitePool,
}

impl SQLiteDatabase {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        
        let db = Self { pool };
        db.init_tables().await?;
        
        Ok(db)
    }

    async fn init_tables(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS urls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                base_domain TEXT NOT NULL,
                source TEXT NOT NULL,
                status_code INTEGER,
                content_type TEXT,
                title TEXT,
                method TEXT NOT NULL,
                discovered_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS subdomains (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                subdomain TEXT NOT NULL,
                base_domain TEXT NOT NULL,
                source TEXT NOT NULL,
                discovered_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS s3_buckets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bucket_url TEXT NOT NULL,
                base_domain TEXT NOT NULL,
                source TEXT NOT NULL,
                verified BOOLEAN NOT NULL DEFAULT FALSE,
                discovered_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS security_findings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                finding_type TEXT NOT NULL,
                severity TEXT NOT NULL,
                description TEXT NOT NULL,
                url TEXT NOT NULL,
                evidence TEXT NOT NULL,
                recommendation TEXT,
                discovered_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_urls_domain ON urls(base_domain)"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_subdomains_domain ON subdomains(base_domain)"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_security_severity ON security_findings(severity)"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Database for SQLiteDatabase {
    async fn save_url(&mut self, url: &UrlRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO urls (url, base_domain, source, status_code, content_type, title, method, discovered_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&url.url)
        .bind(&url.base_domain)
        .bind(&url.source)
        .bind(url.status_code)
        .bind(&url.content_type)
        .bind(&url.title)
        .bind(&url.method)
        .bind(url.discovered_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_urls(&self, limit: Option<usize>) -> Result<Vec<UrlRecord>> {
        let mut query = "SELECT * FROM urls ORDER BY discovered_at DESC".to_string();
        
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        
        let mut urls = Vec::new();
        for row in rows {
            urls.push(UrlRecord {
                id: Some(row.get("id")),
                url: row.get("url"),
                base_domain: row.get("base_domain"),
                source: row.get("source"),
                status_code: row.get("status_code"),
                content_type: row.get("content_type"),
                title: row.get("title"),
                method: row.get("method"),
                discovered_at: row.get("discovered_at"),
            });
        }

        Ok(urls)
    }

    async fn save_subdomain(&mut self, subdomain: &SubdomainRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO subdomains (subdomain, base_domain, source, discovered_at)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(&subdomain.subdomain)
        .bind(&subdomain.base_domain)
        .bind(&subdomain.source)
        .bind(subdomain.discovered_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_subdomains(&self, domain: &str) -> Result<Vec<SubdomainRecord>> {
        let rows = sqlx::query(
            "SELECT * FROM subdomains WHERE base_domain = ? ORDER BY discovered_at DESC"
        )
        .bind(domain)
        .fetch_all(&self.pool)
        .await?;

        let mut subdomains = Vec::new();
        for row in rows {
            subdomains.push(SubdomainRecord {
                id: Some(row.get("id")),
                subdomain: row.get("subdomain"),
                base_domain: row.get("base_domain"),
                source: row.get("source"),
                discovered_at: row.get("discovered_at"),
            });
        }

        Ok(subdomains)
    }

    async fn save_s3_bucket(&mut self, bucket: &S3BucketRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO s3_buckets (bucket_url, base_domain, source, verified, discovered_at)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(&bucket.bucket_url)
        .bind(&bucket.base_domain)
        .bind(&bucket.source)
        .bind(bucket.verified)
        .bind(bucket.discovered_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_s3_buckets(&self) -> Result<Vec<S3BucketRecord>> {
        let rows = sqlx::query("SELECT * FROM s3_buckets ORDER BY discovered_at DESC")
            .fetch_all(&self.pool)
            .await?;

        let mut buckets = Vec::new();
        for row in rows {
            buckets.push(S3BucketRecord {
                id: Some(row.get("id")),
                bucket_url: row.get("bucket_url"),
                base_domain: row.get("base_domain"),
                source: row.get("source"),
                verified: row.get("verified"),
                discovered_at: row.get("discovered_at"),
            });
        }

        Ok(buckets)
    }

    async fn save_security_finding(&mut self, finding: &SecurityFindingRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO security_findings (finding_type, severity, description, url, evidence, recommendation, discovered_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&finding.finding_type)
        .bind(&finding.severity)
        .bind(&finding.description)
        .bind(&finding.url)
        .bind(&finding.evidence)
        .bind(&finding.recommendation)
        .bind(finding.discovered_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_security_findings(&self, severity: Option<&str>) -> Result<Vec<SecurityFindingRecord>> {
        let mut query = "SELECT * FROM security_findings".to_string();
        let mut params = Vec::new();

        if let Some(severity) = severity {
            query.push_str(" WHERE severity = ?");
            params.push(severity.to_string());
        }

        query.push_str(" ORDER BY discovered_at DESC");

        let mut q = sqlx::query(&query);
        for param in &params {
            q = q.bind(param);
        }

        let rows = q.fetch_all(&self.pool).await?;

        let mut findings = Vec::new();
        for row in rows {
            findings.push(SecurityFindingRecord {
                id: Some(row.get("id")),
                finding_type: row.get("finding_type"),
                severity: row.get("severity"),
                description: row.get("description"),
                url: row.get("url"),
                evidence: row.get("evidence"),
                recommendation: row.get("recommendation"),
                discovered_at: row.get("discovered_at"),
            });
        }

        Ok(findings)
    }

    async fn get_stats(&self) -> Result<DatabaseStats> {
        let urls_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM urls")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let subdomains_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM subdomains")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let s3_buckets_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM s3_buckets")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let security_findings_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_findings")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let unique_domains: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT base_domain) FROM urls")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let oldest_record: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
            "SELECT MIN(discovered_at) FROM urls"
        )
        .fetch_one(&self.pool)
        .await?;

        let newest_record: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
            "SELECT MAX(discovered_at) FROM urls"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseStats {
            total_urls: urls_count,
            total_subdomains: subdomains_count,
            total_s3_buckets: s3_buckets_count,
            total_security_findings: security_findings_count,
            unique_domains,
            oldest_record,
            newest_record,
        })
    }

    async fn cleanup_old_records(&mut self, days: u32) -> Result<usize> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days as i64);
        
        let result = sqlx::query("DELETE FROM urls WHERE discovered_at < ?")
            .bind(cutoff_date)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as usize)
    }
}
