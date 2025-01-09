# seq_io_parallel

This a library that extends the excellent [`seq_io`](https://github.com/markschl/seq_io) crate with an alternative parallel implementation.

Note that `seq_io` has a parallel implementation for both `FASTQ` and `FASTA` readers, but I found the API to be somewhat cumbersome. This library aims to provide a more ergonomic API for parallel file reading.

This crate aims to provide a different approach that follows a Map-Reduce style of parallelism that isn't closure based.

## Design

This library is implemented as a single-producer multi-consumer parallel processing pipeline.

A reader thread is spawned that fills a limited reusable set of `RecordSets` continuously until the end of the file is reached.
A set of worker threads are spawned that listen for the indices of the `RecordSets` that are ready to be processed.

Note that the `RecordSets` are not processed in order, but the records within each `RecordSet` are processed in order.

Parallel processing implementors must implement the `ParallelProcessor` trait, which has two available functions:

1. `process_record` - This function is called on each record in the `RecordSet` in parallel within a worker thread within a batch.
2. `on_batch_complete` - This function is called when all records in a `RecordSet` have been processed.

The `process_record` is the map, and the `on_batch_complete` is the reduce.

Each thread will have a `Clone` of the `ParallelProcessor` that is passed to the `RecordSet` for processing which allows for a stateful processing of records in parallel.
If you have a heavy initialization cost for your processor I recommend wrapping those heavy internals in `Arc` to avoid unnecessary cloning to child threads.

Note that the `ParallelProcessor` must be `Send` and `Clone`.

```rust
/// Trait implemented for a type that processes records in parallel
pub trait ParallelProcessor: Send + Clone {
    /// Called on an individual record
    fn process_record<'a, Rf: MinimalRefRecord<'a>>(&mut self, record: Rf) -> Result<()>;

    /// Called when a batch of records is complete
    fn on_batch_complete(&mut self) -> Result<()> {
        Ok(())
    }
}
```

Note that the `MinimalRefRecord` trait is passed generically to the `process_record` function to allow for both `FASTA` and `FASTQ` records to be processed in parallel.

This is a bit of a hack to allow for the parallel processing of both `FASTA` and `FASTQ` records in the same pipeline.

The `MinimalRefRecord` trait is implemented for both `seq_io::fasta::RefRecord` and `seq_io::fastq::RefRecord` and provides a common interface for accessing the sequence and quality data of a record.
The quality data of a `FASTA` record will return an empty slice so it is up to the implementor to handle this case.

The way to access the header, sequence, and quality is to use the `MinimalRefRecord` trait functions which do not overlap the original `seq_io::{fasta, fastq}::Record` functions:

```rust
pub trait MinimalRefRecord<'a> {
    fn ref_head(&self) -> &[u8];

    fn ref_seq(&self) -> &[u8];

    fn ref_qual(&self) -> &[u8];
}
```

## Example

Let's see an example of a simple parallel processing of a `FASTQ` file.

Note that the Processor maintains two variables - a local sum and a global sum.
The local sum is updated for each record processed and **does not need to be locked**.
The global sum is updated atomically and is the sum of all local sums - but is only locked once per batch, minimizing contention.

```rust
use anyhow::{bail, Result};
use seq_io::fastq;
use seq_io_parallel::{MinimalRefRecord, ParallelProcessor, ParallelReader};
use std::sync::{atomic::AtomicUsize, Arc};

#[derive(Clone, Default)]
pub struct ExpensiveCalculation {
    local_sum: usize,
    global_sum: Arc<AtomicUsize>,
}
impl ExpensiveCalculation {
    pub fn get_global_sum(&self) -> usize {
        self.global_sum.load(std::sync::atomic::Ordering::Relaxed)
    }
}
impl ParallelProcessor for ExpensiveCalculation {
    fn process_record<'a, Rf: MinimalRefRecord<'a>>(&mut self, record: Rf) -> Result<()> {
        let seq = record.ref_seq();
        let qual = record.ref_qual();

        for _ in 0..100 {
            for (s, q) in seq.iter().zip(qual.iter()) {
                self.local_sum += (*s - 33) as usize + (*q - 33) as usize;
            }
        }

        Ok(())
    }

    fn on_batch_complete(&mut self) -> Result<()> {
        self.global_sum
            .fetch_add(self.local_sum, std::sync::atomic::Ordering::Relaxed);
        self.local_sum = 0;
        Ok(())
    }
}

pub fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<String>>();
    let path = match args.get(1) {
        Some(path) => path,
        None => bail!("No path provided"),
    };
    let num_threads = match args.get(2) {
        Some(num_threads) => num_threads.parse::<usize>()?,
        None => 1,
    };

    let (handle, _format) = niffler::send::from_path(path)?;
    let reader = fastq::Reader::new(handle);
    let processor = ExpensiveCalculation::default();
    reader.process_parallel(processor.clone(), num_threads)?;

    println!("Global sum: {}", processor.get_global_sum());

    Ok(())
}

```

## Performance

FAST[AQ] processing is inherently IO-bound so you may not see a significant speedup with parallel processing unless your processing task is very expensive.
The above example simulates this with a nonsensical calculation that is repeated 100 times, but in practice, you would replace this with your own processing logic such as alignment, or k-mer count or whatever.

Your mileage may vary.
