use crate::MinimalRefRecord;
use anyhow::Result;

/// Trait implemented for a type that processes records in parallel
pub trait ParallelProcessor: Send + Clone {
    /// Called on an individual record with its global index
    fn process_record<'a, Rf: MinimalRefRecord<'a>>(&mut self, record: Rf, index: usize) -> Result<()>;

    /// Called when a batch of records is complete
    fn on_batch_complete(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the processing for a thread is complete
    fn on_thread_complete(&mut self) -> Result<()> {
        Ok(())
    }

    /// Sets the thread id for the processor
    #[allow(unused_variables)]
    fn set_thread_id(&mut self, thread_id: usize) {
        // Default implementation does nothing
    }

    /// Gets the thread id for the processor
    fn get_thread_id(&self) -> usize {
        unimplemented!("Must be implemented by the processor to be used")
    }
}

/// Trait implemented for a type that processes pairs of records in parallel
pub trait PairedParallelProcessor: Send + Clone {
    /// Called on a pair of records with their indices
    fn process_record_pair<'a, Rf: MinimalRefRecord<'a>>(
        &mut self,
        record1: Rf,
        record2: Rf,
        index1: usize,
        index2: usize,
    ) -> Result<(Rf, Rf)>;

    /// Called when a batch of pairs is complete
    fn on_batch_complete(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the processing for a thread is complete
    fn on_thread_complete(&mut self) -> Result<()> {
        Ok(())
    }

    /// Sets the thread id for the processor
    #[allow(unused_variables)]
    fn set_thread_id(&mut self, thread_id: usize) {
        // Default implementation does nothing
    }

    /// Gets the thread id for the processor
    fn get_thread_id(&self) -> usize {
        unimplemented!("Must be implemented by the processor to be used")
    }
}
