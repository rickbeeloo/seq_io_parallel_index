// use seq_io::{fasta, fastq};
// use std::io;
// use thiserror::Error;

// #[derive(Error, Debug)]
// pub enum ParallelError<E> {
//     #[error("Parallel processing error: {0}")]
//     Processing(E),

//     #[error("IO error: {0}")]
//     Io(io::Error),

//     #[error("Fastq error: {0}")]
//     Fastq(fastq::Error),

//     #[error("Fasta error: {0}")]
//     Fasta(fasta::Error),
// }

// impl<E> From<io::Error> for ParallelError<E> {
//     fn from(err: io::Error) -> Self {
//         ParallelError::Io(err)
//     }
// }

// impl<E> From<fastq::Error> for ParallelError<E> {
//     fn from(err: fastq::Error) -> Self {
//         ParallelError::Fastq(err)
//     }
// }

// impl<E> From<fasta::Error> for ParallelError<E> {
//     fn from(err: fasta::Error) -> Self {
//         ParallelError::Fasta(err)
//     }
// }

// impl From<anyhow::Error> for ParallelError<anyhow::Error> {
//     fn from(err: anyhow::Error) -> Self {
//         ParallelError::Processing(err)
//     }
// }
