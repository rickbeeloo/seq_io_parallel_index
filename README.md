# seq_io_parallel

A parallel processing extension for the [`seq_io`](https://github.com/markschl/seq_io) crate, providing an ergonomic API for parallel FASTA/FASTQ file processing.

## Overview

While `seq_io` includes parallel implementations for both FASTQ and FASTA readers, this library offers an alternative approach with a potentially more ergonomic API that is not reliant on closures.
The implementation follows a Map-Reduce style of parallelism that emphasizes clarity and ease of use.

## Key Features

- Single-producer multi-consumer parallel processing pipeline
- Support for both single-file and paired-end read processing
- Map-Reduce style processing architecture
- Support for both FASTA and FASTQ formats
- Thread-safe stateful processing
- Efficient memory management with reusable record sets

## Architecture

The library implements a parallel processing pipeline with the following components:

1. **Reader Thread**: A dedicated thread that continuously fills a limited set of `RecordSets` until EOF
2. **Worker Threads**: Multiple threads that process ready `RecordSets` in parallel
3. **Record Processing**: While `RecordSets` may be processed out of order, records within each set maintain their sequence

## Implementation

### The ParallelProcessor Traits

To use parallel processing, implement one of the following traits:

```rust
// For single-file processing
pub trait ParallelProcessor: Send + Clone {
    // Map: Process individual records
    fn process_record<'a, Rf: MinimalRefRecord<'a>>(&mut self, record: Rf) -> Result<()>;

    // Reduce: Process completed batches (optional)
    fn on_batch_complete(&mut self) -> Result<()> {
        Ok(())
    }
}

// For paired-end read processing
pub trait PairedParallelProcessor: Send + Clone {
    // Map: Process pairs of records
    fn process_record_pair<'a, Rf: MinimalRefRecord<'a>>(
        &mut self,
        record1: Rf,
        record2: Rf,
    ) -> Result<()>;

    // Reduce: Process completed batches (optional)
    fn on_batch_complete(&mut self) -> Result<()> {
        Ok(())
    }
}
```

### Record Access

Both FASTA and FASTQ records are accessed through the `MinimalRefRecord` trait:

```rust
pub trait MinimalRefRecord<'a> {
    fn ref_head(&self) -> &[u8];  // Header data
    fn ref_seq(&self) -> &[u8];   // Sequence data
    fn ref_qual(&self) -> &[u8];  // Quality scores (empty for FASTA)
}
```

## Usage Examples

### Single-File Processing

Here's a simple example that performs parallel processing of a FASTQ file:

```rust
use anyhow::Result;
use seq_io::fastq;
use seq_io_parallel::{MinimalRefRecord, ParallelProcessor, ParallelReader};
use std::sync::{atomic::AtomicUsize, Arc};

#[derive(Clone, Default)]
pub struct ExpensiveCalculation {
    local_sum: usize,
    global_sum: Arc<AtomicUsize>,
}

impl ParallelProcessor for ExpensiveCalculation {
    fn process_record<'a, Rf: MinimalRefRecord<'a>>(&mut self, record: Rf) -> Result<()> {
        let seq = record.ref_seq();
        let qual = record.ref_qual();

        // Simulate expensive calculation
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

fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("No path provided");
    let num_threads = std::env::args()
        .nth(2)
        .map(|n| n.parse().unwrap())
        .unwrap_or(1);

    let (handle, _) = niffler::send::from_path(&path)?;
    let reader = fastq::Reader::new(handle);
    let processor = ExpensiveCalculation::default();

    reader.process_parallel(processor.clone(), num_threads)?;
    Ok(())
}
```

### Paired-End Read Processing

For processing paired-end reads (e.g., R1 and R2 FASTQ files):

```rust
use seq_io_parallel::{MinimalRefRecord, PairedParallelProcessor, PairedParallelReader};

impl PairedParallelProcessor for ExpensiveCalculation {
    fn process_record_pair<'a, Rf: MinimalRefRecord<'a>>(&mut self, r1: Rf, r2: Rf) -> Result<()> {
        // Validate that paired records belong together (optional, not efficient but useful for sanity checks).
        if r1.ref_head() != r2.ref_head() {
            bail!("Headers do not match");
        }

        // Process both reads
        for _ in 0..50 {
            for (s, q) in r1.ref_seq().iter().zip(r1.ref_qual().iter()) {
                self.local_sum += (*s - 33) as usize + (*q - 33) as usize;
            }
            for (s, q) in r2.ref_seq().iter().zip(r2.ref_qual().iter()) {
                self.local_sum += (*s - 33) as usize + (*q - 33) as usize;
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let (handle_r1, _) = niffler::send::from_path(path_r1)?;
    let (handle_r2, _) = niffler::send::from_path(path_r2)?;
    let reader_r1 = fastq::Reader::new(handle_r1);
    let reader_r2 = fastq::Reader::new(handle_r2);
    let processor = ExpensiveCalculation::default();

    reader_r1.process_parallel_paired(reader_r2, processor.clone(), num_threads)?;
    Ok(())
}
```

## Performance Considerations

FASTA/FASTQ processing is typically I/O-bound, so parallel processing benefits may vary:

- Best for computationally expensive operations (e.g., alignment, k-mer counting)
- Performance gains depend on the ratio of I/O to processing time
- Consider using `Arc` for processor state with heavy initialization costs
- Paired-end processing maintains synchronization between R1 and R2 files.

## Implementation Notes

- Each worker thread receives a `Clone` of the `ParallelProcessor`
- Thread-local state can be maintained without locks
- Global state should use appropriate synchronization (e.g., `Arc<AtomicUsize>`)
- Heavy initialization costs can be mitigated by wrapping in `Arc`

## Future Work

Currently this library is making use of `anyhow` for all error handling.
This is not ideal for custom error types in libraries, but for many CLI tools will work just fine.
In the future this may change.
