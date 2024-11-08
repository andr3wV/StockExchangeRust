use colored::Colorize;
use std::{
    fs::{File, OpenOptions},
    io::{prelude::*, ErrorKind},
    process,
};

#[allow(dead_code)]
#[derive(PartialEq, PartialOrd)]
enum LogLevel {
    INFO = 0,
    WARN = 1,
    ERROR = 2,
}

pub struct Log {
    log_level: LogLevel,
    output_file: Option<String>,
}

#[derive(Debug)]
pub enum FileSaveError {
    FailedToOpenFile,
    FailedToWriteFile,
    FailedToCreateFile,
}

impl Log {
    fn new() -> Self {
        Self {
            log_level: LogLevel::WARN,
            output_file: None,
        }
    }
    fn fmt(log_type: LogLevel, spacing: &str, message: &str) -> String {
        match log_type {
            LogLevel::INFO => format!("[{}]{} {}", "Info".bold().green(), spacing, message),
            LogLevel::WARN => format!("[{}]{} {}", "Warn".bold().yellow(), spacing, message),
            LogLevel::ERROR => format!("[{}]{} {}", "Error".bold().red(), spacing, message),
        }
    }

    pub fn info(message: &str) -> Result<(), FileSaveError> {
        let this = Self::new();
        if this.output_file.is_some() {
            return this.info_file(message);
        }
        this.info_stdout(message);
        return Ok(());
    }
    pub fn warn(message: &str) -> Result<(), FileSaveError> {
        let this = Self::new();
        if this.output_file.is_some() {
            return this.warn_file(message);
        }
        this.warn_stdout(message);
        return Ok(());
    }
    pub fn critical(message: &str) {
        let this = Self::new();
        if this.output_file.is_some() {
            this.critical_file(message);
        }
        this.critical_stdout(message);
    }
    pub fn critical_debug(message: &str, file: &str, line: u32) {
        let this = Self::new();
        if this.output_file.is_some() {
            this.critical_debug_file(file, line, message);
        }
        this.critical_debug_stdout(file, line, message);
    }

    pub fn info_stdout(&self, message: &str) {
        if self.log_level > LogLevel::INFO {
            return;
        }
        println!("{}", Log::fmt(LogLevel::INFO, "", message));
    }
    pub fn info_file(&self, message: &str) -> Result<(), FileSaveError> {
        if self.log_level > LogLevel::INFO {
            return Ok(());
        }
        self.to_file(&Log::fmt(LogLevel::INFO, "", message))
    }

    pub fn warn_stdout(&self, message: &str) {
        if self.log_level > LogLevel::WARN {
            return;
        }
        println!("{}", Log::fmt(LogLevel::WARN, "", message));
    }
    pub fn warn_file(&self, message: &str) -> Result<(), FileSaveError> {
        if self.log_level > LogLevel::WARN {
            return Ok(());
        }
        self.to_file(&Log::fmt(LogLevel::WARN, "", message))
    }

    pub fn critical_stdout(&self, message: &str) -> ! {
        println!("{}", Log::fmt(LogLevel::ERROR, "", message));
        process::exit(1);
    }
    pub fn critical_file(&self, message: &str) -> ! {
        if let Err(e) = self.to_file(&Log::fmt(LogLevel::ERROR, "", message)) {
            println!("{:?}", e);
        }
        process::exit(1);
    }

    pub fn critical_debug_stdout(&self, file: &str, line: u32, message: &str) -> ! {
        println!(
            "{}",
            Log::fmt(LogLevel::ERROR, &format!("[{}:{}]", file, line), message,)
        );
        process::exit(1);
    }
    pub fn critical_debug_file(&self, file: &str, line: u32, message: &str) -> ! {
        if let Err(e) = self.to_file(&Log::fmt(
            LogLevel::ERROR,
            &format!("[{}:{}]", file, line),
            message,
        )) {
            println!("{:?}", e);
        }
        process::exit(1);
    }

    fn to_file(&self, data: &String) -> Result<(), FileSaveError> {
        let output_file = self.output_file.clone().unwrap();
        match OpenOptions::new()
            .write(true)
            .append(true)
            .open(&output_file)
        {
            Ok(mut file) => {
                if let Err(_) = writeln!(file, "{}", data) {
                    return Err(FileSaveError::FailedToOpenFile);
                }
                Ok(())
            }
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    return Err(FileSaveError::FailedToOpenFile);
                }
                match File::create(&output_file) {
                    Ok(mut f) => {
                        if let Err(_) = f.write(data.as_bytes()) {
                            return Err(FileSaveError::FailedToWriteFile);
                        };
                        Ok(())
                    }
                    Err(_) => Err(FileSaveError::FailedToCreateFile),
                }
            }
        }
    }
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! log {
    (info $($e: expr),*) => {
        if let Err(e) = Log::info(&format!($($e),*)) {
            println!("{:?}", e);
        }
    };
    (info object $obj: expr) => {
        if let Err(e) = Log::info(&format!("{:?}", $obj)) {
            println!("{:?}", e);
        }
    };
    (warn $($e: expr),*) => {
        if let Err(e) = Log::warn(&format!($($e),*)) {
            println!("{:?}", e);
        }
    };
    (warn object $obj: expr) => {
        if let Err(e) = Log::warn(&format!("{:?}", $obj)) {
            println!("{:?}", e);
        }
    };
    (err $($e: expr),*) => {
        Log::critical_debug(&format!($($e),*), file!(), line!());
    };
    (err object $obj: expr) => {
        Log::critical_debug(&format!("{:?}", $obj));
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! log {
    (info $($e: expr),*) => {
        if let Err(e) = Log::info(&format!($($e),*)) {
            println!("{:?}", e);
        }
    };
    (info object $obj: expr) => {
        if let Err(e) = Log::info(&format!("{:?}", $obj)) {
            println!("{:?}", e);
        }
    };
    (warn $($e: expr),*) => {
        if let Err(e) = Log::warn(&format!($($e),*)) {
            println!("{:?}", e);
        }
    };
    (warn object $obj: expr) => {
        if let Err(e) = Log::warn(&format!("{:?}", $obj)) {
            println!("{:?}", e);
        }
    };
    (err $($e: expr),*) => {
        Log::critical(&format!($($e),*));
    };
    (err object $obj: expr) => {
        Log::critical(&format!("{:?}", $obj));
    };
}
