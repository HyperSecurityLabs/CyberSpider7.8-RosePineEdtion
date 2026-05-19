use colored::*;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::r#box::DiagonalBox;

/// CyberGreen — vibrant green for all URL highlighting across the tool
pub const URL_GREEN: (u8, u8, u8) = (80, 250, 123);
const SPINNER_TICK_MS: u64 = 160;

/// Global stdout lock to prevent interleaved terminal writes between spinner + logs
fn with_stdout<F: FnOnce(&mut std::io::StdoutLock)>(f: F) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    f(&mut handle);
    handle.flush().ok();
}

pub struct CyberWaveProgress {
    theme: ProgressTheme,
}

#[derive(Debug, Clone)]
pub enum ProgressTheme {
    CyberWave,
    Matrix,
    Neon,
    Terminal,
    RosePine,
}

impl CyberWaveProgress {
    pub fn new(theme: ProgressTheme) -> Self {
        Self { theme }
    }

    pub fn display_cyberwave_logo(&self) {
        let banner_content = if std::path::Path::new("banner.txt").exists() {
            match std::fs::read_to_string("banner.txt") {
                Ok(content) => content.lines().map(|s| s.to_string()).collect::<Vec<String>>(),
                Err(_) => self.get_default_banner().into_iter().map(|s| s.to_string()).collect(),
            }
        } else {
            self.get_default_banner().into_iter().map(|s| s.to_string()).collect()
        };

        let mut full_banner = banner_content.clone();
        full_banner.push("".to_string());
        full_banner.push("                    CYBERSPIDER v7.8.0pro - OFFENSIVE SECURITY".to_string());
        full_banner.push("                    Advanced Offensive Security Reconnaissance".to_string());
        full_banner.push("                    Author: Khaninkali".to_string());

        for (i, line) in full_banner.iter().enumerate() {
            let original_len = self.get_default_banner().len();
            match self.theme {
                ProgressTheme::RosePine => {
                    if i >= original_len {
                        println!("{}", line.truecolor(235, 111, 146).bold());
                    } else if i < 2 {
                        println!("{}", line.truecolor(49, 116, 143).bold());
                    } else if i < 4 {
                        println!("{}", line.truecolor(156, 207, 216).bold());
                    } else {
                        println!("{}", line.truecolor(196, 167, 231).bold());
                    }
                }
                ProgressTheme::CyberWave => {
                    if i >= original_len {
                        println!("{}", line.bright_red().bold());
                    } else {
                        println!("{}", line.cyan().bold());
                    }
                }
                ProgressTheme::Matrix => {
                    if i >= original_len {
                        println!("{}", line.bright_green().bold());
                    } else {
                        println!("{}", line.green());
                    }
                }
                ProgressTheme::Neon => {
                    if i >= original_len {
                        println!("{}", line.bright_magenta().bold());
                    } else {
                        println!("{}", line.magenta().bold());
                    }
                }
                ProgressTheme::Terminal => {
                    if i >= original_len {
                        println!("{}", line.bright_white().bold());
                    } else {
                        println!("{}", line.white());
                    }
                }
            }
        }
        println!();
    }

    pub fn display_scanning_status(&self, current_url: &str, total_urls: usize, processed_urls: usize, depth: usize) {
        let percentage = if total_urls > 0 {
            (processed_urls as f64 / total_urls as f64) * 100.0
        } else {
            0.0
        };

        let status = match self.theme {
            ProgressTheme::RosePine => {
                format!("⟦ ROSEPINE ⟧ Scanning: {} [{}/{} URLs] {:.1}% - Depth {}",
                    current_url.truecolor(224, 222, 244),
                    processed_urls.to_string().truecolor(196, 111, 146),
                    total_urls.to_string().truecolor(156, 207, 216),
                    percentage,
                    depth.to_string().truecolor(246, 193, 119)
                )
            }
            ProgressTheme::CyberWave => {
                format!("⟦ CYBERSPIDER ⟧ Scanning: {} [{}/{} URLs] {:.1}% - Depth {}",
                    current_url.bright_white(),
                    processed_urls.to_string().bright_green(),
                    total_urls.to_string().bright_cyan(),
                    percentage,
                    depth.to_string().bright_yellow()
                )
            }
            ProgressTheme::Matrix => {
                format!("◈ Scanning: {} [{}/{}] {:.1}% - Depth {}",
                    current_url.white(), processed_urls, total_urls, percentage, depth)
            }
            ProgressTheme::Neon => {
                format!("◆ Scanning: {} [{}/{}] {:.1}% - Depth {}",
                    current_url.bright_white(),
                    processed_urls.to_string().magenta(),
                    total_urls.to_string().bright_magenta(),
                    percentage,
                    depth.to_string().yellow()
                )
            }
            ProgressTheme::Terminal => {
                format!("✓ Scanning: {} [{}/{}] {:.1}% - Depth {}",
                    current_url.white(), processed_urls, total_urls, percentage, depth)
            }
        };
        println!("{}", status);
    }

