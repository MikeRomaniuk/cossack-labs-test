use std::collections::VecDeque;
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use interface_types::Telemetry;

use crate::interface::LogWriter;

pub struct FileLogger {
    queue: VecDeque<Telemetry>,
    file: File,
}

impl FileLogger {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let file = File::create(path)?;

        Ok(Self {
            queue: VecDeque::new(),
            file,
        })
    }
}

impl<T: Debug> LogWriter<T> for FileLogger {
    fn log(&mut self, log: T) -> anyhow::Result<()> {
        let telemetry_string = format!("{:?}", log);
        self.file.write_all(telemetry_string.as_bytes())?;
        self.file.write_all(b"\n")?;

        Ok(())
    }
}
