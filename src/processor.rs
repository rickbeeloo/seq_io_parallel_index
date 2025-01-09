use crate::{MinimalRefRecord, ParallelError};

/// Trait implemented for a type that processes records in parallel
pub trait ParallelProcessor<E>: Send + Clone {
    /// Called on an individual record
    fn process_record<'a, Rf: MinimalRefRecord<'a>>(
        &mut self,
        record: Rf,
    ) -> Result<(), ParallelError<E>>;

    /// Called when a batch of records is complete
    fn on_batch_complete(&mut self) -> Result<(), ParallelError<E>> {
        Ok(())
    }
}
