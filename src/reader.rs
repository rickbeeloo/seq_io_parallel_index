use crate::{processor::ParallelProcessor, ParallelError};

use seq_io::policy;
use std::io;

pub trait ParallelReader<R, P>
where
    R: io::Read + Send,
    P: policy::BufPolicy + Send,
{
    fn process_parallel<T, E>(
        self,
        processor: T,
        num_threads: usize,
    ) -> Result<(), ParallelError<E>>
    where
        T: ParallelProcessor<E>,
        E: Into<ParallelError<E>> + Send;
}
