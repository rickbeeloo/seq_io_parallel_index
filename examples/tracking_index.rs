use anyhow::{bail, Result};
use seq_io::fastq;
use seq_io_parallel::{MinimalRefRecord, ParallelProcessor, ParallelReader};
use std::sync::{atomic::AtomicUsize, Arc, Mutex};
use std::io::BufWriter;
use std::fs::File;
use std::env::temp_dir;
use std::io::Write;
#[derive(Clone)]
pub struct ExpensiveOrderedReads {
    buf_writer: Arc<Mutex<BufWriter<File>>>,
    local_sum: usize,
}

impl Default for ExpensiveOrderedReads {
    fn default() -> Self {
        let path = temp_dir().join("default.fastq");
        Self::new(&path.to_string_lossy()).unwrap()
    }
}

impl ExpensiveOrderedReads {
    pub fn new(path: &str) -> Result<Self> {
        let file = File::create(path)?;
        let buf_writer = BufWriter::new(file);
        Ok(Self { 
            buf_writer: Arc::new(Mutex::new(buf_writer)),
            local_sum: 0,
        })
    }
}

impl ParallelProcessor for ExpensiveOrderedReads {
    
    fn process_record<'a, Rf: MinimalRefRecord<'a>>(&mut self, record: Rf, global_idx: usize) -> Result<()> {
        let seq = record.ref_seq();
        let qual = record.ref_qual();

        // Useless in this example, but to do something expensive
        for _ in 0..100 {
            for (s, q) in seq.iter().zip(qual.iter()) {
                self.local_sum += (*s - 33) as usize + (*q - 33) as usize;
            }
        }

        // This should be done in a separate threads of course, but for not mutex locked
        let mut writer = self.buf_writer.lock().unwrap();
        writeln!(writer, "{} {}", String::from_utf8_lossy(record.ref_head()), global_idx)?;
        drop(writer);

        Ok(())
    }

    fn on_batch_complete(&mut self) -> Result<()> {
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
    let processor = ExpensiveOrderedReads::default();
    reader.process_parallel(processor.clone(), num_threads)?;

    println!("Local sum: {}", processor.local_sum);

    Ok(())
}
