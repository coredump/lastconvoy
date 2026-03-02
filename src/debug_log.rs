pub struct DebugLog {
    #[cfg(not(target_arch = "wasm32"))]
    writer: std::io::BufWriter<Box<dyn std::io::Write>>,
}

impl DebugLog {
    pub fn new(path: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::fs::OpenOptions;
            use std::io::BufWriter;
            let writer: BufWriter<Box<dyn std::io::Write>> = if path.is_empty() {
                BufWriter::new(Box::new(std::io::stderr()))
            } else {
                match OpenOptions::new().create(true).append(true).open(path) {
                    Ok(f) => BufWriter::new(Box::new(f)),
                    Err(e) => {
                        eprintln!("[debug_log] Failed to open '{}': {}", path, e);
                        BufWriter::new(Box::new(std::io::stderr()))
                    }
                }
            };
            Self { writer }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = path;
            Self {}
        }
    }

    /// Write a timestamped log line. `run_time` is seconds elapsed in the current run.
    pub fn log(&mut self, run_time: f32, msg: &str) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::io::Write;
            let _ = writeln!(self.writer, "[{:.1}s] {}", run_time, msg);
            let _ = self.writer.flush();
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (run_time, msg);
        }
    }
}
