use anyhow::{Context as _, Result};
use polars::prelude::*;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Pool, Postgres};

pub struct DbClient {
    pool: Pool<Postgres>,
}

impl DbClient {
    pub async fn connect(options: PgConnectOptions) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect_with(options)
            .await
            .context("Failed to connect to PostgreSQL (timeout after 10s)")?;
        Ok(Self { pool })
    }

    pub async fn init_schema(&self) -> Result<()> {
        Ok(())
    }

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

        // Fast data transfer using PostgreSQL COPY in chunks to avoid memory explosion
        let mut conn = self.pool.acquire().await?;

        let mut writer = conn
            .copy_in_raw(&format!(
                "COPY {full_identifier} FROM STDIN WITH (FORMAT csv, NULL '')"
            ))
            .await
            .context("Failed to initiate COPY command")?;

        let chunk_size = 10_000;
        let height = df.height();
        
        for i in (0..height).step_by(chunk_size) {
            let len = std::cmp::min(chunk_size, height - i);
            let mut chunk = df.slice(i as i64, len);
            
            let mut buf = Vec::new();
            CsvWriter::new(&mut buf)
                .include_header(false)
                .with_separator(b',')
                .with_null_value(String::new())
                .finish(&mut chunk)
                .context("Failed to serialize dataframe chunk to CSV")?;

            writer
                .send(buf)
                .await
                .context("Failed to send data chunk via COPY")?;
        }

        writer
            .finish()
            .await
            .context("Failed to finish COPY command")?;

        Ok(())
    }
}
