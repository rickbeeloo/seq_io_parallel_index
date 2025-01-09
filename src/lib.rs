mod macro_impl;
mod macro_paired_impl;
pub mod processor;
pub mod reader;
pub mod record;

pub use processor::{PairedParallelProcessor, ParallelProcessor};
pub use reader::{PairedParallelReader, ParallelReader};
pub use record::MinimalRefRecord;
