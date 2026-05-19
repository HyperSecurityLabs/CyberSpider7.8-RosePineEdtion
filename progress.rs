pub use crate::cyberwave_progress::*;

pub struct ProgressManager {
    pub theme: String,
    cyberwave: CyberWaveProgress,
}

impl ProgressManager {
    pub fn new(theme: &str) -> Self {
        let cyberwave_theme = match theme {
            "matrix" => ProgressTheme::Matrix,
            "neon" => ProgressTheme::Neon,
            "terminal" => ProgressTheme::Terminal,
            "rosepine" => ProgressTheme::RosePine,
            _ => ProgressTheme::CyberWave,
        };

        Self {
            theme: theme.to_string(),
            cyberwave: CyberWaveProgress::new(cyberwave_theme),
        }
    }

    pub fn display_rust_logo() {
        let cyberwave = CyberWaveProgress::new(ProgressTheme::CyberWave);
        cyberwave.display_cyberwave_logo();
    }

    pub fn create_spinner(&self) -> BrailleSpinner {
        self.cyberwave.create_spinner("Processing...")
    }

    pub fn create_bar(&self, _length: u64) -> ProgressBar {
        ProgressBar::new(0)
    }

    pub fn create_crawl_progress_bar(&self, _total_urls: u64, current_depth: usize) -> ProgressBar {
        println!("[DEPTH {}] Starting crawl...", current_depth);
        ProgressBar::new(0)
    }

    pub async fn update_crawl_progress(&self, processed: u64, total: u64, discovered: u64, current_depth: usize) {
        println!("[DEPTH {}] Progress: {}/{} URLs ({} discovered)", current_depth, processed, total, discovered);
    }

    pub fn display_discovery_alert(&self, url_count: usize, source: &str) {
        self.cyberwave.display_discovery_alert(url_count, source);
    }

    pub fn display_error_alert(&self, error: &str, url: &str) {
        self.cyberwave.display_error_alert(error, url);
    }

    pub fn display_scanning_status(&self, current_url: &str, total_urls: usize, processed_urls: usize, depth: usize) {
        self.cyberwave.display_scanning_status(current_url, total_urls, processed_urls, depth);
    }
}
