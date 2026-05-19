use anyhow::Result;
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::path::Path;
use colored::Colorize;
use crossterm::style::Stylize;
use crate::monitoring::{LogEntry, LogLevel, Logger};

pub struct StructuredLogger {
    file_writer: Option<BufWriter<File>>,
    console_output: bool,
    log_level: LogLevel,
    buffer: Vec<LogEntry>,
    max_buffer_size: usize,
}

impl StructuredLogger {
    pub fn new(config: LoggerConfig) -> Result<Self> {
        let file_writer = if let Some(log_file) = config.file_path {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file)?;
            Some(BufWriter::new(file))
        } else {
            None
        };

        Ok(Self {
            file_writer,
            console_output: config.console_output,
            log_level: config.log_level,
            buffer: Vec::new(),
            max_buffer_size: config.max_buffer_size,
        })
    }

    fn should_log(&self, level: &LogLevel) -> bool {
        match (&self.log_level, level) {
            (LogLevel::Trace, _) => true,
            (LogLevel::Debug, LogLevel::Debug | LogLevel::Info | LogLevel::Warn | LogLevel::Error) => true,
            (LogLevel::Info, LogLevel::Info | LogLevel::Warn | LogLevel::Error) => true,
            (LogLevel::Warn, LogLevel::Warn | LogLevel::Error) => true,
            (LogLevel::Error, LogLevel::Error) => true,
            _ => false,
        }
    }

    fn format_log_entry(&self, entry: &LogEntry) -> String {
        let level_str = match entry.level {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };

        let mut metadata_str = String::new();
        if !entry.metadata.is_empty() {
            let metadata_json = serde_json::to_string(&entry.metadata).unwrap_or_default();
            metadata_str = format!(" metadata={}", metadata_json);
        }

        let session_str = entry.session_id
            .as_ref()
            .map(|id| format!(" session={}", id))
            .unwrap_or_default();

        let url_str = entry.url
            .as_ref()
            .map(|url| format!(" url={}", url))
            .unwrap_or_default();

        format!(
            "{} [{}] {}{}{}{} {}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            level_str,
            entry.module,
            session_str,
            url_str,
            metadata_str,
            entry.message
        )
    }

    fn write_to_console(&self, entry: &LogEntry) {
        let colored_message = match entry.level {
            LogLevel::Trace => entry.message.clone().bright_black().to_string(),
            LogLevel::Debug => entry.message.clone().bright_black().to_string(),
            LogLevel::Info => entry.message.clone().blue().to_string(),
            LogLevel::Warn => entry.message.clone().yellow().to_string(),
            LogLevel::Error => entry.message.clone().red().to_string(),
        };

        let level_symbol = match entry.level {
            LogLevel::Trace => "·",
            LogLevel::Debug => "·",
            LogLevel::Info => "ℹ",
            LogLevel::Warn => "⚠",
            LogLevel::Error => "✗",
        };

        let level_color = match entry.level {
            LogLevel::Trace => colored::Color::BrightBlack,
            LogLevel::Debug => colored::Color::BrightBlack,
            LogLevel::Info => colored::Color::Blue,
            LogLevel::Warn => colored::Color::Yellow,
            LogLevel::Error => colored::Color::Red,
        };

        println!(
            "{} {} [{}] {}",
            entry.timestamp.format("%H:%M:%S%.3f"),
            level_symbol.color(level_color),
            entry.module,
            colored_message
        );
    }
}

impl Logger for StructuredLogger {
    fn log(&mut self, entry: LogEntry) {
        if !self.should_log(&entry.level) {
            return;
        }

        if self.console_output {
            self.write_to_console(&entry);
        }

        let formatted = self.format_log_entry(&entry);
        if let Some(ref mut writer) = self.file_writer {
            if let Err(e) = writeln!(writer, "{}", formatted) {
                eprintln!("Failed to write to log file: {}", e);
            }
        }

        self.buffer.push(entry);
        if self.buffer.len() > self.max_buffer_size {
            self.buffer.drain(0..self.buffer.len() / 2);
        }
    }

    fn flush(&mut self) {
        if let Some(ref mut writer) = self.file_writer {
            if let Err(e) = writer.flush() {
                eprintln!("Failed to flush log file: {}", e);
            }
        }
    }

    fn get_logs(&self, limit: Option<usize>) -> Vec<LogEntry> {
        match limit {
            Some(limit) => self.buffer.iter().rev().take(limit).cloned().collect(),
                   None => self.buffer.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub file_path: Option<String>,
    pub console_output: bool,
    pub log_level: LogLevel,
    pub max_buffer_size: usize,
    pub rotation_size_mb: Option<u64>,
    pub compression: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            file_path: Some("cyberspider.log".to_string()),
            console_output: true,
            log_level: LogLevel::Info,
            max_buffer_size: 10000,
            rotation_size_mb: Some(100),
            compression: false,
        }
    }
}

pub struct JsonLogger {
    writer: BufWriter<File>,
}

impl JsonLogger {
    pub fn new<P: AsRef<Path>>(log_file: P) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;
        
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }
}

impl Logger for JsonLogger {
    fn log(&mut self, entry: LogEntry) {
        let json_line = serde_json::to_string(&entry).unwrap_or_default();
        if let Err(e) = writeln!(self.writer, "{}", json_line) {
            eprintln!("Failed to write JSON log: {}", e);
        }
    }

    fn flush(&mut self) {
        if let Err(e) = self.writer.flush() {
            eprintln!("Failed to flush JSON log: {}", e);
        }
    }

    fn get_logs(&self, _limit: Option<usize>) -> Vec<LogEntry> {
        Vec::new() // JSON logger doesn't keep logs in memory
    }
}

pub struct AsyncLogger {
    sender: std::sync::mpsc::Sender<LogEntry>,
    _handle: std::thread::JoinHandle<()>,
}

impl AsyncLogger {
    pub fn new(config: LoggerConfig) -> Result<Self> {
        let (sender, receiver) = std::sync::mpsc::channel::<LogEntry>();
        
        let mut logger = StructuredLogger::new(config)?;
        
        let handle = std::thread::spawn(move || {
            for entry in receiver {
                logger.log(entry);
            }
            logger.flush();
        });

        Ok(Self {
            sender,
            _handle: handle,
        })
    }
}

impl Logger for AsyncLogger {
    fn log(&mut self, entry: LogEntry) {
        if let Err(_) = self.sender.send(entry) {
            eprintln!("Failed to send log entry to async logger");
        }
    }

    fn flush(&mut self) {
        // Async logger flushes automatically
    }

    fn get_logs(&self, _limit: Option<usize>) -> Vec<LogEntry> {
        Vec::new() // Async logger doesn't provide access to buffered logs
    }
}
