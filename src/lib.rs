mod macro_impl;
pub mod processor;
pub mod reader;
pub mod record;

pub use processor::ParallelProcessor;
pub use reader::ParallelReader;
pub use record::MinimalRefRecord;

pub use seq_io::{fasta, fastq, policy};
