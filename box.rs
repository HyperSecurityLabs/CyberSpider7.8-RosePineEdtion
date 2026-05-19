//! Cyberpunk Diagonal Box System for CyberSpider
//! 
//! Advanced diagonal box designs with | / \ // characters
//! Creating fully diagonal cyberpunk-style boxes

use colored::*;

pub struct DiagonalBox;

impl DiagonalBox {
    /// Create a fully diagonal box with | / \ characters
    pub fn create_diagonal_box(width: usize, height: usize, title: Option<&str>, content: &[String]) -> Vec<String> {
        let mut lines = Vec::new();

        // Top border with diagonal pattern
        let top_border = format!("/{}\\", "/\\".repeat(width / 2));
        lines.push(top_border.bright_cyan().to_string());

        // Title line if provided
        if let Some(title_text) = title {
            let title_len = title_text.len();
            if title_len <= width - 4 {
                let padding = (width - 2 - title_len) / 2;
                let title_line = format!("|{}{}{}|", 
                    " ".repeat(padding), 
                    title_text.bright_yellow().bold(), 
                    " ".repeat(width - 2 - padding - title_len)
                );
                lines.push(title_line.to_string());
            } else {
                let truncated = &title_text[..width - 5];
                let title_line = format!("| {}... |", truncated.bright_yellow().bold());
                lines.push(title_line.to_string());
            }
        }

        // Content lines with diagonal borders
        for line in content {
            let line_len = line.len();
            if line_len <= width - 4 {
                let padding = width - 2 - line_len;
                let content_line = format!("|{}{}{}|", 
                    " ".repeat(2), 
                    line.bright_cyan(), 
                    " ".repeat(padding - 2)
                );
                lines.push(content_line.to_string());
            } else {
                let truncated = &line[..width - 5];
                let content_line = format!("| {}... |", truncated.bright_cyan());
                lines.push(content_line.to_string());
            }
        }

        // Fill empty space if needed
        let current_height = lines.len();
        if current_height < height - 1 {
            for _ in 0..(height - 1 - current_height) {
                let empty_line = format!("|{}|", " ".repeat(width - 2));
                lines.push(empty_line);
            }
        }

        // Bottom border with diagonal pattern
        let bottom_border = format!("\\{}/", "\\/".repeat(width / 2));
        lines.push(bottom_border.bright_cyan().to_string());

        lines
    }

    /// Create a double diagonal box with enhanced pattern
    pub fn create_double_diagonal_box(width: usize, height: usize, title: Option<&str>, content: &[String]) -> Vec<String> {
        let mut lines = Vec::new();

        // Enhanced top border with double diagonal
        let top_border = format!("//{}\\\\", "//".repeat(width / 3));
        lines.push(top_border.bright_magenta().to_string());

        // Title line
        if let Some(title_text) = title {
            let title_len = title_text.len();
            if title_len <= width - 4 {
                let padding = (width - 2 - title_len) / 2;
                let title_line = format!("||{}{}{}||", 
                    " ".repeat(padding), 
                    title_text.bright_magenta().bold(), 
                    " ".repeat(width - 2 - padding - title_len)
                );
                lines.push(title_line.to_string());
            }
        }

        // Content lines with double borders
        for line in content {
            let line_len = line.len();
            if line_len <= width - 6 {
                let padding = width - 4 - line_len;
                let content_line = format!("|| {}{}{} ||", 
                    " ".repeat(1), 
                    line.bright_white(), 
                    " ".repeat(padding - 1)
                );
                lines.push(content_line.to_string());
            }
        }

        // Fill empty space
        let current_height = lines.len();
        if current_height < height - 1 {
            for _ in 0..(height - 1 - current_height) {
                let empty_line = format!("||{}||", " ".repeat(width - 4));
                lines.push(empty_line.to_string());
            }
        }

        // Enhanced bottom border
        let bottom_border = format!("\\\\{}//", "\\\\".repeat(width / 3));
        lines.push(bottom_border.bright_magenta().to_string());

        lines
    }

    /// Create a zigzag diagonal box
    pub fn create_zigzag_box(width: usize, height: usize, title: Option<&str>, content: &[String]) -> Vec<String> {
        let mut lines = Vec::new();

        // Zigzag top border
        let mut top_border = String::new();
        for i in 0..width {
            if i % 4 == 0 || i % 4 == 1 {
                top_border.push('/');
            } else {
                top_border.push('\\');
            }
        }
        lines.push(top_border.bright_green().to_string());

        // Title line
        if let Some(title_text) = title {
            let title_len = title_text.len();
            if title_len <= width - 4 {
                let padding = (width - 2 - title_len) / 2;
                let title_line = format!("|{}{}{}|", 
                    " ".repeat(padding), 
                    title_text.bright_green().bold(), 
                    " ".repeat(width - 2 - padding - title_len)
                );
                lines.push(title_line.to_string());
            }
        }

        // Content lines
        for line in content {
            let line_len = line.len();
            if line_len <= width - 4 {
                let padding = width - 2 - line_len;
                let content_line = format!("|{}{}{}|", 
                    " ".repeat(2), 
                    line.bright_cyan(), 
                    " ".repeat(padding - 2)
                );
                lines.push(content_line.to_string());
            }
        }

        // Fill empty space
        let current_height = lines.len();
        if current_height < height - 1 {
            for _ in 0..(height - 1 - current_height) {
                let empty_line = format!("|{}|", " ".repeat(width - 2));
                lines.push(empty_line);
            }
        }

        // Zigzag bottom border (reverse pattern)
        let mut bottom_border = String::new();
        for i in 0..width {
            if i % 4 == 2 || i % 4 == 3 {
                bottom_border.push('/');
            } else {
                bottom_border.push('\\');
            }
        }
        lines.push(bottom_border.bright_green().to_string());

        lines
    }

