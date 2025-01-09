pub mod error;
pub mod fastq;
pub mod processor;
pub mod reader;
pub mod record;

pub use error::ParallelError;
pub use processor::ParallelProcessor;
pub use reader::ParallelReader;
pub use record::MinimalRefRecord;
