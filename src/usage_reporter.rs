pub use anthropic::messages::Usage;
use anyhow::Result;

pub trait UsageReporter: Send + Sync {
    fn report(&self, usage: &Usage) -> Result<()>;
}

pub struct NoopUsageReporter;

impl UsageReporter for NoopUsageReporter {
    fn report(&self, _usage: &Usage) -> Result<()> {
        Ok(())
    }
}