    /// Create a mixed diagonal box with | / \ // patterns
    pub fn create_mixed_diagonal_box(width: usize, height: usize, title: Option<&str>, content: &[String]) -> Vec<String> {
        let mut lines = Vec::new();

        // Mixed pattern top border
        let top_border = format!(" {}", "_".repeat(width - 2));
        lines.push(top_border.bright_green().to_string());

        // Title line
        if let Some(title_text) = title {
            let title_len = title_text.len();
            if title_len <= width - 4 {
                let padding = (width - 2 - title_len) / 2;
                let title_line = format!(" {}{}{} ", 
                    " ".repeat(padding), 
                    title_text.bright_green().bold(), 
                    " ".repeat(width - 2 - padding - title_len)
                );
                lines.push(title_line.to_string());
            }
        }

        // Content lines with mixed borders
        for line in content {
            let line_len = line.len();
            if line_len <= width - 6 {
                let padding = width - 4 - line_len;
                let content_line = format!("|/{}{}{}\\|", 
                    " ".repeat(1), 
                    line.bright_white(), 
                    " ".repeat(padding - 1)
                );
                lines.push(content_line.to_string());
            }
        }

        // Fill empty space
        let current_height = lines.len();
        if current_height < height - 1 {
            for _ in 0..(height - 1 - current_height) {
                let empty_line = format!(" {}", " ".repeat(width - 2));
                lines.push(empty_line);
            }
        }

        // Mixed pattern bottom border
        let bottom_border = format!(" {}", "_".repeat(width - 2));
        lines.push(bottom_border.bright_green().to_string());

        lines
    }

    /// Create a specialized stats box with diagonal design
    pub fn create_diagonal_stats_box(stats: &[(&str, &str)]) -> Vec<String> {
        let max_label_len = stats.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
        let max_value_len = stats.iter().map(|(_, value)| value.len()).max().unwrap_or(0);
        let box_width = (max_label_len + max_value_len + 10).max(60);
        let box_height = stats.len() + 4;

        let title = "CYBERSPIDER STATISTICS";
        let mut content = Vec::new();

        for (label, value) in stats {
            let padded_label = format!("{}:", label);
            let line = format!("{:<25} {:>15}", padded_label, value);
            content.push(line);
        }

        Self::create_mixed_diagonal_box(box_width, box_height, Some(title), &content)
    }

    /// Create a diagonal error box
    pub fn create_diagonal_error_box(message: &str) -> Vec<String> {
        let title = "SYSTEM ERROR";
        let content = vec![message.to_string()];
        Self::create_zigzag_box(60, 5, Some(title), &content)
    }

    /// Create a diagonal success box
    pub fn create_diagonal_success_box(message: &str) -> Vec<String> {
        let title = "OPERATION COMPLETE";
        let content = vec![message.to_string()];
        Self::create_double_diagonal_box(50, 5, Some(title), &content)
    }

    /// Create a diagonal warning box
    pub fn create_diagonal_warning_box(message: &str) -> Vec<String> {
        let title = "SYSTEM WARNING";
        let content = vec![message.to_string()];
        Self::create_diagonal_box(50, 5, Some(title), &content)
    }
}

/// Diagonal box styles
#[derive(Debug, Clone, Copy)]
pub enum DiagonalBoxStyle {
    Simple,      // Simple | / \ pattern
    Double,      // Double // \\ pattern
    Zigzag,      // Zigzag pattern
    Mixed,       // Mixed | / \ // pattern
}

/// Convenience functions for quick diagonal box creation
pub mod convenience {
    use super::*;

    /// Quick stats display with diagonal box
    pub fn quick_diagonal_stats(total_requests: usize, successful: usize, failed: usize, urls: usize) {
        let total_str = total_requests.to_string();
        let success_str = successful.to_string();
        let failed_str = failed.to_string();
        let urls_str = urls.to_string();
        
        let stats = vec![
            ("Total Requests", total_str.as_str()),
            ("Successful", success_str.as_str()),
            ("Failed", failed_str.as_str()),
            ("URLs Discovered", urls_str.as_str()),
        ];

        let box_lines = DiagonalBox::create_diagonal_stats_box(&stats);
        for line in box_lines {
            println!("{}", line);
        }
    }

    /// Quick error display with diagonal box
    pub fn quick_diagonal_error(message: &str) {
        let box_lines = DiagonalBox::create_diagonal_error_box(message);
        for line in box_lines {
            println!("{}", line);
        }
    }

    /// Quick success display with diagonal box
    pub fn quick_diagonal_success(message: &str) {
        let box_lines = DiagonalBox::create_diagonal_success_box(message);
        for line in box_lines {
            println!("{}", line);
        }
    }

    /// Quick warning display with diagonal box
    pub fn quick_diagonal_warning(message: &str) {
        let box_lines = DiagonalBox::create_diagonal_warning_box(message);
        for line in box_lines {
            println!("{}", line);
        }
    }
}
