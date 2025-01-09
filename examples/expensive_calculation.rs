use anyhow::{bail, Result};
use seq_io::fastq;
use seq_io_parallel::{MinimalRefRecord, ParallelProcessor, ParallelReader};
use std::sync::{atomic::AtomicUsize, Arc};

#[derive(Clone, Default)]
pub struct ExpensiveCalculation {
    local_sum: usize,
    local_num_records: usize,
    global_sum: Arc<AtomicUsize>,
    global_num_records: Arc<AtomicUsize>,
}
impl ExpensiveCalculation {
    pub fn get_global_sum(&self) -> usize {
        self.global_sum.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn get_global_num_records(&self) -> usize {
        self.global_num_records
            .load(std::sync::atomic::Ordering::Relaxed)
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

        self.local_num_records += 1;

        Ok(())
    }

    fn on_batch_complete(&mut self) -> Result<()> {
        self.global_sum
            .fetch_add(self.local_sum, std::sync::atomic::Ordering::Relaxed);

        self.global_num_records
            .fetch_add(self.local_num_records, std::sync::atomic::Ordering::Relaxed);

        self.local_sum = 0;
        self.local_num_records = 0;
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
    println!("Global num records: {}", processor.get_global_num_records());

    Ok(())
}
