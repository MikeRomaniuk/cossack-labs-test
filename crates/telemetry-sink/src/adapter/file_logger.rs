use core::fmt::Debug;
use std::fs::File;
use std::io::Write as _;
use std::path::Path;

use crate::interface::LogWriter;

pub(crate) struct FileLogger {
    file: File,
}

impl FileLogger {
    pub(crate) fn new(path: &Path) -> anyhow::Result<Self> {
        let file = File::create(path)?;

        Ok(Self { file })
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
