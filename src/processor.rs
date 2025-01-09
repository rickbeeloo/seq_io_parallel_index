use crate::MinimalRefRecord;
use anyhow::Result;

/// Trait implemented for a type that processes records in parallel
pub trait ParallelProcessor: Send + Clone {
    /// Called on an individual record
    fn process_record<'a, Rf: MinimalRefRecord<'a>>(&mut self, record: Rf) -> Result<()>;

    /// Called when a batch of records is complete
    fn on_batch_complete(&mut self) -> Result<()> {
        Ok(())
    }
}
