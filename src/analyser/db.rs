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
            .idle_timeout(Some(std::time::Duration::from_secs(300))) // Close idle connections after 5 minutes
            .max_lifetime(Some(std::time::Duration::from_secs(1800))) // Recycle connections after 30 minutes
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
        let schema = df.schema();
        self.prepare_table(&schema, analysis_id, schema_name, table_name)
            .await?;

        // Fast data transfer using PostgreSQL COPY in chunks to avoid memory explosion
        let mut conn = self.pool.acquire().await?;
        let full_identifier = Self::get_full_identifier(analysis_id, schema_name, table_name);

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

    pub async fn push_from_csv_file(
        &self,
        path: &std::path::Path,
        schema: &Schema,
        schema_name: Option<&str>,
        table_name: Option<&str>,
    ) -> Result<()> {
        self.prepare_table(schema, 0, schema_name, table_name)
            .await?;

        let mut conn = self.pool.acquire().await?;
        let full_identifier = Self::get_full_identifier(0, schema_name, table_name);

        let mut writer = conn
            .copy_in_raw(&format!(
                "COPY {full_identifier} FROM STDIN WITH (FORMAT csv, HEADER true, NULL '')"
            ))
            .await
            .context("Failed to initiate COPY command")?;

        use std::io::Read as _;
        let mut file = std::fs::File::open(path).context("Failed to open CSV file for DB push")?;
        let mut buf = vec![0u8; 1024 * 1024]; // 1MB buffer

        loop {
            let n = file
                .read(&mut buf)
                .context("Failed to read from CSV file")?;
            if n == 0 {
                break;
            }
            let chunk = buf
                .get(..n)
                .ok_or_else(|| anyhow::anyhow!("Buffer slice error"))?
                .to_vec();
            writer
                .send(chunk)
                .await
                .context("Failed to send CSV chunk to DB")?;
        }

        writer
            .finish()
            .await
            .context("Failed to finish COPY command")?;
        Ok(())
    }

    fn get_full_identifier(
        analysis_id: i32,
        schema_name: Option<&str>,
        table_name: Option<&str>,
    ) -> String {
        let final_table_name =
            table_name.map_or_else(|| format!("data_{analysis_id}"), ToOwned::to_owned);
        let quote = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
        match schema_name {
            Some(s) if !s.is_empty() => format!("{}.{}", quote(s), quote(&final_table_name)),
            _ => quote(&final_table_name),
        }
    }

    async fn prepare_table(
        &self,
        schema: &Schema,
        analysis_id: i32,
        schema_name: Option<&str>,
        table_name: Option<&str>,
    ) -> Result<()> {
        let full_identifier = Self::get_full_identifier(analysis_id, schema_name, table_name);
        let quote = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));

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
        Ok(())
    }
}