    pub fn display_discovery_alert(&self, url_count: usize, source: &str) {
        let alert = match self.theme {
            ProgressTheme::RosePine => {
                format!("⟦ ROSEPINE ⟧ Discovered {} new URLs from {}",
                    url_count.to_string().truecolor(196, 111, 146).bold(),
                    source.truecolor(156, 207, 216)
                )
            }
            ProgressTheme::CyberWave => {
                format!("⟦ CYBERSPIDER ⟧ Discovered {} new URLs from {}",
                    url_count.to_string().bright_green().bold(),
                    source.bright_cyan()
                )
            }
            ProgressTheme::Matrix => format!("◈ Discovered {} URLs from {}", url_count, source),
            ProgressTheme::Neon => {
                format!("◆ Discovered {} URLs from {}",
                    url_count.to_string().magenta(),
                    source.bright_magenta()
                )
            }
            ProgressTheme::Terminal => format!("✓ Discovered {} URLs from {}", url_count, source),
        };
        println!("{}", alert);
    }

    pub fn display_error_alert(&self, error: &str, url: &str) {
        let alert = match self.theme {
            ProgressTheme::RosePine => {
                format!("⟦ ROSEPINE ⟧ Failed to crawl {}: {}",
                    url.truecolor(235, 111, 146),
                    error.truecolor(235, 111, 146)
                )
            }
            ProgressTheme::CyberWave => {
                format!("⟦ CYBERSPIDER ⟧ Failed to crawl {}: {}",
                    url.bright_red(), error.red())
            }
            ProgressTheme::Matrix => format!("☠ Failed to crawl {}: {}", url.red(), error.red()),
            ProgressTheme::Neon => format!("◉ Failed to crawl {}: {}", url.bright_red(), error.red()),
            ProgressTheme::Terminal => format!("✗ Failed to crawl {}: {}", url.red(), error.red()),
        };
        println!("{}", alert);
    }

