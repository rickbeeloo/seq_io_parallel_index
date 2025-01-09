use anyhow::{bail, Result};
use seq_io::fastq;
use seq_io_parallel::{MinimalRefRecord, PairedParallelProcessor, PairedParallelReader};
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
    fn validate_header<'a, Rf: MinimalRefRecord<'a>>(&self, r1: &Rf, r2: &Rf) -> Result<()> {
        if r1.ref_head() != r2.ref_head() {
            bail!("Headers do not match");
        }
        Ok(())
    }
}
impl PairedParallelProcessor for ExpensiveCalculation {
    fn process_record_pair<'a, Rf: MinimalRefRecord<'a>>(&mut self, r1: Rf, r2: Rf) -> Result<()> {
        self.validate_header(&r1, &r2)?;

        for _ in 0..50 {
            for (s, q) in r1.ref_seq().iter().zip(r1.ref_qual().iter()) {
                self.local_sum += (*s - 33) as usize + (*q - 33) as usize;
            }

            for (s, q) in r2.ref_seq().iter().zip(r2.ref_qual().iter()) {
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
    let path_r1 = match args.get(1) {
        Some(path) => path,
        None => bail!("No path provided"),
    };
    let path_r2 = match args.get(2) {
        Some(path) => path,
        None => bail!("No path provided"),
    };
    let num_threads = match args.get(3) {
        Some(num_threads) => num_threads.parse::<usize>()?,
        None => 1,
    };

    let (handle_r1, _format_r1) = niffler::send::from_path(path_r1)?;
    let (handle_r2, _format_r2) = niffler::send::from_path(path_r2)?;
    let reader_r1 = fastq::Reader::new(handle_r1);
    let reader_r2 = fastq::Reader::new(handle_r2);
    let processor = ExpensiveCalculation::default();
    reader_r1.process_parallel_paired(reader_r2, processor.clone(), num_threads)?;

    println!("Global sum: {}", processor.get_global_sum());
    println!("Global num records: {}", processor.get_global_num_records());

    Ok(())
}
