use colored::Colorize;
use std::{
    fs::{File, OpenOptions},
    io::{prelude::*, ErrorKind},
    process,
};

#[allow(dead_code)]
#[derive(PartialEq, PartialOrd)]
enum LogLevel {
    Info = 0,
    Warn = 1,
    Error = 2,
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
    #[cfg(debug_assertions)]
    fn new() -> Self {
        Self {
            log_level: LogLevel::Info,
            output_file: Some("debug.log".to_string()),
        }
    }
    #[cfg(not(debug_assertions))]
    fn new() -> Self {
        Self {
            log_level: LogLevel::Warn,
            output_file: None,
        }
    }

    fn fmt(log_type: LogLevel, spacing: &str, message: &str, formatting: bool) -> String {
        let (info_str, warn_str, error_str) = match formatting {
            true => (
                "Info".bold().green(),
                "Warn".bold().yellow(),
                "Error".bold().red(),
            ),
            false => ("Info".normal(), "Warn".normal(), "Error".normal()),
        };
        match log_type {
            LogLevel::Info => format!("[{}]{} {}", info_str, spacing, message),
            LogLevel::Warn => format!("[{}]{} {}", warn_str, spacing, message),
            LogLevel::Error => format!("[{}]{} {}", error_str, spacing, message),
        }
    }

    pub fn info(message: &str) -> Result<(), FileSaveError> {
        let this = Self::new();
        if this.output_file.is_some() {
            this.info_stdout(message);
            return this.info_file(message);
        }
        this.info_stdout(message);
        Ok(())
    }
    pub fn warn(message: &str) -> Result<(), FileSaveError> {
        let this = Self::new();
        if this.output_file.is_some() {
            this.warn_stdout(message);
            return this.warn_file(message);
        }
        this.warn_stdout(message);
        Ok(())
    }
    pub fn critical(message: &str) {
        let this = Self::new();
        if this.output_file.is_some() {
            println!("{}", Log::fmt(LogLevel::Error, "", message, true));
            this.critical_file(message);
        }
        this.critical_stdout(message);
    }
    pub fn critical_debug(message: &str, file: &str, line: u32) {
        let this = Self::new();
        if this.output_file.is_some() {
            println!(
                "{}",
                Log::fmt(
                    LogLevel::Error,
                    &format!("[{}:{}]", file, line),
                    message,
                    true,
                )
            );
            this.critical_debug_file(file, line, message);
        }
        this.critical_debug_stdout(file, line, message);
    }

    pub fn info_stdout(&self, message: &str) {
        if self.log_level > LogLevel::Info {
            return;
        }
        println!("{}", Log::fmt(LogLevel::Info, "", message, true,));
    }
    pub fn info_file(&self, message: &str) -> Result<(), FileSaveError> {
        if self.log_level > LogLevel::Info {
            return Ok(());
        }
        self.to_file(&Log::fmt(LogLevel::Info, "", message, false))
    }

    pub fn warn_stdout(&self, message: &str) {
        if self.log_level > LogLevel::Warn {
            return;
        }
        println!("{}", Log::fmt(LogLevel::Warn, "", message, true,));
    }
    pub fn warn_file(&self, message: &str) -> Result<(), FileSaveError> {
        if self.log_level > LogLevel::Warn {
            return Ok(());
        }
        self.to_file(&Log::fmt(LogLevel::Warn, "", message, false))
    }

    pub fn critical_stdout(&self, message: &str) -> ! {
        println!("{}", Log::fmt(LogLevel::Error, "", message, true));
        process::exit(1);
    }
    pub fn critical_file(&self, message: &str) -> ! {
        if let Err(e) = self.to_file(&Log::fmt(LogLevel::Error, "", message, false)) {
            println!("{:?}", e);
        }
        process::exit(1);
    }

    pub fn critical_debug_stdout(&self, file: &str, line: u32, message: &str) -> ! {
        println!(
            "{}",
            Log::fmt(
                LogLevel::Error,
                &format!("[{}:{}]", file, line),
                message,
                true,
            )
        );
        process::exit(1);
    }
    pub fn critical_debug_file(&self, file: &str, line: u32, message: &str) -> ! {
        if let Err(e) = self.to_file(&Log::fmt(
            LogLevel::Error,
            &format!("[{}:{}]", file, line),
            message,
            false,
        )) {
            println!("{:?}", e);
        }
        process::exit(1);
    }

    fn to_file(&self, data: &String) -> Result<(), FileSaveError> {
        let output_file = self.output_file.clone().unwrap();
        match OpenOptions::new().append(true).open(&output_file) {
            Ok(mut file) => {
                if writeln!(file, "{}", data).is_err() {
                    return Err(FileSaveError::FailedToOpenFile);
                }
                Ok(())
            }
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    return Err(FileSaveError::FailedToOpenFile);
                }
                let Ok(mut f) = File::create(&output_file) else {
                    return Err(FileSaveError::FailedToCreateFile);
                };
                if f.write(data.as_bytes()).is_err() {
                    return Err(FileSaveError::FailedToWriteFile);
                };
                Ok(())
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
