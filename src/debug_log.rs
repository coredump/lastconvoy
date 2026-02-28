use std::fs::File;
use std::io::{BufWriter, Write};

pub struct DebugLog {
    writer: BufWriter<Box<dyn Write>>,
}

impl DebugLog {
    pub fn new(path: &str) -> Self {
        let writer: BufWriter<Box<dyn Write>> = if path.is_empty() {
            BufWriter::new(Box::new(std::io::stderr()))
        } else {
            match File::create(path) {
                Ok(f) => BufWriter::new(Box::new(f)),
                Err(e) => {
                    eprintln!("[debug_log] Failed to open '{}': {}", path, e);
                    BufWriter::new(Box::new(std::io::stderr()))
                }
            }
        };
        Self { writer }
    }

    /// Write a timestamped log line. `run_time` is seconds elapsed in the current run.
    pub fn log(&mut self, run_time: f32, msg: &str) {
        let _ = writeln!(self.writer, "[{:.1}s] {}", run_time, msg);
        let _ = self.writer.flush();
    }
}
