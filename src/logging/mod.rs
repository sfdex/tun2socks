use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::str::FromStr;
use std::time::SystemTime;

// #[derive(Copy, Clone)]
pub struct Logging {
    file: File,
}

impl Logging {
    pub fn new(path: &str) -> Self {
        let file = OpenOptions::new().create(true).append(true).open(path).unwrap();
        Logging { file }
    }

    pub fn v(&mut self, content: String) {
        writeln!(&mut self.file, "{:?} E: {}", Self::now(), content).unwrap();
    }

    pub fn d(&mut self, content: String) {
        writeln!(&mut self.file, "{:?} E: {}", Self::now(), content).unwrap();
    }

    pub fn i(&mut self, content: String) {
        writeln!(&mut self.file, "{:?} E: {}", Self::now(), content).unwrap();
    }

    pub fn w(&mut self, content: String) {
        writeln!(&mut self.file, "{:?} W: {}", Self::now(), content).unwrap();
    }

    pub fn e(&mut self, content: String) {
        writeln!(&mut self.file, "{:?} E: {}", Self::now(), content).unwrap();
    }

    fn now() -> SystemTime {
        SystemTime::now()
    }
}

// impl Copy for Logging {}

impl Clone for Logging {
    fn clone(&self) -> Self {
        let f = self.file.try_clone().unwrap();
        Self {
            file: f
        }
    }
}

