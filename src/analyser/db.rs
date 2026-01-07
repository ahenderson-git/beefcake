use crate::analyser::logic::{ColumnSummary, FileHealth};
use anyhow::{Context as _, Result};
use polars::prelude::*;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Pool, Postgres};

pub struct DbClient {
    pool: Pool<Postgres>,
}

pub struct AnalysisPush<'a> {
    pub file_path: &'a str,
    pub file_size: u64,
    pub health: &'a FileHealth,
    pub summaries: &'a [ColumnSummary],
    pub df: &'a DataFrame,
    pub schema_name: Option<&'a str>,
    pub table_name: Option<&'a str>,
}

impl DbClient {
    pub async fn connect(options: PgConnectOptions) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("Failed to connect to PostgreSQL")?;
        Ok(Self { pool })
    }

    pub async fn init_schema(&self) -> Result<()> {
        //noinspection SqlNoDataSourceInspection
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS analyses (
                id SERIAL PRIMARY KEY,
                file_path TEXT NOT NULL,
                file_size BIGINT NOT NULL,
                health_score FLOAT NOT NULL,
                created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create 'analyses' table")?;

        //noinspection SqlNoDataSourceInspection
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS column_summaries (
                id SERIAL PRIMARY KEY,
                analysis_id INTEGER REFERENCES analyses(id) ON DELETE CASCADE,
                column_name TEXT NOT NULL,
                kind TEXT NOT NULL,
                row_count BIGINT NOT NULL,
                null_count BIGINT NOT NULL,
                interpretation TEXT,
                business_summary TEXT,
                ml_advice TEXT,
                stats JSONB
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create 'column_summaries' table")?;

        // Ensure missing columns exist in case the table was created with an older version
        //noinspection SqlNoDataSourceInspection
        sqlx::query("ALTER TABLE column_summaries ADD COLUMN IF NOT EXISTS interpretation TEXT")
            .execute(&self.pool)
            .await
            .context("Failed to add 'interpretation' column to 'column_summaries'")?;
        //noinspection SqlNoDataSourceInspection
        sqlx::query("ALTER TABLE column_summaries ADD COLUMN IF NOT EXISTS business_summary TEXT")
            .execute(&self.pool)
            .await
            .context("Failed to add 'business_summary' column to 'column_summaries'")?;
        //noinspection SqlNoDataSourceInspection
        sqlx::query("ALTER TABLE column_summaries ADD COLUMN IF NOT EXISTS ml_advice TEXT")
            .execute(&self.pool)
            .await
            .context("Failed to add 'ml_advice' column to 'column_summaries'")?;

        Ok(())
    }

    pub async fn push_analysis(&self, params: AnalysisPush<'_>) -> Result<()> {
        // 1. Insert analysis metadata
        //noinspection SqlNoDataSourceInspection
        let analysis_id: i32 = sqlx::query_scalar(
            "INSERT INTO analyses (file_path, file_size, health_score) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(params.file_path)
        .bind(params.file_size as i64)
        .bind(params.health.score as f64)
        .fetch_one(&self.pool)
        .await
        .context("Failed to insert analysis metadata")?;

        // 2. Insert column summaries
        for col in params.summaries {
            //noinspection SqlNoDataSourceInspection
            sqlx::query(
                r#"
                INSERT INTO column_summaries
                (analysis_id, column_name, kind, row_count, null_count, interpretation, business_summary, ml_advice, stats)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#
            )
            .bind(analysis_id)
            .bind(&col.name)
            .bind(col.kind.as_str())
            .bind(col.count as i64)
            .bind(col.nulls as i64)
            .bind(col.interpretation.join(" "))
            .bind(col.business_summary.join(" "))
            .bind(col.ml_advice.join(" "))
            .bind(serde_json::to_value(&col.stats)?)
            .execute(&self.pool)
            .await
            .context(format!("Failed to insert summary for column '{}'", col.name))?;
        }

        // 3. Push the data to a dedicated table
        self.push_dataframe(
            analysis_id,
            params.df,
            params.schema_name,
            params.table_name,
        )
        .await
        .context("Failed to push data to dedicated table")?;

        Ok(())
    }

    #[expect(clippy::too_many_lines)]
    pub async fn push_dataframe(
        &self,
        analysis_id: i32,
        df: &DataFrame,
        schema_name: Option<&str>,
        table_name: Option<&str>,
    ) -> Result<()> {
        let final_table_name =
            table_name.map_or_else(|| format!("data_{analysis_id}"), ToOwned::to_owned);

        let quote = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));

        let full_identifier = match schema_name {
            Some(s) if !s.is_empty() => format!("{}.{}", quote(s), quote(&final_table_name)),
            _ => quote(&final_table_name),
        };

        let schema = df.schema();
        let mut create_table_query = format!("CREATE TABLE IF NOT EXISTS {full_identifier} (");
        let mut column_definitions = Vec::new();
        for (name, dtype) in schema.iter() {
            let sql_type = match dtype {
                DataType::Int8
                | DataType::Int16
                | DataType::Int32
                | DataType::Int64
                | DataType::UInt8
                | DataType::UInt16
                | DataType::UInt32
                | DataType::UInt64 => "BIGINT",
                DataType::Float32 | DataType::Float64 => "DOUBLE PRECISION",
                DataType::Boolean => "BOOLEAN",
                DataType::Date => "DATE",
                DataType::Datetime(_, _) => "TIMESTAMPTZ",
                _ => "TEXT",
            };
            column_definitions.push(format!("{} {sql_type}", quote(name)));
        }
        create_table_query.push_str(&column_definitions.join(", "));
        create_table_query.push(')');

        sqlx::query(&create_table_query)
            .execute(&self.pool)
            .await
            .context(format!("Failed to create data table '{full_identifier}'"))?;

        // Fast data transfer using PostgreSQL COPY
        let mut conn = self.pool.acquire().await?;

        let mut buf = Vec::new();
        CsvWriter::new(&mut buf)
            .include_header(false)
            .with_separator(b',')
            .with_null_value("".to_owned())
            .finish(&mut df.clone())
            .context("Failed to serialize dataframe to CSV for COPY")?;

        let mut writer = conn
            .copy_in_raw(&format!(
                "COPY {full_identifier} FROM STDIN WITH (FORMAT csv, NULL '')"
            ))
            .await
            .context("Failed to initiate COPY command")?;

        writer
            .send(buf)
            .await
            .context("Failed to send data via COPY")?;
        writer
            .finish()
            .await
            .context("Failed to finish COPY command")?;

        Ok(())
    }
}
