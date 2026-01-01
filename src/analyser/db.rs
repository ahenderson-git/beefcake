use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, QueryBuilder};
use polars::prelude::*;
use crate::analyser::logic::{ColumnSummary, FileHealth};

pub struct DbClient {
    pool: Pool<Postgres>,
}

impl DbClient {
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn init_schema(&self) -> Result<()> {
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
        .await?;

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
                stats JSONB
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn push_analysis(
        &self,
        file_path: &str,
        file_size: u64,
        health: &FileHealth,
        summaries: &[ColumnSummary],
        df: &DataFrame,
        schema_name: Option<&str>,
        table_name: Option<&str>,
    ) -> Result<()> {
        // 1. Insert analysis metadata
        let analysis_id: i32 = sqlx::query_scalar(
            "INSERT INTO analyses (file_path, file_size, health_score) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(file_path)
        .bind(file_size as i64)
        .bind(health.score as f64)
        .fetch_one(&self.pool)
        .await?;

        // 2. Insert column summaries
        for col in summaries {
            sqlx::query(
                r#"
                INSERT INTO column_summaries 
                (analysis_id, column_name, kind, row_count, null_count, interpretation, business_summary, stats)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#
            )
            .bind(analysis_id)
            .bind(&col.name)
            .bind(col.kind.as_str())
            .bind(col.count as i64)
            .bind(col.nulls as i64)
            .bind(&col.interpretation)
            .bind(&col.business_summary)
            .bind(serde_json::to_value(&col.stats)?)
            .execute(&self.pool)
            .await?;
        }

        // 3. Push the data to a dedicated table
        self.push_dataframe(analysis_id, df, schema_name, table_name).await?;

        Ok(())
    }

    async fn push_dataframe(&self, analysis_id: i32, df: &DataFrame, schema_name: Option<&str>, table_name: Option<&str>) -> Result<()> {
        let final_table_name = table_name.unwrap_or(&format!("data_{}", analysis_id)).to_string();
        let full_identifier = match schema_name {
            Some(s) if !s.is_empty() => format!("\"{}\".\"{}\"", s, final_table_name),
            _ => format!("\"{}\"", final_table_name),
        };
        
        let schema = df.schema();
        let mut create_table_query = format!("CREATE TABLE {} (", full_identifier);
        let mut column_definitions = Vec::new();
        for (name, dtype) in schema.iter() {
            let sql_type = match dtype {
                DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 |
                DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => "BIGINT",
                DataType::Float32 | DataType::Float64 => "DOUBLE PRECISION",
                DataType::String => "TEXT",
                DataType::Boolean => "BOOLEAN",
                DataType::Date => "DATE",
                DataType::Datetime(_, _) => "TIMESTAMPTZ",
                _ => "TEXT",
            };
            column_definitions.push(format!("\"{}\" {}", name, sql_type));
        }
        create_table_query.push_str(&column_definitions.join(", "));
        create_table_query.push_str(")");

        sqlx::query(&create_table_query).execute(&self.pool).await?;

        // Batch insert data using QueryBuilder
        let batch_size = 1000;
        let column_names: Vec<String> = schema.iter().map(|(n, _)| n.to_string()).collect();
        
        for chunk_start in (0..df.height()).step_by(batch_size) {
            let chunk_end = (chunk_start + batch_size).min(df.height());
            let mut query_builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(format!("INSERT INTO {} (", full_identifier));
            
            for (i, name) in column_names.iter().enumerate() {
                query_builder.push(format!("\"{}\"", name));
                if i < column_names.len() - 1 {
                    query_builder.push(", ");
                }
            }
            query_builder.push(") ");

            query_builder.push_values(chunk_start..chunk_end, |mut b, row_idx| {
                for col_idx in 0..df.width() {
                    let series = &df.get_columns()[col_idx];
                    let val = series.get(row_idx).unwrap_or(AnyValue::Null);
                    
                    match val {
                        AnyValue::Int8(v) => { b.push_bind(v as i64); },
                        AnyValue::Int16(v) => { b.push_bind(v as i64); },
                        AnyValue::Int32(v) => { b.push_bind(v as i64); },
                        AnyValue::Int64(v) => { b.push_bind(v); },
                        AnyValue::UInt8(v) => { b.push_bind(v as i64); },
                        AnyValue::UInt16(v) => { b.push_bind(v as i64); },
                        AnyValue::UInt32(v) => { b.push_bind(v as i64); },
                        AnyValue::UInt64(v) => { b.push_bind(v as i64); },
                        AnyValue::Float32(v) => { b.push_bind(v as f64); },
                        AnyValue::Float64(v) => { b.push_bind(v); },
                        AnyValue::String(v) => { b.push_bind(v); },
                        AnyValue::StringOwned(v) => { b.push_bind(v.to_string()); },
                        AnyValue::Boolean(v) => { b.push_bind(v); },
                        AnyValue::Date(v) => {
                             let date = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap() + chrono::Duration::days(v as i64);
                             b.push_bind(date);
                        },
                        AnyValue::Datetime(v, tu, _) => {
                             let dt = match tu {
                                TimeUnit::Nanoseconds => chrono::DateTime::from_timestamp(v / 1_000_000_000, (v % 1_000_000_000) as u32),
                                TimeUnit::Microseconds => chrono::DateTime::from_timestamp(v / 1_000_000, ((v % 1_000_000) * 1000) as u32),
                                TimeUnit::Milliseconds => chrono::DateTime::from_timestamp(v / 1000, ((v % 1000) * 1_000_000) as u32),
                             };
                             b.push_bind(dt);
                        }
                        AnyValue::Null => { b.push_bind(None::<i64>); },
                        _ => { b.push_bind(val.to_string()); }
                    }
                }
            });

            let query = query_builder.build();
            query.execute(&self.pool).await?;
        }

        Ok(())
    }
}
