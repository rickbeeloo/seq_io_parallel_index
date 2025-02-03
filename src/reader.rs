use anyhow::Result;
use seq_io::policy;
use std::io;

use crate::ParallelProcessor;

pub trait ParallelReader<R, P>
where
    R: io::Read + Send,
    P: policy::BufPolicy + Send,
{
    fn process_parallel<T>(self, processor: T, num_threads: usize) -> Result<()>
    where
        T: ParallelProcessor;
}
