use anyhow::Result;
use seq_io::policy;
use std::io;

use crate::{PairedParallelProcessor, ParallelProcessor};

pub trait ParallelReader<R, P>
where
    R: io::Read + Send,
    P: policy::BufPolicy + Send,
{
    fn process_parallel<T>(self, processor: T, num_threads: usize) -> Result<()>
    where
        T: ParallelProcessor;
}

/// Trait for parallel processing of paired reads
pub trait PairedParallelReader<R, P>: ParallelReader<R, P>
where
    R: io::Read + Send,
    P: policy::BufPolicy + Send,
{
    /// Process paired FASTQ/FASTA files in parallel
    fn process_parallel_paired<T>(
        self,
        reader2: Self,
        processor: T,
        num_threads: usize,
    ) -> Result<()>
    where
        T: PairedParallelProcessor;
}
