use anthropic::messages::Usage;
use anyhow::{anyhow, Result};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub trait UsageReporter: Send + Sync {
    fn report(&self, usage: &Usage) -> Result<()>;
}

pub struct NoopUsageReporter;

impl UsageReporter for NoopUsageReporter {
    fn report(&self, _usage: &Usage) -> Result<()> {
        Ok(())
    }
}

pub struct PostgresUsageReporter {
    pool: Pool<Postgres>,
}

pub struct PostgresUsageReporterBuilder {
    address: Option<String>,
}

impl PostgresUsageReporter {
    pub fn builder() -> PostgresUsageReporterBuilder {
        PostgresUsageReporterBuilder { address: None }
    }
}

impl PostgresUsageReporterBuilder {
    pub fn with_address<S>(mut self, address: S) -> Self
    where
        S: Into<String>,
    {
        self.address = Some(address.into());
        self
    }

    pub async fn build(self) -> Result<PostgresUsageReporter> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(
                &self
                    .address
                    .ok_or_else(|| anyhow::anyhow!("Address not provided"))?,
            )
            .await?;

        Ok(PostgresUsageReporter { pool })
    }
}

impl UsageReporter for PostgresUsageReporter {
    fn report(&self, usage: &Usage) -> Result<()> {
        let input_tokens = usage
            .input_tokens
            .ok_or_else(|| anyhow!("invalid input token"))?;

        let output_tokens = usage.output_tokens;

        let pool = self.pool.clone();

        tokio::spawn(async move {
            match sqlx::query(
                r#"
                INSERT INTO anthropic_usage (input_tokens, output_tokens)
                VALUES ($1, $2)
                "#,
            )
            .bind(input_tokens as i32)
            .bind(output_tokens as i32)
            .execute(&pool)
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Failed to insert usage: {}", e);
                }
            }
        });

        Ok(())
    }
}