    fn get_default_banner(&self) -> Vec<&'static str> {
        vec![
            "   ______      __              _____       _     __         _    _______",
            "  / ____/_  __/ /_  ___  _____/ ___/____  (_)___/ /__  ____| |  / /__  /",
            " / /   / / / / __ \\/ _ \\/ ___/\\__ \\/ __ \\/ / __  / _ \\/ ___/ | / /  / / ",
            "/ /___/ /_/ / /_/ /  __/ /   ___/ / /_/ / / /_/ /  __/ /   | |/ /  / /  ",
            "\\____/\\__, /_.___/\\___/_/   /____/ .___/_/\\__,_/\\___/_/    |___/  /_/   ",
            "     /____/                     /_/                                      ",
        ]
    }

    pub fn create_spinner(&self, message: &str) -> BrailleSpinner {
        match self.theme {
            ProgressTheme::RosePine => {
                BrailleSpinner::new(message, &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"], (196, 111, 146))
            }
            ProgressTheme::CyberWave => {
                BrailleSpinner::new(message, &["▰▰▰▰▱", "▰▰▰▱▰", "▰▰▱▰▰", "▰▱▰▰▰", "▱▰▰▰▰", "▰▱▰▰▰", "▰▰▱▰▰", "▰▰▰▱▰"], (0, 255, 255))
            }
            ProgressTheme::Matrix => {
                BrailleSpinner::new(message, &["◐", "◓", "◑", "◒"], (0, 255, 0))
            }
            ProgressTheme::Neon => {
                BrailleSpinner::new(message, &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"], (255, 0, 255))
            }
            ProgressTheme::Terminal => {
                BrailleSpinner::new(message, &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"], (255, 255, 255))
            }
        }
    }

    pub fn print_success(&self, message: &str) {
        match self.theme {
            ProgressTheme::RosePine => {
                let box_lines = DiagonalBox::create_diagonal_success_box(message);
                for line in box_lines { println!("{}", line); }
            }
            ProgressTheme::CyberWave => {
                let box_lines = DiagonalBox::create_diagonal_success_box(message);
                for line in box_lines { println!("{}", line); }
            }
            ProgressTheme::Matrix => println!("◈ {}", message.green()),
            ProgressTheme::Neon => println!("◆ {}", message.magenta().bold()),
            ProgressTheme::Terminal => println!("✓ {}", message.white()),
        }
    }

    pub fn print_error(&self, message: &str) {
        match self.theme {
            ProgressTheme::RosePine => {
                let box_lines = DiagonalBox::create_diagonal_error_box(message);
                for line in box_lines { println!("{}", line); }
            }
            ProgressTheme::CyberWave => {
                let box_lines = DiagonalBox::create_diagonal_error_box(message);
                for line in box_lines { println!("{}", line); }
            }
            ProgressTheme::Matrix => println!("☠ {}", message.red().bold()),
            ProgressTheme::Neon => println!("◉ {}", message.red().bold()),
            ProgressTheme::Terminal => println!("✗ {}", message.red()),
        }
    }

    pub fn print_warning(&self, message: &str) {
        match self.theme {
            ProgressTheme::RosePine => {
                let box_lines = DiagonalBox::create_diagonal_warning_box(message);
                for line in box_lines { println!("{}", line); }
            }
            ProgressTheme::CyberWave => {
                let box_lines = DiagonalBox::create_diagonal_warning_box(message);
                for line in box_lines { println!("{}", line); }
            }
            ProgressTheme::Matrix => println!("⚡ {}", message.yellow()),
            ProgressTheme::Neon => println!("⚡ {}", message.yellow().bold()),
            ProgressTheme::Terminal => println!("⚠ {}", message.yellow()),
        }
    }

    pub fn print_info(&self, message: &str) {
        match self.theme {
            ProgressTheme::RosePine => println!("ℹ {}", message.truecolor(156, 207, 216).bold()),
            ProgressTheme::CyberWave => println!("ℹ {}", message.cyan().bold()),
            ProgressTheme::Matrix => println!("◉ {}", message.cyan()),
            ProgressTheme::Neon => println!("○ {}", message.cyan().bold()),
            ProgressTheme::Terminal => println!("ℹ {}", message.cyan()),
        }
    }

    pub fn display_stats(&self, stats: &CrawlStats) {
        let total_str = stats.total_requests.to_string();
        let success_str = stats.successful_requests.to_string();
        let failed_str = stats.failed_requests.to_string();
        let urls_str = stats.urls_discovered.to_string();
        let subdomains_str = stats.subdomains_found.to_string();
        let s3_str = stats.s3_buckets_found.to_string();
        let duration_str = stats.duration_ms.to_string();
        let rps_str = format!("{:.2}", stats.requests_per_second);

        let stats_data = vec![
            ("Total Requests", total_str.as_str()),
            ("Successful", success_str.as_str()),
            ("Failed", failed_str.as_str()),
            ("URLs Discovered", urls_str.as_str()),
            ("Subdomains Found", subdomains_str.as_str()),
            ("S3 Buckets", s3_str.as_str()),
            ("Duration (ms)", duration_str.as_str()),
            ("Requests/sec", rps_str.as_str()),
        ];

        let box_lines = DiagonalBox::create_diagonal_stats_box(&stats_data);
        for line in box_lines { println!("{}", line); }
    }

    pub fn display_completion_box(&self, message: &str) {
        let box_lines = match self.theme {
            ProgressTheme::RosePine | ProgressTheme::CyberWave =>
                DiagonalBox::create_double_diagonal_box(60, 5, Some("CYBERSPIDER COMPLETE"), &[message.to_string()]),
            ProgressTheme::Matrix =>
                DiagonalBox::create_zigzag_box(60, 5, Some("SCAN COMPLETE"), &[message.to_string()]),
            ProgressTheme::Neon =>
                DiagonalBox::create_mixed_diagonal_box(60, 5, Some("NEON SCAN COMPLETE"), &[message.to_string()]),
            ProgressTheme::Terminal =>
                DiagonalBox::create_diagonal_box(60, 5, Some("TERMINAL COMPLETE"), &[message.to_string()]),
        };
        for line in box_lines { println!("{}", line); }
    }

    pub fn display_error_box(&self, message: &str) {
        let box_lines = match self.theme {
            ProgressTheme::RosePine | ProgressTheme::CyberWave =>
                DiagonalBox::create_zigzag_box(60, 5, Some("CYBERSPIDER ERROR"), &[message.to_string()]),
            ProgressTheme::Matrix =>
                DiagonalBox::create_diagonal_box(60, 5, Some("MATRIX ERROR"), &[message.to_string()]),
            ProgressTheme::Neon =>
                DiagonalBox::create_double_diagonal_box(60, 5, Some("NEON ERROR"), &[message.to_string()]),
            ProgressTheme::Terminal =>
                DiagonalBox::create_mixed_diagonal_box(60, 5, Some("TERMINAL ERROR"), &[message.to_string()]),
        };
        for line in box_lines { println!("{}", line); }
    }

    pub fn display_warning_box(&self, message: &str) {
        let box_lines = match self.theme {
            ProgressTheme::RosePine | ProgressTheme::CyberWave =>
                DiagonalBox::create_mixed_diagonal_box(60, 5, Some("CYBERSPIDER WARNING"), &[message.to_string()]),
            ProgressTheme::Matrix =>
                DiagonalBox::create_double_diagonal_box(60, 5, Some("MATRIX WARNING"), &[message.to_string()]),
            ProgressTheme::Neon =>
                DiagonalBox::create_zigzag_box(60, 5, Some("NEON WARNING"), &[message.to_string()]),
            ProgressTheme::Terminal =>
                DiagonalBox::create_diagonal_box(60, 5, Some("TERMINAL WARNING"), &[message.to_string()]),
        };
        for line in box_lines { println!("{}", line); }
    }

    pub fn display_info_box(&self, message: &str) {
        let box_lines = match self.theme {
            ProgressTheme::RosePine | ProgressTheme::CyberWave =>
                DiagonalBox::create_diagonal_box(60, 5, Some("CYBERSPIDER INFO"), &[message.to_string()]),
            ProgressTheme::Matrix =>
                DiagonalBox::create_mixed_diagonal_box(60, 5, Some("MATRIX INFO"), &[message.to_string()]),
            ProgressTheme::Neon =>
                DiagonalBox::create_zigzag_box(60, 5, Some("NEON INFO"), &[message.to_string()]),
            ProgressTheme::Terminal =>
                DiagonalBox::create_double_diagonal_box(60, 5, Some("TERMINAL INFO"), &[message.to_string()]),
        };
        for line in box_lines { println!("{}", line); }
    }
}

pub struct BrailleSpinner {
    message: Arc<Mutex<String>>,
    running: Arc<AtomicBool>,
    _chars: &'static [&'static str],
    _color: (u8, u8, u8),
}

impl BrailleSpinner {
    pub fn new(message: &str, chars: &'static [&'static str], color: (u8, u8, u8)) -> Self {
        let msg = Arc::new(Mutex::new(message.to_string()));
        let running = Arc::new(AtomicBool::new(true));

        let msg_clone = msg.clone();
        let run_clone = running.clone();

        tokio::spawn(async move {
            let mut idx = 0usize;
            while run_clone.load(Ordering::Relaxed) {
                let ch = chars[idx % chars.len()];
                let m = msg_clone.lock().unwrap().clone();
                with_stdout(|handle| {
                    write!(handle,
                        "\r{} {} \x1b[K",
                        ch.truecolor(color.0, color.1, color.2),
                        m.truecolor(color.0, color.1, color.2)
                    ).ok();
                });
                idx = (idx + 1) % chars.len();
                tokio::time::sleep(Duration::from_millis(SPINNER_TICK_MS)).await;
                tokio::task::yield_now().await;
            }
        });

        Self { message: msg, running, _chars: chars, _color: color }
    }

    pub fn set_message(&self, message: &str) {
        *self.message.lock().unwrap() = message.to_string();
    }

    pub fn log(&self, message: &str) {
        with_stdout(|handle| {
            write!(handle, "\r\x1b[K\n").ok();
            writeln!(handle, "{}", message).ok();
        });
    }

    /// Throttled log — tracks last-log time and skips if called too fast
    pub fn log_throttled(&self, message: &str, last: &mut std::time::Instant, min_interval_ms: u64) {
        let now = std::time::Instant::now();
        if now.duration_since(*last).as_millis() as u64 >= min_interval_ms {
            self.log(message);
            *last = now;
        } else {
            // Still update the spinner message so the status is visible
            self.set_message(message);
        }
    }

    pub fn finish(&self) {
        self.running.store(false, Ordering::Relaxed);
        with_stdout(|handle| {
            write!(handle, "\r\x1b[K").ok();
        });
    }
}

