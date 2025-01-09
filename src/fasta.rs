use anyhow::Result;
use crossbeam_channel;
use std::io;
use std::marker::Send;
use std::sync::{Arc, Mutex};
use std::thread;

use seq_io::{
    fasta::{Reader, RecordSet},
    policy,
};

use crate::{ParallelProcessor, ParallelReader};

impl<R, P> ParallelReader<R, P> for Reader<R, P>
where
    R: io::Read + Send,
    P: policy::BufPolicy + Send,
{
    fn process_parallel<T>(mut self, processor: T, num_threads: usize) -> Result<()>
    where
        T: ParallelProcessor,
    {
        // Create shared vector of record sets
        let record_sets: Vec<_> = (0..num_threads * 2)
            .map(|_| Mutex::new(RecordSet::default()))
            .collect();
        let record_sets = Arc::new(record_sets);

        // Create bounded MPMC channel
        let (tx, rx) = crossbeam_channel::bounded(num_threads * 2);

        thread::scope(|scope| -> Result<()> {
            // Spawn reader thread
            let record_sets_reader = Arc::clone(&record_sets);
            let reader_handle = scope.spawn(move || -> Result<()> {
                let mut current_idx = 0;

                loop {
                    // Get next record set
                    let mut record_set = record_sets_reader[current_idx].lock().unwrap();

                    if let Some(result) = self.read_record_set(&mut record_set) {
                        result?;

                        // Check if we got any records
                        if record_set.into_iter().next().is_some() {
                            drop(record_set); // Release lock before send
                            tx.send(Some(current_idx)).unwrap();
                            current_idx += 1;

                            if current_idx >= record_sets_reader.len() {
                                current_idx = 0;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                // Signal completion to all workers
                for _ in 0..num_threads {
                    tx.send(None).unwrap();
                }

                Ok(())
            });

            // Spawn worker threads
            let mut handles = Vec::new();

            for _tid in 0..num_threads {
                let record_sets = Arc::clone(&record_sets);
                let rx = rx.clone();
                let mut processor = processor.clone();

                let handle = scope.spawn(move || -> Result<()> {
                    while let Ok(Some(idx)) = rx.recv() {
                        let record_set = record_sets[idx].lock().unwrap();

                        for record in record_set.into_iter() {
                            processor.process_record(record)?;
                        }

                        processor.on_batch_complete()?;
                    }
                    Ok(())
                });

                handles.push(handle);
            }

            // Wait for reader thread
            reader_handle.join().unwrap()?;

            // Wait for worker threads
            for handle in handles {
                handle.join().unwrap()?;
            }

            Ok(())
        })?;

        Ok(())
    }
}
