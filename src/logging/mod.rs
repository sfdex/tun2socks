use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::str::FromStr;
use std::time::{Duration, Instant, SystemTime};

// #[derive(Copy, Clone)]
pub struct Logging {
    file: File,
    instant: Instant,
}

impl Logging {
    pub fn new(path: &str) -> Self {
        let file = OpenOptions::new().create(true).append(true).open(path).unwrap();
        Logging { file, instant: Instant::now() }
    }

    fn writeln(&mut self, level: &str, content: String) {
        let elapsed = self.elapsed();
        writeln!(&mut self.file, "{:?} {level}: {}", elapsed, content).unwrap();
        // println!("{:?} {level}: {}", elapsed, content);
    }

    pub fn v(&mut self, content: String) {
        self.writeln("V", content);
    }

    pub fn d(&mut self, content: String) {
        self.writeln("D", content);
    }

    pub fn i(&mut self, content: String) {
        self.writeln("I", content);
    }

    pub fn w(&mut self, content: String) {
        self.writeln("W", content);
    }

    pub fn e(&mut self, content: String) {
        self.writeln("E", content);
    }

    fn now() -> SystemTime {
        SystemTime::now()
    }

    fn elapsed(&self) -> Duration {
        self.instant.elapsed()
    }
}

// impl Copy for Logging {}

impl Clone for Logging {
    fn clone(&self) -> Self {
        let f = self.file.try_clone().unwrap();
        Self {
            file: f,
            instant: self.instant.clone(),
        }
    }
}