#[derive(Debug, Default)]
pub struct CrawlStats {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub urls_discovered: usize,
    pub subdomains_found: usize,
    pub s3_buckets_found: usize,
    pub duration_ms: u64,
    pub requests_per_second: f64,
}

impl CrawlStats {
    pub fn calculate_rps(&mut self) {
        if self.duration_ms > 0 {
            self.requests_per_second = (self.total_requests as f64) / (self.duration_ms as f64 / 1000.0);
        }
    }

    pub fn display_with_progress(&self, progress: &CyberWaveProgress) {
        progress.display_stats(self);
    }

    pub fn create_summary_message(&self) -> String {
        format!(
            "Scan completed: {} URLs discovered, {} successful, {} failed in {}ms",
            self.urls_discovered,
            self.successful_requests,
            self.failed_requests,
            self.duration_ms
        )
    }
}

pub struct ProgressBar;

impl ProgressBar {
    pub fn new(_total: u64) -> Self { Self }
    pub fn set_length(&self, _len: u64) {}
    pub fn set_position(&self, _pos: u64) {}
    pub fn set_message(&self, _msg: &str) {}
    pub fn inc(&self, _delta: u64) {}
    pub fn finish(&self) {}
    pub fn finish_and_clear(&self) {}
    pub fn println(&self, msg: &str) { println!("{}", msg); }
}
